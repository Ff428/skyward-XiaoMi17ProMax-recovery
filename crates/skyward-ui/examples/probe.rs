//! 在真机上把一页画出来、存成 PNG，并把每个可点区域的坐标打出来。
//!
//! 用途：recovery 里没有 SurfaceFlinger，`screencap` 是空壳、没有 framebuffer
//! 设备，屏幕截不到。这个探针用同一份 `skyward-ui`、同一套字体，在手机上把
//! `App::compose` 的结果写进一张 1200×2608 的画布，直接出 PNG；同时把
//! `Frame.hits` 的每条（动作 + 矩形）打出来，相当于“文本版截图”。
//!
//! 用法：
//!   probe <fonts_dir> <settings.json|-> <screen> [out.png] [选项]
//!     screen: home | order | partitions | settings ...
//!       order  = 首页排序（HomeOrder）
//!     选项：
//!       --tap X,Y        在 (X,Y) 点一下再画（X/Y 是屏幕像素）
//!       --long X,Y       在 (X,Y) 长按 600 ms（拎起来）
//!       --move X,Y       接着上一个长按，手指挪到 (X,Y)
//!       --up             松手
//!       --page N         排序页先翻到第 N 屏（滚动位置 = N 屏高）
//!       --bench N        量 N 次「拖动一格 → 重画」的耗时，打印中位数
//!       --seq  a,b;c,d   依次投一串触摸（见下）
//!
//! `--seq` 的写法：段与段用 `;` 分开，每段是 `down:x,y` / `move:x,y` /
//! `up` / `wait:ms`，全部按 16 ms 一帧推进，方便一次跑完一段手势。

use skyward_info::{Battery, Boot, Charge, Cpu, DeviceInfo, Storage};
use skyward_vab::{Slot, SnapshotState};
use skyward_ui::app::Screen;
use skyward_ui::settings::Settings;
use skyward_ui::{App, Canvas, Fonts};

const W: u32 = 1200;
const H: u32 = 2608;
const PPD: f32 = 3.0;
const FRAME_MS: u64 = 16;

fn info() -> DeviceInfo {
    DeviceInfo {
        model: "Xiaomi 17 Pro Max".into(),
        codename: "popsicle".into(),
        soc: None,
        cpu: Cpu { cores: 8, max_khz: Some(4_320_000) },
        ram_kib: 16 * 1024 * 1024,
        panel: Some((W, H)),
        screen: Some(6.9),
        os_version: Some("OS4.0.8.0".into()),
        android: Some("16".into()),
        locale: Some("zh-CN".into()),
        time_zone: Some("Asia/Shanghai".into()),
        security_patch: None,
        kernel: "6.12.23".into(),
        storage: Storage { total: 1024 * 1024 * 1024 * 1024, free: Some(512 * 1024 * 1024 * 1024) },
        battery: Some(Battery {
            level: 62,
            charge: Charge::NotCharging,
            temp: Some(315),
            health: Some("Good".into()),
            cycles: None,
            design: None,
            full: None,
            voltage: None,
            current: None,
            quick: false,
            plugged: false,
        }),
        slot: Some(Slot::A),
        snapshot: SnapshotState::None,
        boot: Boot { unlocked: None, verified: None },
    }
}

fn screen_of(name: &str) -> Screen {
    match name {
        "home" => Screen::Home,
        "order" | "home_order" | "homeorder" => Screen::HomeOrder,
        "partitions" | "part" => Screen::Partitions,
        "settings" => Screen::Settings,
        "wlan" => Screen::Wlan,
        other => panic!("不认识的页面：{other}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).cloned().unwrap_or_else(|| "/skyward".into());
    let settings_path = args.get(2).cloned().unwrap_or_else(|| "-".into());
    let screen_name = args.get(3).cloned().unwrap_or_else(|| "order".into());
    let named_png = args.get(4).filter(|a| !a.starts_with("--")).cloned();
    let out_path = named_png.clone().unwrap_or_else(|| "/data/local/tmp/probe.png".into());

    let f = Fonts::load(format!("{dir}/MiSansVF.ttf"), 0, format!("{dir}/KirkMono-Regular.ttf")).expect("load the fonts");
    let i = info();

    let mut app = App::new(screen_of(&screen_name), W, H, PPD);
    app.set_screen_corners(140.0);
    if settings_path != "-" {
        let text = std::fs::read_to_string(&settings_path).expect("read the settings");
        let kept = Settings::read(&text).expect("parse the settings");
        app.take_settings(Some(&kept), Some(skyward_ui::Lang::Hans));
    } else {
        app.take_settings(None, Some(skyward_ui::Lang::Hans));
    }

    // 分区页：这台机器 by-name 底下 166 条，名字与大小照抄真机的量级。
    if matches!(screen_of(&screen_name), Screen::Partitions) && app.state.partitions.is_none() {
        let mut rows = Vec::new();
        for k in 0..166 {
            let word = ["boot", "dtbo", "vendor", "system", "cust", "modem", "dsp"][k % 7];
            let tail = ["a", "b", "c", "d", "e", "f", "g", "h"][k % 8];
            let name = format!("{word}_{tail}");
            rows.push((name, 1024 * 1024 * (1 + k as u64 % 97)));
        }
        app.state.partitions = Some(Ok(rows));
        app.refresh();
    }

    let mut now = 1000u64;
    let mut out = Canvas::new(W, H);
    // 稳定下来：标题、状态栏都需要几帧。
    for _ in 0..6 {
        now += FRAME_MS;
        let _ = app.tick(now);
        if app.take_dirty() {
            app.compose(&mut out, &f, &i, "07:40", now);
        }
    }

    // 参数解析：按顺序执行，边跑边画。
    let mut idx = if named_png.is_some() { 5 } else { 4 };
    let mut bench: Option<usize> = None;
    let mut shot_done = false;
    let mut pending_down: Option<(f32, f32)> = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "--tap" => {
                let (x, y) = pair(&args[idx + 1]);
                now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Down(x, y), &f, now);
                now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Up, &f, now);
                settle(&mut app, &mut out, &f, &i, &mut now, 4);
                idx += 2;
            }
            "--long" => {
                let (x, y) = pair(&args[idx + 1]);
                now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Down(x, y), &f, now);
                // 长按门限 400 ms，多等一帧。
                for _ in 0..30 {
                    now += FRAME_MS;
                    let _ = app.tick(now);
                }
                let _ = app.take_dirty();
                pending_down = Some((x, y));
                idx += 2;
            }
            "--move" => {
                let (x, y) = pair(&args[idx + 1]);
                now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Move(x, y), &f, now);
                settle(&mut app, &mut out, &f, &i, &mut now, 2);
                idx += 2;
            }
            "--up" => {
                now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Up, &f, now);
                settle(&mut app, &mut out, &f, &i, &mut now, 4);
                pending_down = None;
                idx += 1;
            }
            "--page" => {
                let n: f32 = args[idx + 1].parse().expect("a screen count");
                app.scroll_to_y(n * H as f32);
                settle(&mut app, &mut out, &f, &i, &mut now, 4);
                idx += 2;
            }
            "--seq" => {
                run_seq(&mut app, &mut out, &f, &i, &mut now, &args[idx + 1]);
                idx += 2;
            }
            "--bench" => {
                bench = Some(args[idx + 1].parse().expect("a count"));
                idx += 2;
            }
            "--shot" => {
                idx += 1;
            }
            other => {
                eprintln!("多余的参数：{other}");
                idx += 1;
            }
        }
    }

    // 再画一帧，保证 PNG 是最新的。
    now += FRAME_MS;
    let _ = app.tick(now);
    if app.take_dirty() {
        app.compose(&mut out, &f, &i, "07:40", now);
    }

    println!("page: {}", screen_name);
    println!("scroll: {:.1}", app.scroll_y());
    dump_hits(&mut app, &f, &i);

    let file = std::fs::File::create(&out_path).expect("create the png");
    out.write_png(std::io::BufWriter::new(file)).expect("write the png");
    println!("png: {out_path}");

    if let Some(n) = bench {
        bench_drag(&mut app, &mut out, &f, &i, n, pending_down.is_some());
    }
}

fn settle(app: &mut App, out: &mut Canvas, f: &Fonts, i: &DeviceInfo, now: &mut u64, frames: usize) {
    for _ in 0..frames {
        *now += FRAME_MS;
        let _ = app.tick(*now);
        if app.take_dirty() {
            app.compose(out, f, i, "07:40", *now);
        }
    }
}

fn run_seq(app: &mut App, out: &mut Canvas, f: &Fonts, i: &DeviceInfo, now: &mut u64, seq: &str) {
    for step in seq.split(';') {
        let (verb, arg) = match step.split_once(':') {
            Some((v, a)) => (v.trim(), a.trim()),
            None => (step.trim(), ""),
        };
        match verb {
            "down" => {
                let (x, y) = pair(arg);
                *now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Down(x, y), f, *now);
            }
            "move" => {
                let (x, y) = pair(arg);
                *now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Move(x, y), f, *now);
            }
            "up" => {
                *now += FRAME_MS;
                let _ = app.touch(skyward_ui::app::Touch::Up, f, *now);
            }
            "wait" => {
                let ms: u64 = arg.parse().expect("milliseconds");
                let end = *now + ms;
                while *now < end {
                    *now += FRAME_MS;
                    let _ = app.tick(*now);
                }
            }
            "" => {}
            other => eprintln!("seq 里不认识的一步：{other}"),
        }
        // 每一步都重画：和真机上「每一帧都刷新」一致。
        *now += FRAME_MS;
        let _ = app.tick(*now);
        if app.take_dirty() {
            app.compose(out, f, i, "07:40", *now);
        }
    }
}

fn pair(s: &str) -> (f32, f32) {
    let (a, b) = s.split_once(',').expect("x,y");
    (a.trim().parse().expect("x"), b.trim().parse().expect("y"))
}

fn dump_hits(app: &mut App, f: &Fonts, i: &DeviceInfo) {
    let frame = app.frame(f, i);
    let mut hits: Vec<(f32, f32, f32, f32, String)> = frame
        .hits
        .iter()
        .map(|h| (h.rect.x, h.rect.y, h.rect.w, h.rect.h, format!("{:?}", h.action)))
        .collect();
    hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    println!("hits: {}", hits.len());
    for (x, y, w, h, a) in hits {
        println!("  y={y:>7.1} h={h:>6.1} x={x:>6.1} w={w:>6.1}  {a}");
    }
}

/// 量拖动：从排序页第一条拎起来，往下挪十行，每一步「touch + 重画」各记一次。
fn bench_drag(app: &mut App, out: &mut Canvas, f: &Fonts, i: &DeviceInfo, n: usize, holding: bool) {
    use std::time::Instant;
    let first = app
        .frame(f, i)
        .hits
        .iter()
        .filter(|h| matches!(h.action, skyward_ui::app::Action::HomePick(_)))
        .map(|h| (h.rect.y, h.rect.x + h.rect.w / 2.0, h.rect.h))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let Some((y0, x0, row_h)) = first else {
        println!("bench: 这一页没有可拎的行");
        return;
    };
    let mut now = 100_000u64;
    if !holding {
        let _ = app.touch(skyward_ui::app::Touch::Down(x0, y0 + row_h / 2.0), f, now);
        for _ in 0..30 {
            now += FRAME_MS;
            let _ = app.tick(now);
        }
        let _ = app.take_dirty();
    }
    let mut touch_ms = Vec::new();
    let mut layout_ms = Vec::new();
    let mut draw_ms = Vec::new();
    for k in 0..n {
        let y = y0 + row_h / 2.0 + (k as f32 * 24.0).sin() * row_h * 2.0;
        now += FRAME_MS;
        let t0 = Instant::now();
        let _ = app.touch(skyward_ui::app::Touch::Move(x0, y), f, now);
        touch_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        let _ = app.tick(now);
        if app.take_dirty() {
            // 先单独量「这一页重新排版 + 画一条带」，再量整屏合成。
            let t1 = Instant::now();
            let _ = app.frame(f, i);
            layout_ms.push(t1.elapsed().as_secs_f64() * 1000.0);
            let t2 = Instant::now();
            app.compose(out, f, i, "07:40", now);
            draw_ms.push(t2.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let _ = app.touch(skyward_ui::app::Touch::Up, f, now);
    println!("bench: {} 步，每步 touch + compose", n);
    report("touch", &mut touch_ms);
    report("page", &mut layout_ms);
    report("compose", &mut draw_ms);
}

fn report(name: &str, v: &mut Vec<f64>) {
    if v.is_empty() {
        println!("  {name}: 没有采样");
        return;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let p = |q: f64| v[((n as f64 - 1.0) * q).round() as usize];
    let sum: f64 = v.iter().sum();
    println!(
        "  {name}: n={n} 平均 {:.2} ms  p50 {:.2}  p90 {:.2}  最大 {:.2}",
        sum / n as f64,
        p(0.5),
        p(0.9),
        v[n - 1]
    );
}
