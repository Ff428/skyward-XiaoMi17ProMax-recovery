use crate::app::{
    Action, BackupAt, BackupEnd, BackupJob, Check, ClashChoice, Confirm, Damage, Data, Dock, DownloadEnd, EraseStop, Field, FileKind,
    FileOp, FileTask, Folder, Frame, FsAction, FsChoice, FsRow, Have, HeroArt, Hit, Install, JobEnd, Keys, Pick, Place, Rows, Screen, SearchScope,
    Fetch, RecordSetting, Serving, SkywardCheck, SortKey, SkywardEnd, SkywardJob, SkywardStage, Slider, ImageAction, ImageCopy, Region, Source, State, Switch, SystemsWork, Target, UsbDrive, UsbLink, Wipe,
    ChargeMode,
    ADDABLE,
    BACKUP_PARTS, DATA_ERASED, DATA_PARTITION, DOWNLOADS, FASTBOOT_CHOICES, SCREEN_OFF, TARGETS, THERMAL_LIMIT_LEAST, THERMAL_LIMIT_MOST, TIME_ZONES, UTC, USB_DRIVE_UNREADABLE,
    USB_DRIVE_WANTED, WIPES, installs, listed, starting,
};
use crate::app::{is_newer, KernelSuEnd, KernelSuJob};
use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::editor;
use crate::files;
use crate::fmt::{self, Verb};
use crate::font::Fonts;
use crate::geom::Rect;
use crate::lang::{Lang, Text};
use crate::search;
use crate::term::{self, Term};
use crate::theme::Theme;
use crate::tr;
use crate::icons::{self, Icon};
use crate::widgets::{self, Caption, Detail, Fact, Outline, Row, Spec, SystemCard, SystemMark, Tone, Trail};
use skyward_info::{DeviceInfo, RUNNING};
use skyward_install::root::{Rooted, Story};
use skyward_install::{Outcome, Reflash, Refusal, Stage, Unflashed};
use skyward_net::download::Progress;
use skyward_net::supplicant::Security;
use skyward_net::wlan::{Bands, Network, Power, Stage as Joining};
use skyward_systems::{Held, Who};
use skyward_vab::{Slot, Standing};
use std::path::Path;

// The colours the stock Settings fills its icons' tiles with, the same in
// light and dark mode.
const BLUE: Rgba = Rgba::rgb(0x34, 0x82, 0xff);
const GREEN: Rgba = Rgba::rgb(0x1d, 0xcd, 0x3a);
const PURPLE: Rgba = Rgba::rgb(0x77, 0x67, 0xf9);
const SLATE: Rgba = Rgba::rgb(0x8c, 0x9d, 0xb0);
const ORANGE: Rgba = Rgba::rgb(0xff, 0x69, 0x00);
const RED: Rgba = Rgba::rgb(0xfa, 0x38, 0x2e);
/// 解密页进度条前端那个点：白，落在蓝条上最清楚。
const WHITE: Rgba = Rgba::rgb(0xff, 0xff, 0xff);
/// 解密页进度条占整行内容宽的比例：这段等待短，条不铺满（2026-11-25）。
const BAR_SHARE: f32 = 0.62;

/// What every page draws from: style, typefaces, the device readings, and
/// what the pages hold, from the choices made on them to where the data
/// partition stands.
pub struct Ctx<'a> {
    pub t: &'a Theme,
    pub f: &'a Fonts,
    pub info: &'a DeviceInfo,
    pub state: &'a State,
    /// The shell the terminal page shows, once one has started.
    pub term: Option<&'a Term>,
    /// The part of the page to draw; all of it if none is given. The page
    /// is laid out whole either way.
    pub band: Option<Band>,
}

/// Rows `rows` of a page, and of them the columns `cols`, each from the
/// first to one past the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    pub rows: (u32, u32),
    pub cols: (u32, u32),
}

pub fn render(screen: Screen, cx: &Ctx) -> Frame {
    match screen {
        Screen::Decrypt => decrypt(cx),
        Screen::Root => root(cx),
        Screen::KernelSu => kernel_su(cx),
        Screen::KernelSuKmi => kernel_su_kmi(cx),
        Screen::Magisk => magisk(cx),
        Screen::Home => home(cx),
        Screen::Settings => settings(cx),
        Screen::Reboot => reboot(cx),
        Screen::Wipe => wipe(cx),
        Screen::Mount => mount(cx),
        Screen::Install => install(cx),
        Screen::Choose(place) => choose(cx, place),
        Screen::Installing => installing(cx),
        Screen::About => about(cx),
        Screen::Specs => specs(cx),
        Screen::ScreenOff => screen_off(cx),
        Screen::Search => search(cx),
        Screen::Backup => backup(cx),
        Screen::Restore => restore(cx),
        Screen::RestoreBackup => restore_backup(cx),
        Screen::BackupJob => backup_job(cx),
        Screen::Advanced => advanced(cx),
        Screen::Modules => modules(cx),
        Screen::DataApps => data_apps(cx),
        Screen::SystemApps => system_apps(cx),
        Screen::StoppedApps => stopped_apps(cx),
        Screen::DataAppInfo => data_app_info(cx),
        Screen::ModuleInfo => module_info(cx),
        Screen::Files => files(cx),
        Screen::FileInfo => file_info(cx),
        Screen::Permissions => permissions(cx),
        Screen::Editor => editor(cx),
        Screen::Terminal => terminal(cx),
        Screen::Relabel => relabel(cx),
        Screen::Filesystems => filesystems(cx),
        Screen::Filesystem => filesystem(cx),
        Screen::PartWork => part_work(cx),
        Screen::Language => language(cx),
        Screen::TimeZone => time_zone(cx),
        Screen::TimeZones(region) => time_zones(cx, region),
        Screen::ImageBackup => image_backup(cx),
        Screen::Logs => logs(cx),
        Screen::Update => update(cx),
        Screen::Serve => serve(cx),
        Screen::Fetch => fetch(cx),
        Screen::Charge => charge(cx),
        Screen::Avb => avb(cx),
        Screen::SkywardUpdate => skyward_update(cx),
        Screen::License => license(cx),
        Screen::FontLicense => font_license(cx),
        Screen::Slot => slot(cx),
        Screen::Systems => systems(cx),
        Screen::Fastboot => fastboot(cx),
        Screen::Erase => erase(cx),
        Screen::CantStart => cant_start(cx),
        Screen::Unhandled => unhandled(cx),
        Screen::Rename => rename(cx),
        Screen::Clash => clash(cx),
        Screen::FileJob => file_job(cx),
        Screen::Wlan => wlan(cx),
        Screen::WlanPassword => wlan_password(cx),
        Screen::WlanNetwork => wlan_network(cx),
        Screen::FilesOtg => files_otg(cx),
        Screen::WlanAdd => wlan_add(cx),
        Screen::Tasks => tasks(cx),
        Screen::Partitions => partitions(cx),
        Screen::Shutdown => shutdown(cx),
        Screen::Record => record(cx),
    }
}

/// 录屏：这一段写在哪、录了多久，下面一条开始或停止。
pub fn record(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // 这一页比一屏长一点：底下那句说明以前被屏幕边缘切掉半行，现在按内容裁、
    // 能滚到底（2026-11-21）。
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["这一段", "這一段", "This take"]));
    let state = match (s.record.on, s.record.paused) {
        (true, true) => l.pick(["已暂停", "已暫停", "Paused"]),
        (true, false) => l.pick(["正在录制", "正在錄影", "Recording"]),
        _ => l.pick(["没有在录", "沒有在錄", "Not recording"]),
    };
    let length = if s.record.on { s.record.elapsed.clone() } else { "—".to_string() };
    let file = if s.record.file.is_empty() { l.pick(["还没有录过", "還沒有錄過", "None yet"]).to_string() } else { s.record.file.clone() };
    let spec = if s.record.spec.is_empty() { "—".to_string() } else { s.record.spec.clone() };
    let rows = [
        (l.pick(["状态", "狀態", "State"]), state.to_string()),
        (l.pick(["已录", "已錄", "Length"]), length),
        (l.pick(["文件", "檔案", "File"]), file),
        (l.pick(["规格", "規格", "Specs"]), spec),
    ];
    let y = widgets::info_card(&mut c, t, f, y, &rows);

    // 录成什么样：三条各点一下换一档。正在录的时候不让改 —— 画面大小一变，
    // 编码线程与已经写出去的文件头都得重来。容器只有 MP4，没得选。
    let y = widgets::section(&mut c, t, f, y, l.pick(["录成什么样", "錄成什麼樣", "How it records"]));
    let panel = (t.m.w, t.m.h);
    let scale_names = [l.pick(["原画", "原畫", "Full"]), l.pick(["高清", "高清", "Sharp"]), l.pick(["标准", "標準", "Std"]), l.pick(["流畅", "流暢", "Smooth"])];
    let quality_names = [l.pick(["高", "高", "High"]), l.pick(["中", "中", "Mid"]), l.pick(["低", "低", "Low"])];
    let sizes: Vec<String> = (0..crate::record::SCALES.len())
        .map(|k| {
            let z = crate::record::Settings::of(panel, k, s.record.quality, s.record.fps);
            format!("{}×{}", z.width, z.height)
        })
        .collect();
    let chosen = crate::record::Settings::of(panel, s.record.scale, s.record.quality, s.record.fps);
    let per_minute = chosen.per_frame() as f64 * chosen.fps as f64 * 60.0 / 1e6;
    let values = [
        format!("{} {}", scale_names[s.record.scale.min(3)], sizes[s.record.scale.min(3)]),
        quality_names[s.record.quality.min(2)].to_string(),
        format!("{:.0} fps", crate::record::RATES[s.record.fps.min(3)]),
    ];
    let sized = format!("{}×{} · 每分钟约 {per_minute:.0} MB", chosen.width, chosen.height);
    let locked = l.pick(["正在录制，停下这一段才能改", "正在錄影，停下這一段才能改", "Recording: stop this take to change it"]);
    let rows = [
        Row::new(l.pick(["分辨率", "解析度", "Size"]))
            .icon(BLUE, Icon::Record)
            .subtitle(if s.record.on { locked } else { sized.as_str() })
            .trail(Trail::Value(values[0].as_str())),
        Row::new(l.pick(["画质", "畫質", "Quality"]))
            .icon(ORANGE, Icon::Record)
            .subtitle(if s.record.on { locked } else { l.pick(["画质越高越清楚，文件也越大", "畫質越高越清楚，檔案也越大", "Higher quality is sharper and bigger"]) })
            .trail(Trail::Value(values[1].as_str())),
        Row::new(l.pick(["帧率", "幀率", "Frame rate"]))
            .icon(GREEN, Icon::Record)
            .subtitle(if s.record.on { locked } else { l.pick(["画面不动也照样出帧；编码跟不上就少几帧，时长不变", "畫面不動也照樣出幀；編碼跟不上就少幾幀，時長不變", "Held at this rate even when nothing moves; if the encoder falls behind, fewer frames but the same length"]) })
            .trail(Trail::Value(values[2].as_str())),
        Row::new(l.pick(["容器", "容器", "Container"]))
            .icon(PURPLE, Icon::Record)
            .subtitle(l.pick([
                "只有 MP4：里面是 H.264，相册、ffmpeg、VLC 都放得开",
                "只有 MP4：裡面是 H.264，相簿、ffmpeg、VLC 都放得開",
                "MP4 only, carrying H.264; galleries, ffmpeg and VLC all open it",
            ]))
            .trail(Trail::Value(crate::record::Container::ONLY.short())),
    ];
    let list = widgets::list(&mut c, t, f, y, &rows);
    let mut out = hits(&list.rows, &[
        Action::RecordSetting(RecordSetting::Scale),
        Action::RecordSetting(RecordSetting::Quality),
        Action::RecordSetting(RecordSetting::Fps),
    ]);

    let doing = if s.record.on { l.pick(["停止录制", "停止錄影", "Stop recording"]) } else { l.pick(["开始录制", "開始錄影", "Start recording"]) };
    let rows = [Row::new(doing).icon(RED, Icon::Record).trail(Trail::None)];
    let list = widgets::list(&mut c, t, f, list.bottom + t.m.dp(10.0), &rows);
    out.extend(hits(&list.rows, &[Action::Record]));
    let bottom = widgets::footnote(&mut c, t, f, list.bottom, l.pick([
        "录好的文件在内部存储的 DCIM/Screenrecords，边录边写：断电最多丢最后一秒。容器只有 MP4，里面是 H.264 Baseline —— 相册、ffmpeg、VLC 都直接能放，拖动也不糊。以前装的是 MJPEG，相册一律说文件损坏，因为系统把 mp4v 当 MPEG-4 去解，解不出来。MP4 的索引写在文件末尾，录制中途断电那份打不开。录的时候那枚胶囊可以拖到任意位置：点左边的红点是暂停或继续，点右边的方块是结束这一段。",
        "錄好的檔案在內部儲存的 DCIM/Screenrecords，邊錄邊寫：斷電最多丟最後一秒。容器只有 MP4，裡面是 H.264 Baseline —— 相簿、ffmpeg、VLC 都直接能放，拖動也不糊。以前裝的是 MJPEG，相簿一律說檔案損壞，因為系統把 mp4v 當 MPEG-4 去解，解不出來。MP4 的索引寫在檔案末尾，錄影中途斷電那份打不開。錄的時候那枚膠囊可以拖到任意位置：點左邊的紅點是暫停或繼續，點右邊的方塊是結束這一段。",
        "Takes go to DCIM/Screenrecords on internal storage and are written as they are recorded: a power cut loses the last second at most. MP4 is the only container and it carries H.264 Baseline, which galleries, ffmpeg and VLC all decode, and which seeks cleanly. Earlier builds carried MJPEG, which every gallery called a broken file: the system maps the mp4v sample entry to MPEG-4 and the decoder chokes on the first JPEG. An MP4 keeps its index at the end, so a take cut off by a power loss will not open. Drag the pill anywhere while it runs; tap its dot to pause or carry on, tap its square to end the take.",
    ]));
    Frame::new(t, fit(c, t, bottom), out, l.pick(["录屏", "錄影", "Screen recording"]), top)
}

/// Label and colour of each swipe-to-confirm control: the filesystem
/// page's as `s` says what it is to do.
pub fn confirm_label(which: Confirm, s: &State) -> (&'static str, bool) {
    const INSTALL: Text = ["滑动以安装", "滑動以安裝", "Slide to install"];
    let (label, danger) = match which {
        Confirm::Avb => (["滑动以写入", "滑動以寫入", "Slide to write"], true),
        Confirm::KernelSu => (["滑动以写入 KernelSU", "滑動以寫入 KernelSU", "Slide to write KernelSU"], true),
        Confirm::Magisk => (["滑动以写入 Magisk", "滑動以寫入 Magisk", "Slide to write Magisk"], true),
        Confirm::Vbmeta => (["滑动以写回", "滑動以寫回", "Slide to put back"], true),
        Confirm::Filesystem => match s.fs_action {
            FsAction::Repair => (["滑动以修复", "滑動以修復", "Slide to repair"], true),
            FsAction::Remake(_) => (["滑动以更换", "滑動以更換", "Slide to change"], true),
        },
        Confirm::Reboot => (["滑动以重启", "滑動以重新啟動", "Slide to reboot"], false),
        Confirm::Shutdown => (["滑动以关机", "滑動以關機", "Slide to power off"], true),
        Confirm::Wipe => (["滑动以清除", "滑動以清除", "Slide to wipe"], true),
        Confirm::Flash => (["滑动以刷入", "滑動以刷入", "Slide to flash"], true),
        Confirm::Backup => (["滑动以备份", "滑動以備份", "Slide to back up"], false),
        Confirm::Restore => (["滑动以恢复", "滑動以還原", "Slide to restore"], true),
        Confirm::Slot => (["滑动以切换", "滑動以切換", "Slide to switch"], true),
        Confirm::FactoryReset => (["滑动以恢复出厂设置", "滑動以恢復原廠設定", "Slide to factory reset"], true),
        Confirm::DeleteEntry | Confirm::DeleteBackup | Confirm::ImageDelete => (["滑动以删除", "滑動以刪除", "Slide to delete"], true),
        Confirm::ImageRestore => (["滑动以写回", "滑動以寫回", "Slide to write back"], true),
        Confirm::DeleteApp => (["滑动以卸载应用", "滑動以卸載應用程式", "Slide to uninstall"], true),
        Confirm::DeleteModule => (["滑动以卸载模块", "滑動以卸載模組", "Slide to uninstall"], true),
        Confirm::KeepLock => (["滑动以开启", "滑動以開啟", "Slide to turn on"], true),
        Confirm::Install => (INSTALL, false),
        Confirm::InstallWipe => (["滑动以安装并清除", "滑動以安裝並清除", "Slide to install and wipe"], true),
        Confirm::RunScript => (INSTALL, true),
        Confirm::Merge => (["滑动以合并", "滑動以合併", "Slide to merge"], true),
        Confirm::GiveUp => (["滑动以放弃这次更新", "滑動以放棄這次更新", "Slide to give the update up"], true),
        Confirm::Relabel => (["滑动以修复", "滑動以修復", "Slide to fix"], true),
        Confirm::SwitchSystem => (["滑动以切换并重启", "滑動以切換並重新啟動", "Slide to switch and reboot"], false),
        Confirm::RemoveSystem => (["滑动以删除", "滑動以刪除", "Slide to remove"], true),
        Confirm::InstallSystem => (INSTALL, false),
    };
    (s.lang.pick(label), danger)
}

/// The name the build goes by. The same source is built twice, once for the
/// phone's owner and once as the plain recovery: what the build was told to
/// call itself, or the name it carries otherwise.
const fn brand(said: Option<&'static str>, otherwise: &'static str) -> &'static str {
    match said {
        Some(name) => name,
        None => otherwise,
    }
}

/// The home page's large title, and what its search bar and the search
/// page's field say while nothing is typed.
pub const HOME_TITLE: &str = brand(option_env!("SKYWARD_BRAND_TITLE"), "卑鄙坑Baby");
pub const SEARCH_HINT: Text = ["搜索功能与分区", "搜尋功能與分割區", "Search features and partitions"];

const REBOOT_TARGETS: [(Text, Option<Text>); 6] = [
    (["系统", "系統", "System"], None),
    (["恢复模式", "復原模式", "Recovery"], None),
    (["Fastboot"; 3], Some(["用户空间 fastbootd", "使用者空間 fastbootd", "Userspace fastbootd"])),
    (["Bootloader"; 3], None),
    (["EDL"; 3], Some(["高通紧急下载模式", "高通緊急下載模式", "Qualcomm emergency download"])),
    (["关机", "關機", "Power off"], None),
];

/// What pages are called both on the rows that open them and in their own
/// titles.
const WLAN: Text = ["无线网络", "無線網路", "WLAN"];
/// Whether the WLAN opens as the recovery starts.
const WLAN_AUTO: Text = ["WiFi 自动连接", "WiFi 自動連線", "WLAN on at start"];
/// Whether the password of the network joined last is kept with it.
const WLAN_KEEP: Text = ["记住 WiFi 密码", "記住 WiFi 密碼", "Keep the password"];
const DECRYPT: Text = ["解密", "解密", "Decrypt"];
const CHARGE: Text = ["充电", "充電", "Charging"];
const ADVANCED: Text = ["高级工具", "進階工具", "Advanced"];
const SETTINGS: Text = ["选项设置", "選項設定", "Options"];
const REBOOT: Text = ["重启系统", "重新啟動", "Reboot"];
const ABOUT: Text = ["关于本机", "關於手機", "About phone"];
const SPECS: Text = ["全部参数与信息", "全部規格與資訊", "All specs"];
const LOGS: Text = ["导出日志", "匯出記錄", "Export logs"];
const SCREEN_TIMEOUT: Text = ["息屏时间", "螢幕自動關閉", "Screen timeout"];
const UPDATE: Text = ["系统更新", "系統更新", "System update"];
const SKYWARD_UPDATE: Text = ["版本更新", "版本更新", "Version update"];
const FILES: Text = ["文件管理", "檔案管理", "File manager"];
const TERMINAL: Text = ["终端", "終端機", "Terminal"];
const LANGUAGE: Text = ["语言", "語言", "Language"];
const TIME_ZONE: Text = ["时区", "時區", "Time zone"];
const IMAGE_BACKUP: Text = ["分区镜像", "分割區映像檔", "Partition images"];
const LICENSE: Text = ["开源许可", "開放原始碼授權", "Open source licenses"];
const FONTS: Text = ["界面字体", "介面字型", "UI fonts"];
const DUAL_BOOT: Text = ["第二系统", "第二系統", "Dual boot"];

/// A canvas for a page `h` tall in the colour `bg`, holding the rows `cx`
/// asks to be drawn.
fn sheet(cx: &Ctx, h: u32, bg: Rgba) -> Canvas {
    let w = cx.t.m.w;
    let Band { rows, cols } = cx.band.unwrap_or(Band { rows: (0, h), cols: (0, w) });
    Canvas::band(w, h, rows.0..rows.1, cols.0..cols.1, bg)
}

fn page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h, cx.t.pal.bg)
}

/// A canvas tall enough for a scrolling page; [`fit`] trims it to the content.
fn long_page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h * 3, cx.t.pal.bg)
}

/// 一页画布，高到装得下 `rows` 行内容：默认三屏，行多了按行高估一个上界（标题、
/// 节标题、脚注与卡片间隔算作六行）。上限六屏 —— 画布是整屏的像素，再长就不值
/// 得了。`fit` 之后仍按真画到的位置裁短，所以估多了并不浪费。
///
/// 2026-11-25 用户报的「下拉到最后」：无线网络页把整个扫描结果一次画进画布，
/// 三屏装不下时列表尾、卡片底圆角与「添加网络」就永远滚不到。
fn long_page_rows(cx: &Ctx, rows: usize) -> Canvas {
    let m = &cx.t.m;
    let wanted = m.content_top + (rows as f32 + 6.0) * m.row_h + m.margin * 2.0;
    let (least, most) = ((m.h * 3) as f32, (m.h * 6) as f32);
    sheet(cx, wanted.max(least).min(most) as u32, cx.t.pal.bg)
}

/// How tall a long page whose content ends at `bottom` is: to the bottom
/// margin after it, and never shorter than one screen.
fn fitted(t: &Theme, bottom: f32) -> u32 {
    ((bottom + t.m.margin * 2.0).ceil() as u32).max(t.m.h)
}

/// Trim a long page to its content plus the bottom margin, and never shorter
/// than one screen.
fn fit(c: Canvas, t: &Theme, bottom: f32) -> Canvas {
    c.crop_height(fitted(t, bottom))
}

fn hits(rects: &[Rect], actions: &[Action]) -> Vec<Hit> {
    rects.iter().zip(actions).map(|(&rect, &action)| Hit { rect, action }).collect()
}

/// Recovery home, laid out like the Settings root: title, search, a device
/// card, then grouped entries. The status bar carries the readings: the
/// slot, the data partition's lock and the battery's temperature.
pub fn home(cx: &Ctx) -> Frame {
    let (t, f, i, s, l) = (cx.t, cx.f, cx.info, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // No search bar: the page starts with the phone itself.
    let mut y = t.m.content_top;

    // Under the phone's name the card holds only what asks for something to
    // be done: a battery too low to install on, an update waiting to merge,
    // and a restart into the slot the system does not run from now.
    let low = match i.battery.as_ref().and_then(skyward_install::battery_refusal) {
        Some(Refusal::Battery { level, charging }) => Some(fmt::battery_low(l, level, charging)),
        _ => None,
    };
    let next = match (i.slot, &cx.state.boot_slots) {
        (Some(now), Some(Ok(slots))) if slots.active != now => Some(fmt::next_boot(l, slots.active, now)),
        _ => None,
    };
    let mut rows = Vec::new();
    let mut opens = Vec::new();
    if let Some((title, note)) = &low {
        rows.push(Row::new(title).icon(RED, Icon::Battery).subtitle(note).trail(Trail::None));
        opens.push(None);
    }
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Update)));
    }
    if let Some((title, note)) = &next {
        rows.push(Row::new(title).icon(ORANGE, Icon::Power).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Slot)));
    }
    if cx.state.system_notice.is_some() {
        let why = l.pick(["点按查看原因", "點按查看原因", "Tap to see why"]);
        rows.push(Row::new(l.pick(SECOND_FAILED)).icon(RED, Icon::Systems).subtitle(why).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Systems)));
    }
    // The account's row is the phone's own: the owner's picture and the name
    // the phone was given, with the model and the system under it, and the page
    // of details a tap away. It waits for the partition it is kept on to be
    // open; before that neither the picture nor the name is there.
    // The model and the codename only: the version is on the page the row opens,
    // and inside the row it runs past the end.
    let detail = format!("{} \u{b7} {}", i.model, i.codename);
    // Under the name, what the phone answers at a glance: the slot it runs
    // from, whether the data partition is open, the room left on it, and the
    // charge it is taking. Only the ones it has answered for stand, so the band
    // never shows a figure that is not there.
    let mut readouts: Vec<widgets::Readout> = Vec::new();
    if let Some(slot) = i.slot {
        readouts.push(widgets::Readout::new(slot.letter(), l.pick(SLOT_NOW)));
    }
    readouts.push(widgets::Readout::new(fmt::data(l, &s.data), l.pick(DATA_FIGURE)));
    if let Some(free) = i.storage.free {
        readouts.push(widgets::Readout::new(fmt::gb(free), l.pick(FREE_SPACE)));
    }
    // 电池那一格：电量在上，下排写它此刻在做什么。插着线的时候功率跟在状态后面
    // —— `充电中 · 7.9W`，与充电页「现在」那个数字同源；充到 100% 也照样给，
    // 满电还在补的那点功率一样看得见。拔了线才只剩「电池电量」。
    if let Some(b) = &i.battery {
        let label = if b.charging() {
            let tenths = b.watts().map(|w| (w * 10.0).round() as u32).filter(|t| *t > 0);
            match tenths {
                Some(t) => format!("{} · {}", l.pick(CHARGING_NOW), fmt::watts(t)),
                None => l.pick(CHARGING_NOW).to_owned(),
            }
        } else {
            l.pick(BATTERY_FIGURE).to_owned()
        };
        readouts.push(widgets::Readout::new(format!("{}%", b.level), label));
    }
    match &s.device_name {
        Some(name) => {
            let account = widgets::account_card(&mut c, t, f, y, name, Some(&detail), s.avatar.as_ref(), &readouts);
            // Every rect the card gave back answers: the name's row and the
            // band of readings under it are one control, so wherever on the
            // box a finger lands, the details open.
            out.extend(account.rows.iter().map(|r| Hit { rect: *r, action: Action::Open(Screen::About) }));
            y = account.bottom + t.m.card_gap;
        }
        // Nothing of the phone itself read yet: the model alone stands in, and
        // a tap still goes to the details.
        None => {
            let card = widgets::account_card(&mut c, t, f, y, &i.model, Some(&i.codename), None, &readouts);
            out.extend(card.rows.iter().map(|r| Hit { rect: *r, action: Action::Open(Screen::About) }));
            y = card.bottom + t.m.card_gap;
        }
    }
    // What asks for something to be done stands on its own under it: a battery
    // too low to install on, an update waiting to merge, a restart into the
    // slot the system does not run from now.
    if !rows.is_empty() {
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(list.rows.iter().zip(&opens).filter_map(|(&rect, open)| open.map(|action| Hit { rect, action })));
        y = list.bottom + t.m.card_gap;
    }
    // On a phone whose WLAN the recovery brings up, the WLAN comes first
    // under the device, as on the stock settings home; with it, where the
    // phone can hold a second system, the two systems.
    // The network the phone is joined to and the file manager come first.
    let mut first_rows = Vec::new();
    let mut first_opens = Vec::new();
    if let Some(w) = &cx.state.wlan {
        first_rows.push(Row::new(l.pick(WLAN)).icon(BLUE, Icon::Wlan).trail(Trail::Value(fmt::wlan_value(l, w))));
        first_opens.push(Action::Open(Screen::Wlan));
    }
    first_rows.push(Row::new(l.pick(FILES)).icon(BLUE, Icon::Folder));
    first_opens.push(Action::Open(Screen::Files));
    // 模块管理紧挨着文件管理：装模块就是往 data 里放文件，放在这里顺手。
    first_rows.push(Row::new(l.pick(MODULES)).icon(BLUE, Icon::Layers));
    first_opens.push(Action::Open(Screen::Modules));
    first_rows.push(Row::new(l.pick(DATA_APPS)).icon(PURPLE, Icon::Layers));
    first_opens.push(Action::Open(Screen::DataApps));
    let first = widgets::list(&mut c, t, f, y, &first_rows);
    out.extend(hits(&first.rows, &first_opens));
    y = first.bottom + t.m.card_gap;
    // Then the three rows that put something on the phone or take it off.
    let tools = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["安装刷机", "安裝刷機", "Install"])).icon(GREEN, Icon::Install),
        Row::new(l.pick(["创建备份", "建立備份", "Backup"])).icon(BLUE, Icon::Backup),
        Row::new(l.pick(["恢复备份", "還原備份", "Restore"])).icon(PURPLE, Icon::Restore),
    ]);
    out.extend(hits(&tools.rows, &[
        Action::Open(Screen::Install),
        Action::Open(Screen::Backup),
        Action::Open(Screen::Restore),
    ]));
    y = tools.bottom + t.m.card_gap;
    // Where the phone holds a second system, it comes first here.
    let mut system_rows = Vec::new();
    let mut system_opens = Vec::new();
    let starts = systems_value(cx);
    if cx.state.offers(Screen::Systems) {
        let row = Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems);
        system_rows.push(match &starts {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        system_opens.push(Action::Open(Screen::Systems));
    }
    // 充电不在这里：它挪进了高级工具，排在录屏下面。
    system_rows.push(Row::new(l.pick(ADVANCED)).icon(SLATE, Icon::More));
    system_opens.push(Action::Open(Screen::Advanced));
    system_rows.push(Row::new(l.pick(SETTINGS)).icon(SLATE, Icon::Gear));
    system_opens.push(Action::Open(Screen::Settings));
    let system = widgets::list(&mut c, t, f, y, &system_rows);
    out.extend(hits(&system.rows, &system_opens));
    // At the foot: rebooting, wiping and this recovery's own update.
    let mut footer_rows = Vec::new();
    let mut footer_opens = Vec::new();
    let target = l.pick(REBOOT_TARGETS[cx.state.reboot].0);
    footer_rows.push(Row::new(l.pick(REBOOT)).icon(ORANGE, Icon::Power).trail(Trail::Value(target)));
    footer_opens.push(Action::Open(Screen::Reboot));
    footer_rows.push(Row::new(l.pick(["清除数据", "清除資料", "Wipe data"])).icon(RED, Icon::Wipe));
    footer_opens.push(Action::Open(Screen::Wipe));
    if cx.state.offers(Screen::SkywardUpdate) {
        let row = Row::new(l.pick(SKYWARD_UPDATE)).icon(BLUE, Icon::Install);
        footer_rows.push(match skyward_value(cx) {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        footer_opens.push(Action::Open(Screen::SkywardUpdate));
    }
    let footer = widgets::list(&mut c, t, f, system.bottom + t.m.card_gap, &footer_rows);
    out.extend(hits(&footer.rows, &footer_opens));
    // However little it holds, the page scrolls far enough to fold its bar
    // away.
    let height = fitted(t, footer.bottom).max(t.m.h);
    Frame { back: false, ..Frame::new(t, c.crop_height(height), out, HOME_TITLE, t.m.content_top) }
}

/// What the home page says beside its entry to the systems: the one that
/// starts, where a second system is installed, once they are read.
fn systems_value(cx: &Ctx) -> Option<String> {
    let l = cx.state.lang;
    match &cx.state.systems {
        Some(Ok(systems)) => Some(match (&systems.other, starting(systems)) {
            (None, _) => l.pick(["未安装第二系统", "未安裝第二系統", "No second system"]).into(),
            (Some(_), Who::Installed) => systems.installed.name.clone(),
            (Some(other), Who::Other) => other.name.clone(),
        }),
        _ => None,
    }
}

/// What the advanced page says beside its system update entry: a download
/// under way, a newer system found, or where an update the system installed
/// stands.
fn update_value(cx: &Ctx) -> &'static str {
    let (s, l) = (cx.state, cx.state.lang);
    if s.downloading() {
        return l.pick(DOWNLOADING);
    }
    if s.newest().is_some_and(|r| cx.info.os_version.as_deref() != Some(r.version.as_str())) {
        return l.pick(["有新版本", "有新版本", "New version"]);
    }
    fmt::update_value(l, &s.update)
}

/// Preferences, opening with the stock「显示与亮度」theme previews.
pub fn settings(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let themes = widgets::theme_picker(&mut c, t, f, l, top + t.m.dp(8.9));
    out.extend(hits(&themes.rows, &[Action::Theme { dark: false }, Action::Theme { dark: true }]));
    let mut y = widgets::section(&mut c, t, f, themes.bottom, l.pick(["亮度", "亮度", "Brightness"]));
    let off = fmt::screen_off(l, SCREEN_OFF[s.screen_off]);
    let below = Row::new(l.pick(SCREEN_TIMEOUT)).trail(Trail::Value(&off));
    let slider = widgets::slider_card(&mut c, t, f, y, Caption::Sun, l.pick(["屏幕亮度", "螢幕亮度", "Screen brightness"]), s.brightness, Some(below));
    out.push(Hit { rect: slider.track, action: Action::Slider(Slider::Brightness) });
    if let Some(r) = slider.below {
        out.push(Hit { rect: r, action: Action::Open(Screen::ScreenOff) });
    }
    y = widgets::section(&mut c, t, f, slider.bottom, l.pick(["振动", "震動", "Vibration"]));
    let below = Row::new(l.pick(["触摸振动", "觸控震動", "Touch vibration"])).trail(Trail::Switch(s.vibrate));
    let vibration = widgets::slider_card(&mut c, t, f, y, Caption::Vibrate, l.pick(["振动强度", "震動強度", "Vibration intensity"]), s.vibration, Some(below));
    out.push(Hit { rect: vibration.track, action: Action::Slider(Slider::Vibration) });
    if let Some(r) = vibration.below {
        out.push(Hit { rect: r, action: Action::Toggle(Switch::Vibrate) });
    }
    y = widgets::section(&mut c, t, f, vibration.bottom, l.pick(["通用", "一般", "General"]));
    let general = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(LANGUAGE)).icon(BLUE, Icon::Globe).trail(Trail::Value(l.name())),
        Row::new(l.pick(TIME_ZONE)).icon(ORANGE, Icon::Clock).trail(Trail::Value(s.zone_name())),
    ]);
    out.extend(hits(&general.rows, &[Action::Open(Screen::Language), Action::Open(Screen::TimeZone)]));
    y = widgets::section(&mut c, t, f, general.bottom, "USB");
    let usb = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["文件传输", "檔案傳輸", "File transfer"])).icon(BLUE, Icon::Usb).trail(Trail::Switch(s.file_transfer))]);
    out.extend(hits(&usb.rows, &[Action::Toggle(Switch::FileTransfer)]));
    let note = l.pick([
        "开启后，可在电脑上管理内部存储中的文件，需先完成「解密」。",
        "開啟後，可在電腦上管理內部儲存空間中的檔案，需先完成「解密」。",
        "When on, a computer can manage the files in internal storage, once it is decrypted.",
    ]);
    let bottom = widgets::footnote(&mut c, t, f, usb.bottom, note);
    // 终端不在这里：高级页已经有一个，同一个东西开两个入口反而添乱
    //（2026-11-17）。关于与更新也一样，更新在首页底部单独一栏。
    Frame::new(t, fit(c, t, bottom), out, l.pick(SETTINGS), top)
}

/// What the settings page says beside its Skyward update entry: an update
/// under way, or a newer release found.
fn skyward_value(cx: &Ctx) -> Option<&'static str> {
    let (s, l) = (cx.state, cx.state.lang);
    if s.updating_skyward() {
        // 具体百分比在更新页里；首页这一行只说它还在进行，退出来也看得见。
        return Some(l.pick(["正在更新", "正在更新", "Updating"]));
    }
    match (s.skyward_newest(), &s.skyward_job) {
        (_, Some(SkywardJob { outcome: Some(SkywardEnd::Updated(_)), .. })) => Some(l.pick(["已更新", "已更新", "Updated"])),
        (Some(n), _) if n.newer && n.build.is_some() => Some(l.pick(["有新版本", "有新版本", "New version"])),
        _ => None,
    }
}

/// How long the screen stays on with nothing touched, opened from the
/// settings.
pub fn screen_off(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let labels = SCREEN_OFF.map(|ms| fmt::screen_off(l, ms));
    let rows: Vec<Row> = labels.iter().enumerate().map(|(k, label)| Row::new(label).trail(Trail::Check(s.screen_off == k))).collect();
    let times = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = (0..SCREEN_OFF.len()).map(Action::ScreenOff).collect();
    let note = l.pick([
        "超过所选时间未操作，屏幕将变暗并熄灭，按电源键即可点亮。",
        "超過所選時間未操作，螢幕將變暗並關閉，按電源鍵即可喚醒。",
        "After the time chosen with nothing touched, the screen dims and turns off. Press the power key to light it again.",
    ]);
    let y = widgets::footnote(&mut c, t, f, times.bottom, note);
    if SCREEN_OFF[s.screen_off].is_none() {
        let note = l.pick([
            "屏幕将保持常亮，长时间显示同一画面可能留下残影。",
            "螢幕將保持恆亮，長時間顯示同一畫面可能留下殘影。",
            "The screen stays on. Showing the same image for long may leave it burnt in.",
        ]);
        widgets::footnote(&mut c, t, f, y, note);
    }
    Frame::new(t, c, hits(&times.rows, &picks), l.pick(SCREEN_TIMEOUT), top)
}

/// Mount manager: the slot's partitions, each with a switch to mount it
/// read-only or unmount it; one that programs are using says which, and
/// stays.
pub fn mount(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;

    let y = widgets::section_first(&mut c, t, f, top, l.pick(["分区", "分割區", "Partitions"]));
    let bottom = match &s.mounts {
        Err(why) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["无法读取分区", "無法讀取分割區", "Couldn't read the partitions"])).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Ok(mounts) if mounts.is_empty() => {
            widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["暂无可挂载的分区", "沒有可掛載的分割區", "No partitions to mount"])).trail(Trail::None)]).bottom
        }
        Ok(mounts) => {
            let subs: Vec<String> = mounts
                .iter()
                .map(|m| {
                    let point = &m.point;
                    // 一个名字都不列。odm 上挂着的叫
                    // `android.hardware.gatekeeper-rust-service-qti`，点两个名字
                    // 照样折成三行，右边那枚「使用中」还飘在三行的中间
                    // （2026-11-21 真机）。数个数，一行放得下。
                    let n = m.holders.len();
                    match n {
                        0 => point.clone(),
                        1 => tr!(l, "{point} · 1 个正在使用", "{point} · 1 個正在使用", "{point} · in use by 1 service"),
                        _ => tr!(l, "{point} · {n} 个正在使用", "{point} · {n} 個正在使用", "{point} · in use by {n} services"),
                    }
                })
                .collect();
            let rows: Vec<Row> = mounts
                .iter()
                .zip(&subs)
                .map(|(m, sub)| {
                    let trail = if m.holders.is_empty() { Trail::Switch(m.mounted) } else { Trail::Text(l.pick(["使用中", "使用中", "In use"])) };
                    Row::new(&m.base).subtitle(sub).trail(trail)
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            out.extend(list.rows.iter().zip(mounts).enumerate().map(|(i, (&rect, m))| {
                let action = match m.holders.is_empty() {
                    true => Action::MountToggle(i),
                    false => Action::Unavailable(l.pick(["无法卸载正在使用的分区", "無法卸載正在使用的分割區", "A partition in use can't be unmounted"])),
                };
                Hit { rect, action }
            }));
            let y = match &s.mount_failure {
                Some(failure) => widgets::footnote(&mut c, t, f, list.bottom, &fmt::mount_failure(l, failure)),
                None => list.bottom,
            };
            let about = match cx.info.slot.map(|s| s.letter()) {
                Some(slot) => tr!(
                    l,
                    "仅显示当前槽位 {slot} 在 super 中的分区，挂载后均为只读。",
                    "僅顯示目前槽位 {slot} 在 super 中的分割區，掛載後均為唯讀。",
                    "Only the current slot {slot}'s partitions in super show here, all mounted read-only."
                ),
                None => l
                    .pick(["仅显示 super 中的分区，挂载后均为只读。", "僅顯示 super 中的分割區，掛載後均為唯讀。", "Only partitions in super show here, all mounted read-only."])
                    .into(),
            };
            widgets::footnote(&mut c, t, f, y, &about)
        }
    };

    Frame::new(t, fit(c, t, bottom), out, l.pick(["挂载", "掛載", "Mount"]), top)
}

/// Who made the recovery, as the About page names them under its version.
const AUTHOR: &str = brand(option_env!("SKYWARD_BRAND_AUTHOR"), "© 2026 卑鄙坑Baby");

/// The wordmark drawn over the About page's colour field.
const WORDMARK: &str = brand(option_env!("SKYWARD_BRAND_WORDMARK"), "叼着奶瓶逛酒吧");

/// About this device, modelled on the stock「我的设备」: the wordmark, the
/// version and its author over a moving colour field, and the summary,
/// hardware and link cards scrolling over it. Only the cards are drawn
/// here; the canvas is clear wherever the field shows.
pub fn about(cx: &Ctx) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let m = &t.m;
    // The page is drawn whole, under the field: laid out first to learn
    // how tall it comes to, then drawn that tall.
    let (_, _, bottom) = about_cards(cx, &mut Canvas::band(m.w, m.h * 3, 0..0, 0..0, Rgba { r: 0, g: 0, b: 0, a: 0 }));
    let mut c = Canvas::new(m.w, fitted(t, bottom));
    let (out, summary_top, bottom) = about_cards(cx, &mut c);
    let canvas = fit(c, t, bottom);
    let art = HeroArt {
        wordmark: widgets::wordmark(t, f, WORDMARK, m.dp(233.7), m.dp(37.8)),
        version: widgets::version_line(t, f, RUNNING),
        author: widgets::author_line(t, f, AUTHOR),
        runs: canvas.runs(),
    };
    Frame { header: false, hero: Some(art), ..Frame::new(t, canvas, out, cx.state.lang.pick(ABOUT), summary_top) }
}

const MODEL: Text = ["设备型号", "裝置型號", "Model"];
const PROCESSOR: Text = ["处理器", "處理器", "Processor"];
const RAM: Text = ["运行内存", "記憶體", "RAM"];
const BATTERY_CAPACITY: Text = ["电池容量", "電池容量", "Battery capacity"];
const SCREEN_SIZE: Text = ["屏幕尺寸", "螢幕尺寸", "Screen size"];
const RESOLUTION: Text = ["分辨率", "解析度", "Resolution"];
const OS_VERSION: Text = ["OS 版本", "OS 版本", "OS version"];
const ANDROID_VERSION: Text = ["Android 版本", "Android 版本", "Android version"];
const CURRENT_SLOT: Text = ["当前槽位", "目前槽位", "Current slot"];
const DATA_PART: Text = ["data 分区", "data 分割區", "data partition"];

/// 设备卡上那一排读数各自的说明：数值在上，这一行在下。
const SLOT_NOW: Text = ["当前槽位", "當前槽位", "Current slot"];
const DATA_FIGURE: Text = ["数据分区", "資料分區", "Data"];
const FREE_SPACE: Text = ["可用空间", "可用空間", "Free space"];
const CHARGING_NOW: Text = ["充电中", "充電中", "Charging"];
const BATTERY_FIGURE: Text = ["电池电量", "電池電量", "Battery"];

/// The phone's processor by its marketing name in `l`, from the phone's
/// settings, or else by its cores.
fn soc(l: Lang, i: &DeviceInfo) -> Option<&str> {
    i.soc.as_ref().map(|n| l.pick([n.hans.as_str(), n.hant.as_str(), n.en.as_str()]))
}

/// The About page's cards on `c`: what touching them does, where the
/// first starts, and where the last ends.
fn about_cards(cx: &Ctx, c: &mut Canvas) -> (Vec<Hit>, f32, f32) {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let m = &t.m;
    let tone = Tone::hero(t);
    let mut out = Vec::new();

    let storage = fmt::storage_used(&i.storage).unwrap_or_else(|| fmt::capacity(i.storage.total));
    let mut facts = vec![
        Fact { title: l.pick(MODEL), value: i.model.clone(), link: false },
        Fact { title: l.pick(["存储空间", "儲存空間", "Storage"]), value: storage, link: true },
    ];
    if let Some(v) = &i.os_version {
        facts.push(Fact { title: l.pick(OS_VERSION), value: v.clone(), link: false });
    }
    if let Some(r) = &i.android {
        facts.push(Fact { title: l.pick(ANDROID_VERSION), value: r.clone(), link: false });
    }
    let summary = widgets::facts(c, t, f, m.dp(490.7), &facts, &tone);
    out.push(Hit { rect: summary.rows[1], action: Action::Open(Screen::Specs) });

    let processor = soc(l, i).map_or_else(|| fmt::cpu(l, &i.cpu), str::to_string);
    let mut specs = vec![(l.pick(PROCESSOR), processor), (l.pick(RAM), fmt::ram(i.ram_kib))];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        specs.push((l.pick(BATTERY_CAPACITY), fmt::mah(design)));
    }
    if let Some(d) = i.screen {
        specs.push((l.pick(SCREEN_SIZE), fmt::inches(d)));
    }
    if let Some(p) = i.panel {
        specs.push((l.pick(RESOLUTION), fmt::panel(p)));
    }
    let y = widgets::spec_card(c, t, f, summary.bottom + m.card_gap, &fmt::model_lines(&i.model), &specs, &tone);
    let links = widgets::list_toned(c, t, f, y + m.card_gap, &[Row::new(l.pick(SPECS)), Row::new(l.pick(LOGS))], &tone);
    out.extend(hits(&links.rows, &[Action::Open(Screen::Specs), Action::Open(Screen::Logs)]));
    (out, summary.rows[0].y, links.bottom)
}

/// Every reading the recovery has, modelled on the stock「全部参数与信息」.
pub fn specs(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let gap = t.m.card_gap;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // This page leaves more room under the title than the settings pages.
    let top = t.m.dp(188.0);

    let mut grid = vec![Spec { icon: Outline::Phone, label: l.pick(MODEL), value: fmt::model_lines(&i.model) }];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        grid.push(Spec { icon: Outline::Battery, label: l.pick(BATTERY_CAPACITY), value: fmt::mah(design) });
    }
    let cpu = match soc(l, i) {
        Some(soc) => format!("{soc}\n{}", fmt::cpu(l, &i.cpu)),
        None => fmt::cpu(l, &i.cpu),
    };
    grid.push(Spec { icon: Outline::Chip, label: l.pick(PROCESSOR), value: cpu });
    grid.push(Spec { icon: Outline::Memory, label: l.pick(RAM), value: fmt::ram(i.ram_kib) });
    if let Some(p) = i.panel {
        grid.push(Spec { icon: Outline::Resolution, label: l.pick(RESOLUTION), value: fmt::panel(p) });
    }
    if let Some(d) = i.screen {
        grid.push(Spec { icon: Outline::Screen, label: l.pick(SCREEN_SIZE), value: fmt::inches(d) });
    }
    grid.push(Spec { icon: Outline::Storage, label: l.pick(["存储容量", "儲存容量", "Storage capacity"]), value: fmt::capacity(i.storage.total) });
    grid.push(Spec { icon: Outline::Tag, label: l.pick(["设备代号", "裝置代號", "Codename"]), value: i.codename.clone() });
    let mut y = widgets::spec_grid(&mut c, t, f, top, &grid);

    if let Some(v) = &i.os_version {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(OS_VERSION), v.clone())]);
    }
    if let Some(r) = &i.android {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(ANDROID_VERSION), fmt::android(l, r, i.security_patch.as_ref()))]);
    }
    y = widgets::info_card(&mut c, t, f, y + gap, &[
        (l.pick(["Recovery 版本", "Recovery 版本", "Recovery version"]), format!("{HOME_TITLE} {RUNNING}")),
        (l.pick(["内核版本", "核心版本", "Kernel version"]), i.kernel.clone()),
    ]);

    let mut boot = Vec::new();
    if let Some(s) = i.slot {
        boot.push((l.pick(CURRENT_SLOT), s.letter().to_string()));
        boot.push((l.pick(UPDATE), fmt::snapshot(l, &i.snapshot).to_string()));
    }
    if let Some(unlocked) = i.boot.unlocked {
        let state = if unlocked { ["已解锁", "已解鎖", "Unlocked"] } else { ["已锁定", "已鎖定", "Locked"] };
        boot.push(("Bootloader", l.pick(state).to_string()));
    }
    if let Some(v) = &i.boot.verified {
        boot.push((l.pick(["验证启动", "驗證開機", "Verified boot"]), fmt::verified(l, v)));
    }
    boot.push((l.pick(DATA_PART), fmt::data(l, &cx.state.data).to_string()));
    y = widgets::info_card(&mut c, t, f, y + gap, &boot);

    if let Some(b) = &i.battery {
        let mut rows = vec![(l.pick(["电量", "電量", "Level"]), fmt::level(l, b))];
        if let Some(v) = b.temp {
            rows.push((l.pick(["温度", "溫度", "Temperature"]), fmt::celsius(v)));
        }
        if let Some(v) = fmt::wear(l, b) {
            rows.push((l.pick(["健康", "健康狀態", "Health"]), v));
        }
        if let Some(v) = b.cycles {
            rows.push((l.pick(["充电循环", "充電循環", "Charge cycles"]), tr!(l, "{v} 次", "{v} 次", "{v}")));
        }
        if let Some(v) = b.voltage {
            rows.push((l.pick(["电压", "電壓", "Voltage"]), fmt::volts(v)));
        }
        y = widgets::info_card(&mut c, t, f, y + gap, &rows);
    }

    let total = fmt::capacity(i.storage.total);
    let storage = match i.storage.free.map(fmt::gb) {
        Some(free) => tr!(l, "可用存储 {free} / 总存储 {total}", "可用空間 {free} / 總容量 {total}", "{free} free of {total}"),
        None => tr!(l, "总存储 {total} · data 分区未挂载", "總容量 {total} · data 分割區未掛載", "{total} in all · data partition not mounted"),
    };
    y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(["机身存储", "內建儲存空間", "Internal storage"]), storage)]);
    let links = widgets::list(&mut c, t, f, y + gap, &[
        Row::new(l.pick(LICENSE)).trail(Trail::Value("GPL-3.0")),
        Row::new(l.pick(FONTS)).trail(Trail::Value("MiSans")),
    ]);
    out.extend(hits(&links.rows, &[Action::Open(Screen::License), Action::Open(Screen::FontLicense)]));
    Frame::new(t, fit(c, t, links.bottom), out, l.pick(SPECS), top)
}

/// The data partition and its users' own files. The page opens the
/// partition as it shows, which is how it learns each user's screen lock;
/// a user whose files wait on one is asked with the control that lock is
/// answered with, from the start.
pub fn decrypt(cx: &Ctx) -> Frame {
    let (c, l) = (page(cx), cx.state.lang);
    match &cx.state.data {
        Data::Locked { user, keys, checking } => code(cx, c, *user, *keys, *checking),
        // Without a lock kept, the page the start opens on is the one to type
        // it into: nothing turns while the phone's own parts come up, and the
        // keyboard takes nothing until the lock is known.
        Data::Closed | Data::Opening if !cx.state.remember => code(cx, c, 0, Keys::Digits, true),
        Data::Closed | Data::Opening => data_status(
            cx,
            c,
            Icon::Lock,
            l.pick(["解密 data 分区", "解密 data 分割區", "Decrypt the data partition"]),
            Some(if cx.state.prepared {
                l.pick(["正在读取锁屏方式", "正在讀取螢幕鎖定方式", "Reading the screen lock"])
            } else {
                l.pick(["正在准备分区与服务", "正在準備分割區與服務", "Bringing the phone's own parts up"])
            }),
            None,
            true,
        ),
        Data::Open => {
            let t = cx.t;
            let m = &t.m;
            let mut frame = data_status(
                cx,
                c,
                Icon::Folder,
                l.pick(["data 分区已解密", "data 分割區已解密", "The data partition is decrypted"]),
                Some(l.pick(["你可以在「文件管理」中查看本机文件", "你可以在「檔案管理」中查看本機檔案", "You can see the phone's files in File manager"])),
                None,
                false,
            );
            // The lock is shown here too, and dropped from here: once it is
            // kept the decrypt page opens straight into the menu, so this is
            // where it is come back to in order to stop keeping it.
            let y = m.dp(150.0) + m.dp(98.0) + m.dp(52.0);
            let (kept, not_kept) = (
                l.pick(["已记住密码", "已記住密碼", "Screen lock kept"]),
                l.pick(["未记住密码", "未記住密碼", "Screen lock not kept"]),
            );
            let row = widgets::list(
                &mut frame.canvas,
                t,
                cx.f,
                y,
                &[Row::new(match cx.state.remember {
                    true => kept,
                    false => not_kept,
                })
                .icon(if cx.state.remember { PURPLE } else { SLATE }, Icon::Lock)
                .trail(Trail::Switch(cx.state.remember))],
            );
            frame.hits.extend(hits(&row.rows, &[Action::Toggle(Switch::Remember)]));
            frame
        }
        Data::Failed(why) => data_status(cx, c, Icon::Lock, l.pick(["未能打开 data 分区", "無法開啟 data 分割區", "Couldn't open the data partition"]), None, Some(why), false),
        Data::Erased => data_status(
            cx,
            c,
            Icon::Wipe,
            l.pick(DATA_ERASED),
            Some(l.pick(["下次开机时将重新创建", "下次開機時將重新建立", "It's made anew when the phone next starts"])),
            None,
            false,
        ),
    }
}

/// Where the partition stands, under its sign: a line on it, or why it
/// did not open and a way to try again; then the way off the page.
fn data_status(cx: &Ctx, mut c: Canvas, icon: Icon, title: &str, note: Option<&str>, why: Option<&str>, busy: bool) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let sign_cy = m.dp(150.0);
    c.fill_circle(x, sign_cy, m.dp(30.0), t.pal.card);
    icons::glyph(&mut c, icon, x, sign_cy, m.dp(40.0), t.pal.accent);
    // A ring about the sign that turns while the partition is being opened --
    // or while the page waits for the phone's own parts, which the opening
    // needs -- so that a page that is waiting has something moving on it.
    if busy {
        let r = m.dp(48.0);
        let start = -std::f32::consts::FRAC_PI_2 + cx.state.data_phase * std::f32::consts::TAU;
        c.stroke_circle(x, sign_cy, r, m.dp(1.5), t.pal.card);
        for k in 0..4 {
            let a = 0.5 + 0.5 * (k as f32 / 4.0);
            let color = crate::color::mix(t.pal.bg, t.pal.accent, a * 0.55);
            c.stroke_arc(x, sign_cy, r, start - 0.14 * (4 - k) as f32, 0.15, m.dp(2.5), color);
        }
        c.stroke_arc(x, sign_cy, r, start, 0.9, m.dp(2.5), t.pal.accent);
    }
    c.text_center(&f.heading, title, x, sign_cy + m.dp(64.0), m.dp(24.0), t.pal.text);
    if let Some(note) = note {
        // 两句说明也可能长过屏幕：一行放不下就缩字号，别左右顶出去。
        let room = m.w as f32 - m.margin * 2.0;
        let tw = c.text_width(&f.body, note, m.sub_px);
        let px = if tw > room && tw > 0.0 { (m.sub_px * room / tw).max(m.dp(9.0)) } else { m.sub_px };
        c.text_center(&f.body, note, x, sign_cy + m.dp(118.0), px, t.pal.subtitle);
    }
    // A bar under the note, drawn from the page's own sweep: it runs one
    // length from the moment the page starts waiting to the moment the
    // partition is open, so that the stages of the boot behind it and of the
    // opening itself do not stop it part way (2026-11-19).
    if busy {
        // 与安装、备份那些卡里的条一个高度；长度按整行收窄到六成并居中：
        // 这段等待通常只有半秒上下，铺满整行会显得走过的路比实际长
        // （2026-11-25 用户要求缩短）。
        let (left, h) = (m.margin + m.inner, m.dp(6.0));
        let room = m.w as f32 - m.margin - m.inner - left;
        let w = room * BAR_SHARE;
        let x0 = left + (room - w) / 2.0;
        let bar = sign_cy + m.dp(152.0);
        let done = cx.state.data_progress.clamp(0.0, 1.0);
        c.fill_round_rect(x0, bar, w, h, h / 2.0, t.pal.card);
        if done > 0.001 {
            let filled = (w * done).max(h);
            c.fill_round_rect(x0, bar, filled, h, h / 2.0, t.pal.accent);
            // 条的前端一个白点，走到哪儿一眼看得见（2026-11-25 用户要求白点）。
            let dot = x0 + filled.clamp(h / 2.0, w - h / 2.0);
            c.fill_circle(dot, bar + h / 2.0, h / 2.0 + m.dp(2.0), WHITE);
        }
        let percent = format!("{}%", (done * 100.0).round() as u32);
        c.text_center(&f.body, &percent, x, bar + m.dp(26.0), m.sub_px, t.pal.subtitle);
    }
    let mut out = Vec::new();
    if let Some(why) = why {
        widgets::paragraph(&mut c, t, f, sign_cy + m.dp(88.0), why);
        let again = widgets::button(&mut c, t, f, m.h as f32 * 0.62, l.pick(["重试", "重試", "Try again"]));
        out.push(Hit { rect: again, action: Action::OpenData });
    }
    let leave_cy = m.h as f32 - m.dp(64.0);
    let leave = l.pick(if matches!(cx.state.data, Data::Open | Data::Erased) { ["完成", "完成", "Done"] } else { NOT_NOW });
    c.text_center(&f.heading, leave, x, leave_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), leave_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    Frame { header: false, ..Frame::new(t, c, out, l.pick(DECRYPT), 0.0) }
}

/// What leaves the decrypt page with the data partition left as it is.
const NOT_NOW: Text = ["暂不解密", "暫不解密", "Not now"];

/// What a row whose work waits on the data partition says under it until
/// that is decrypted.
const DECRYPT_NEEDED: Text = ["需先完成「解密」", "需先完成「解密」", "Decrypt first"];

/// A user's own files wait on their screen lock: the control it is
/// answered with, which takes nothing while what was entered is checked.
fn code(cx: &Ctx, mut c: Canvas, user: u32, keys: Keys, checking: bool) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let mut out = Vec::new();
    let title = match (keys == Keys::Pattern, user) {
        (true, 0) => l.pick(["绘制锁屏图案", "繪製螢幕鎖定圖案", "Draw the screen lock pattern"]).to_string(),
        (true, user) => tr!(l, "绘制用户 {user} 的锁屏图案", "繪製使用者 {user} 的螢幕鎖定圖案", "Draw user {user}'s screen lock pattern"),
        (false, 0) => l.pick(["输入锁屏密码", "輸入螢幕鎖定密碼", "Enter the screen lock"]).to_string(),
        (false, user) => tr!(l, "输入用户 {user} 的锁屏密码", "輸入使用者 {user} 的螢幕鎖定密碼", "Enter user {user}'s screen lock"),
    };
    let note = l.pick(if checking { ["正在验证", "正在驗證", "Checking"] } else { ["解密后可查看本机文件", "解密後可查看本機檔案", "Decrypt to see the phone's files"] });
    c.text_center(&f.heading, &title, x, m.dp(130.0), m.dp(24.0), t.pal.text);
    c.text_center(&f.body, note, x, m.dp(166.0), m.sub_px, t.pal.subtitle);
    let mut pattern = None;
    let bottom = match keys {
        Keys::Letters => {
            let top = m.h as f32 - widgets::keyboard_height(t);
            let caps = widgets::keyboard(&mut c, t, f, top, s.layer, s.key_held(), l.pick(DECRYPT), None, false);
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
        Keys::Pattern => {
            let span = m.dp(300.0);
            let cy = m.dp(300.0) + span / 2.0;
            widgets::pattern_lock(&mut c, t, x, cy, span, &s.pattern);
            if !checking {
                out.push(Hit { rect: Rect::new(x - span, cy - span, span * 2.0, span * 2.0), action: Action::Pattern });
                pattern = Some((x, cy, span));
            }
            m.h as f32 - m.dp(26.0)
        }
        Keys::Digits => {
            let top = m.h as f32 - widgets::keypad_height(t);
            let caps = widgets::keypad(&mut c, t, f, l, top, s.key_held());
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
    };
    // The dots are up under the words, and what is kept is said below them; a
    // pattern is its own showing of what has been drawn.
    if keys != Keys::Pattern {
        widgets::code_dots(&mut c, t, x, m.dp(212.0), s.code.chars().count());
    }
    let skip_cy = bottom - m.dp(34.0);
    c.text_center(&f.heading, l.pick(NOT_NOW), x, skip_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), skip_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    // Once something has been typed, it can be kept for the starts that
    // follow: the risk of keeping it is put before it when it is.
    if !s.code.is_empty() && !checking {
        // Under the dots, clear of the pattern where that is what answers.
        let row_y = match keys {
            Keys::Pattern => m.dp(636.0),
            _ => m.dp(258.0),
        };
        // The switch is the state itself: taken across, the lock is kept; left
        // as it is, it is not, and the words say which of the two it is.
        // A pill about what is written, not a row across the page: the dots
        // above are small, and a card from edge to edge would drown them.
        let kept = l.pick(["已记住密码", "已記住密碼", "Screen lock kept"]);
        let not_kept = l.pick(["未记住密码", "未記住密碼", "Screen lock not kept"]);
        let word = match s.remember {
            true => kept,
            false => not_kept,
        };
        let px = m.sub_px;
        let pad = m.dp(18.0);
        let gap = m.dp(12.0);
        let pill_h = m.dp(48.0);
        let text = c.text_width(&f.body, word, px);
        let total = pad + text + gap + m.switch_w + pad;
        let left = x - total / 2.0;
        c.fill_round_rect(left, row_y, total, pill_h, pill_h / 2.0, t.pal.card);
        c.text_center(&f.body, word, left + pad + text / 2.0, row_y + pill_h / 2.0, px, t.pal.text);
        widgets::switch(&mut c, t, left + total - pad, row_y + pill_h / 2.0, s.remember);
        out.push(Hit { rect: Rect::new(left, row_y, total, pill_h), action: Action::Toggle(Switch::Remember) });
    }
    Frame { header: false, pattern, ..Frame::new(t, c, out, l.pick(DECRYPT), 0.0) }
}

/// What a slot is shown as where the boot control could not be read.
const UNREADABLE: Text = ["无法读取", "無法讀取", "Unreadable"];

/// Reboot target picker with a swipe-to-confirm control.
pub fn reboot(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut y = widgets::section_first(&mut c, t, f, top, l.pick(["重启到", "重新啟動到", "Reboot to"]));
    let rows: Vec<Row> = REBOOT_TARGETS
        .iter()
        .enumerate()
        .map(|(k, &(title, sub))| {
            let r = Row::new(l.pick(title)).trail(Trail::Check(s.reboot == k));
            match sub {
                Some(text) => r.subtitle(l.pick(text)),
                None => r,
            }
        })
        .collect();
    let targets = widgets::list(&mut c, t, f, y, &rows);
    let picks: Vec<Action> = (0..TARGETS.len()).map(Action::RebootTarget).collect();
    out.extend(hits(&targets.rows, &picks));
    y = targets.bottom;
    if let Some(slot) = cx.info.slot {
        let next = match &s.boot_slots {
            Some(Ok(slots)) if slots.active == slot => {
                let slot = slot.letter();
                tr!(l, "{slot}（当前）", "{slot}（目前）", "{slot} (current)")
            }
            Some(Ok(slots)) => slots.active.letter().to_string(),
            Some(Err(_)) => l.pick(UNREADABLE).to_string(),
            None => String::new(),
        };
        y = widgets::section(&mut c, t, f, y, l.pick(["槽位", "槽位", "Slot"]));
        let slots = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["下次启动", "下次啟動", "Next boot"])).trail(Trail::Value(&next))]);
        out.extend(hits(&slots.rows, &[Action::Open(Screen::Slot)]));
        y = slots.bottom;
    }
    let (label, danger) = confirm_label(Confirm::Reboot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Reboot) });
    Frame::new(t, c, out, l.pick(REBOOT), top)
}

/// Installing, opened from the home page: the system on the phone now, and
/// where a package can come from.
pub fn install(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut facts = Vec::new();
    if let Some(v) = &i.os_version {
        facts.push((l.pick(["当前系统", "目前系統", "Current system"]), v.clone()));
    }
    if let Some(p) = &i.security_patch {
        facts.push((l.pick(SECURITY_PATCH), fmt::patch(l, p)));
    }
    if let Some(s) = i.slot {
        let (slot, other) = (s.letter(), s.other().letter());
        let detail = tr!(l, "{slot}，系统更新将安装到槽位 {other}", "{slot}，系統更新將安裝到槽位 {other}", "{slot}, and a system update installs to slot {other}");
        facts.push((l.pick(CURRENT_SLOT), detail));
    }
    let ways = l.pick(["安装方式", "安裝方式", "Ways to install"]);
    let y = if facts.is_empty() {
        widgets::section_first(&mut c, t, f, top, ways)
    } else {
        let y = widgets::section_first(&mut c, t, f, top, l.pick(["本机", "本機", "This phone"]));
        let y = widgets::info_card(&mut c, t, f, y, &facts);
        widgets::section(&mut c, t, f, y, ways)
    };
    let mut rows = Vec::new();
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::None));
    }
    // Internal storage opens with the data partition, which the decrypt
    // page opens or tells why it does not.
    let data = &cx.state.data;
    let (storage, from_storage) = if data.storage_open() {
        (
            ["选择内部存储中的更新包或镜像", "選擇內部儲存空間中的更新檔或映像檔", "Choose a package or image in internal storage"],
            Action::Open(Screen::Choose(Place::Storage)),
        )
    } else if *data == Data::Erased {
        (DATA_ERASED, Action::Open(Screen::Decrypt))
    } else {
        (DECRYPT_NEEDED, Action::Open(Screen::Decrypt))
    };
    // The drive's page waits for one to be plugged in, and says why one
    // that is cannot be read.
    let usb = match &cx.state.usb_drive {
        Some(UsbDrive::Shown) => ["选择 U 盘中的更新包或镜像", "選擇 USB 隨身碟中的更新檔或映像檔", "Choose a package or image on the USB drive"],
        Some(UsbDrive::Unreadable(_)) => USB_DRIVE_UNREADABLE,
        None => USB_DRIVE_WANTED,
    };
    rows.push(Row::new(install_title(l, Source::File(Place::Storage))).icon(BLUE, Icon::Phone).subtitle(l.pick(storage)));
    rows.push(Row::new(install_title(l, Source::File(Place::UsbDrive))).icon(PURPLE, Icon::UsbDrive).subtitle(l.pick(usb)));
    let sideload = l.pick(["在电脑上运行 adb sideload", "在電腦上執行 adb sideload", "Run adb sideload on the computer"]);
    rows.push(Row::new(install_title(l, Source::Computer)).icon(GREEN, Icon::Usb).subtitle(sideload));
    let methods = widgets::list(&mut c, t, f, y, &rows);
    let from_usb = Action::Open(Screen::Choose(Place::UsbDrive));
    let mut out = hits(&methods.rows[methods.rows.len() - 3..], &[from_storage, from_usb, Action::Sideload]);
    // A phone that can hold a second system installs one from here too.
    let what = l.pick(match cx.state.dual_boot {
        true => [
            "支持安装 A/B 完整更新包、带安装脚本的刷机包和第二系统的安装包，以及刷入 boot、init_boot、vendor_boot、recovery、dtbo、vbmeta 分区镜像。系统更新将安装到另一个槽位，重启后从该槽位启动；第二系统安装在 data 分区中，可在「双系统」中切换。选择文件时仅列出文件夹以及 .zip、.img 文件。",
            "支援安裝 A/B 完整更新檔、帶有安裝指令碼的刷機包和第二系統的安裝檔，以及刷入 boot、init_boot、vendor_boot、recovery、dtbo、vbmeta 分割區映像檔。系統更新將安裝到另一個槽位，重新啟動後從該槽位啟動；第二系統安裝在 data 分割區中，可在「雙系統」中切換。選擇檔案時僅列出資料夾以及 .zip、.img 檔案。",
            "Installs A/B full OTA packages, flashable zips with an install script and second systems' packages, and flashes boot, init_boot, vendor_boot, recovery, dtbo and vbmeta partition images. A system update installs to the other slot, which starts after a reboot; a second system installs into the data partition, and Dual boot switches to it. Choosing a file lists only folders and .zip and .img files.",
        ],
        false => [
            "支持安装 A/B 完整更新包和带安装脚本的刷机包，以及刷入 boot、init_boot、vendor_boot、recovery、dtbo、vbmeta 分区镜像。系统更新将安装到另一个槽位，重启后从该槽位启动。选择文件时仅列出文件夹以及 .zip、.img 文件。",
            "支援安裝 A/B 完整更新檔和帶有安裝指令碼的刷機包，以及刷入 boot、init_boot、vendor_boot、recovery、dtbo、vbmeta 分割區映像檔。系統更新將安裝到另一個槽位，重新啟動後從該槽位啟動。選擇檔案時僅列出資料夾以及 .zip、.img 檔案。",
            "Installs A/B full OTA packages and flashable zips with an install script, and flashes boot, init_boot, vendor_boot, recovery, dtbo and vbmeta partition images. A system update installs to the other slot, which starts after a reboot. Choosing a file lists only folders and .zip and .img files.",
        ],
    });
    let y = widgets::footnote(&mut c, t, f, methods.bottom, what);
    let y = widgets::section(&mut c, t, f, y, l.pick(["选项", "選項", "Options"]));
    let check = Row::new(l.pick(["验证刷机包签名", "驗證刷機包簽章", "Verify zip signatures"])).trail(Trail::Switch(cx.state.check_signature));
    let signature = widgets::list(&mut c, t, f, y, &[check]);
    out.extend(hits(&signature.rows, &[Action::Toggle(Switch::Signature)]));
    let note = l.pick(["开启后，仅安装带有本机官方签名的刷机包。", "開啟後，僅安裝帶有本機官方簽章的刷機包。", "When on, only zips signed with this phone's official keys install."]);
    let mut y = widgets::footnote(&mut c, t, f, signature.bottom, note);
    if cx.state.recovery_partitions {
        let row = Row::new(l.pick(["自动重新刷入 Skyward", "自動重新刷入 Skyward", "Flash Skyward again"])).trail(Trail::Switch(cx.state.keep_recovery));
        let options = widgets::list(&mut c, t, f, y + t.m.card_gap, &[row]);
        out.extend(hits(&options.rows, &[Action::Toggle(Switch::KeepRecovery)]));
        let note = l.pick([
            "开启后，安装系统更新或刷机包时被覆盖的 recovery 分区将重新刷入 Skyward。",
            "開啟後，安裝系統更新或刷機包時被覆寫的 recovery 分割區將重新刷入 Skyward。",
            "When on, Skyward goes back into the recovery partitions a system update or flashable zip writes over.",
        ]);
        y = widgets::footnote(&mut c, t, f, options.bottom, note);
    }
    Frame::new(t, fit(c, t, y), out, l.pick(["安装", "安裝", "Install"]), top)
}

const SECURITY_PATCH: Text = ["安全补丁", "安全性修補程式", "Security patch"];

/// How installing from `source` is named: on the install page, and as the
/// title of the page that installs.
fn install_title(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer => ["通过电脑安装", "透過電腦安裝", "Install from a computer"],
        Source::File(Place::Storage) => ["从内部存储安装", "從內部儲存空間安裝", "Install from internal storage"],
        Source::File(Place::UsbDrive) => ["从 U 盘安装", "從 USB 隨身碟安裝", "Install from a USB drive"],
        Source::Opened { .. } => ["从文件管理安装", "從檔案管理安裝", "Install from File manager"],
        Source::Asked { security: false } => ["安装系统更新", "安裝系統更新", "Install a system update"],
        Source::Asked { security: true } => ["安装安全更新", "安裝安全性更新", "Install a security update"],
        Source::Rescue => ["救援模式", "救援模式", "Rescue mode"],
    })
}

/// What a place is called at the top of its folders.
fn place_name(l: Lang, place: Place) -> &'static str {
    l.pick(match place {
        Place::Storage => ["内部存储", "內部儲存空間", "Internal storage"],
        Place::UsbDrive => ["U 盘", "USB 隨身碟", "USB drive"],
    })
}

/// A one-line card on how things stand.
fn standing<'a>(title: &'a str, color: Rgba, icon: Icon, detail: &'a str) -> Row<'a> {
    Row::new(title).icon(color, icon).subtitle(detail).trail(Trail::None)
}

fn share(done: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64) as f32
    }
}

fn percent(share: f32) -> u32 {
    (share * 100.0).floor() as u32
}

/// How much of an update is written and how fast it is going, where the
/// install says: a nine-gigabyte package is minutes of writing, and the bar
/// alone says nothing of how much of it is left.
fn write_pace(l: Lang, done: u64, total: u64, rate: f32) -> String {
    let (written, all) = (fmt::bytes(done), fmt::bytes(total));
    if rate < 1000.0 {
        return tr!(l, "已写入 {written} / {all}", "已寫入 {written} / {all}", "{written} of {all} written");
    }
    let speed = fmt::bytes(rate as u64);
    let left = fmt::left(l, (total.saturating_sub(done) as f64 / f64::from(rate)).round() as u64);
    tr!(
        l,
        "已写入 {written} / {all} · {speed}/s · 约剩 {left}",
        "已寫入 {written} / {all} · {speed}/s · 約剩 {left}",
        "{written} of {all} · {speed}/s · about {left} left"
    )
}

/// Where an update stands, and the share of its step done when known.
/// `shown` is how far the bar is drawn, which follows what the installer
/// reported a moment behind rather than in its own jumps.
fn update_stage(l: Lang, stage: Option<&Stage>, shown: Option<f32>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Writing { done, total }) => {
            let s = shown.unwrap_or_else(|| share(*done, *total));
            let p = percent(s);
            (tr!(l, "第 1 步，共 3 步：写入新系统 {p}%", "第 1 步，共 3 步：寫入新系統 {p}%", "Step 1 of 3: writing the new system {p}%"), Some(s))
        }
        Some(Stage::Verifying { index, count, partition, done, total }) => {
            let n = index + 1;
            let said = tr!(l, "第 2 步，共 3 步：校验 {partition}（{n}/{count}）", "第 2 步，共 3 步：驗證 {partition}（{n}/{count}）", "Step 2 of 3: verifying {partition} ({n}/{count})");
            // 有字节数就按字节走：一个大分区要读回二十秒，按分区个数走的话
            // 这二十秒里条一动不动（2026-11-21）。
            let share = if *total > 0 {
                *done as f32 / *total as f32
            } else {
                *index as f32 / *count as f32
            };
            (said, Some(share.clamp(0.0, 1.0)))
        }
        Some(Stage::Finishing) => (l.pick(["第 3 步，共 3 步：完成安装", "第 3 步，共 3 步：完成安裝", "Step 3 of 3: finishing"]).into(), None),
        Some(Stage::Reflashing { done, total }) => (l.pick(REFLASHING).into(), Some(*done as f32 / *total as f32)),
        Some(Stage::Flashing { .. } | Stage::Script { .. } | Stage::Signature { .. } | Stage::Allocating { .. } | Stage::Root { .. } | Stage::Storing) | None => {
            (l.pick(GETTING_READY).into(), None)
        }
    }
}

/// Where installing a system beside the installed one stands, and the share
/// of its step done when known.
fn system_stage(l: Lang, stage: Option<&Stage>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Allocating { done, total }) => {
            let s = share(*done, *total);
            let p = percent(s);
            (tr!(l, "第 1 步，共 3 步：在 data 分区中分配空间 {p}%", "第 1 步，共 3 步：在 data 分割區中分配空間 {p}%", "Step 1 of 3: setting room aside in the data partition {p}%"), Some(s))
        }
        Some(Stage::Root { done, total }) => {
            let s = share(*done, *total);
            let p = percent(s);
            let said = match done * 2 <= *total {
                true => tr!(l, "第 2 步，共 3 步：写入系统 {p}%", "第 2 步，共 3 步：寫入系統 {p}%", "Step 2 of 3: writing the system {p}%"),
                false => tr!(l, "第 2 步，共 3 步：校验系统 {p}%", "第 2 步，共 3 步：驗證系統 {p}%", "Step 2 of 3: verifying the system {p}%"),
            };
            (said, Some(s))
        }
        Some(Stage::Storing) => (l.pick(["第 3 步，共 3 步：保存启动镜像", "第 3 步，共 3 步：儲存啟動映像檔", "Step 3 of 3: keeping its boot images"]).into(), None),
        _ => (l.pick(GETTING_READY).into(), None),
    }
}

const REFLASHING: Text = ["正在重新刷入 Skyward", "正在重新刷入 Skyward", "Flashing Skyward again"];

/// Where putting the phone's root back into the boot image of the slot that
/// starts next has come, and how much of the bar that is: reading the image,
/// patching it, writing it back and reading it back weigh the bar by what
/// they move, except the patch, which counts no bytes and holds the bar
/// where the reading left it (2026-11-21).
fn root_stage(l: Lang, story: Story, done: u64, total: u64, slot: &str) -> (String, Option<f32>) {
    let part = share(done, total);
    let p = percent(part);
    let weighed = |from: f32, to: f32| Some(from + (to - from) * part);
    match story {
        Story::Reading => (
            tr!(
                l,
                "读出 {slot} 槽的启动镜像 {p}%",
                "讀出 {slot} 槽的啟動映像檔 {p}%",
                "Reading the {slot} slot's boot image, {p}%"
            ),
            weighed(0.0, 0.15),
        ),
        Story::Patching => (
            l.pick([
                "用好手机里的 root 工具打补丁",
                "用好手機裡的 root 工具打補丁",
                "Patching it with the phone's own root tool",
            ])
            .into(),
            Some(0.15),
        ),
        Story::Writing => (
            tr!(
                l,
                "写入启动镜像 {p}%",
                "寫入啟動映像檔 {p}%",
                "Writing the boot image back, {p}%"
            ),
            weighed(0.15, 0.9),
        ),
        Story::Checking => (
            tr!(
                l,
                "回读校验 {p}%",
                "讀回驗證 {p}%",
                "Reading it back to be sure, {p}%"
            ),
            weighed(0.9, 1.0),
        ),
    }
}

/// What the card says while an install script runs, and how far it is: the
/// last line the script showed and how far it says it is, then Skyward
/// going back into the recovery partitions it wrote over.
fn script_stage<'a>(l: Lang, stage: Option<&Stage>, said: &'a [String]) -> (&'a str, Option<f32>) {
    let running = l.pick(["正在运行安装脚本", "正在執行安裝指令碼", "Running the install script"]);
    let last = said.iter().rev().map(|line| line.trim()).find(|line| !line.is_empty()).unwrap_or(running);
    match stage {
        Some(Stage::Reflashing { done, total }) => (l.pick(REFLASHING), Some(*done as f32 / *total as f32)),
        Some(Stage::Script { thousandths }) => (last, Some(f32::from(*thousandths) / 1000.0)),
        _ => (last, None),
    }
}

/// How Skyward went back into the recovery partitions the install wrote
/// over, where it did.
fn reflashed(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    match &cx.state.reflash {
        None => y,
        Some(Reflash::Written(written)) => {
            let slots = fmt::slots(l, written);
            let said = match (l, written.len()) {
                (Lang::En, 1) => format!("Skyward went back into {slots}'s recovery partition."),
                _ => tr!(l, "已将 Skyward 重新刷入{slots} 的 recovery 分区。", "已將 Skyward 重新刷入{slots} 的 recovery 分割區。", "Skyward went back into the recovery partitions of {slots}."),
            };
            widgets::footnote(c, t, f, y, &said)
        }
        Some(Reflash::Failed { slots: none, .. }) if none.is_empty() => {
            // Which partitions even those were could not be worked out;
            // the log has what was looked at.
            let note = l.pick([
                "未能将 Skyward 重新刷入 recovery 分区，原因记在日志里。",
                "未能將 Skyward 重新刷入 recovery 分割區，原因記在日誌裡。",
                "Couldn't flash Skyward back into the recovery partitions; the log says why.",
            ]);
            widgets::footnote(c, t, f, y, note)
        }
        Some(Reflash::Failed { slots, why, saved }) => {
            let slots = fmt::slots(l, slots);
            let note = match why {
                Unflashed::ReadOnly => tr!(
                    l,
                    "未能将 Skyward 重新刷入{slots} 的 recovery 分区：分区被锁成了只读。",
                    "未能將 Skyward 重新刷入{slots} 的 recovery 分割區：分割區被鎖成了唯讀。",
                    "Couldn't flash Skyward back into {slots}'s recovery partition: it is locked read-only."
                ),
                Unflashed::Missing => tr!(
                    l,
                    "未能将 Skyward 重新刷入{slots}：这台手机没有那个 recovery 分区。",
                    "未能將 Skyward 重新刷入{slots}：這台手機沒有那個 recovery 分割區。",
                    "Couldn't flash Skyward back into {slots}: this phone has no such recovery partition."
                ),
                Unflashed::Busy => tr!(
                    l,
                    "未能将 Skyward 重新刷入{slots} 的 recovery 分区：分区正被占用。",
                    "未能將 Skyward 重新刷入{slots} 的 recovery 分割區：分割區正被佔用。",
                    "Couldn't flash Skyward back into {slots}'s recovery partition: something else holds it."
                ),
                Unflashed::Other => tr!(
                    l,
                    "未能将 Skyward 重新刷入{slots} 的 recovery 分区，原因记在日志里。",
                    "未能將 Skyward 重新刷入{slots} 的 recovery 分割區，原因記在日誌裡。",
                    "Couldn't flash Skyward back into {slots}'s recovery partition; the log says why."
                ),
            };
            let y = widgets::footnote(c, t, f, y, &note);
            match saved {
                // The copy left on storage is what a computer flashes back
                // with, so the phone is not left without a recovery at all.
                Some(at) => {
                    let said = tr!(
                        l,
                        "Skyward 镜像已存到 {at}：用 fastboot flash recovery 那个文件即可刷回。",
                        "Skyward 映像檔已存到 {at}：用 fastboot flash recovery 那個檔案即可刷回。",
                        "A copy of Skyward is at {at}: `fastboot flash recovery` it to put it back."
                    );
                    widgets::footnote(c, t, f, y, &said)
                }
                None => y,
            }
        }
    }
}

/// What an update leaves to say once its package is in: whether it is being
/// merged, which this phone needs before it starts the new system, and where
/// the boot images went that the root it took away has to be put back into
/// (2026-11-20).
fn after_update(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut y = y;
    match cx.state.update {
        // The merge is followed on the installing page with a bar of its
        // own, so the page that says the install is done only comes once it
        // has ended.
        Standing::Merged => {
            let said = l.pick(["更新已合并，可以重启。", "更新已合併，可以重新啟動。", "The update is merged; a reboot starts the new system."]);
            y = widgets::footnote(c, t, f, y, said);
        }
        // A merge that stopped leaves the update in the slot; the page to
        // start it again is the system update's.
        Standing::Failed(_) => {
            let said = l.pick([
                "更新合并没有完成，可在「高级 > 系统更新」里再合并一次。",
                "更新合併沒有完成，可在「進階 > 系統更新」裡再合併一次。",
                "The update did not finish merging. Start it again from Advanced > System update.",
            ]);
            y = widgets::footnote(c, t, f, y, said);
        }
        _ => {}
    }
    match &cx.state.root {
        // Root went back into the image the update wrote: nothing more to
        // do, and the phone starts the new system with it.
        Some(Rooted::Kept(kept)) => {
            let (part, what) = (kept.partition, kept.what);
            let said = match kept.from {
                // The image the phone runs from has the root that works in
                // it, and went into the slot that starts next whole.
                Some(from) => {
                    let from = from.letter();
                    tr!(
                        l,
                        "已把 {from} 槽的 {part} 整个刷进新系统：{what} 的 root 就在里面，重启后仍然是 root。",
                        "已把 {from} 槽的 {part} 整個刷進新系統：{what} 的 root 就在裡面，重新啟動後仍然是 root。",
                        "The {part} of slot {from} went into the new system whole: {what}'s root is in it, and a reboot starts with it."
                    )
                }
                None => tr!(
                    l,
                    "{what} 的 root 已经重新打进 {part} 的镜像，重启后仍然是 root。",
                    "{what} 的 root 已經重新打進 {part} 的映像檔，重新啟動後仍然是 root。",
                    "{what}'s root is back in the {part} image; a reboot starts the new system with it."
                ),
            };
            y = widgets::footnote(c, t, f, y, &said);
        }
        // Root lives in the kernel, and the tools that patch it stay in the
        // manager that installed it: say which.
        Some(Rooted::Other { what }) => {
            let said = tr!(
                l,
                "这台手机的 root 是 {what}，要用它自己的管理器再装一次。",
                "這台手機的 root 是 {what}，要用它自己的管理器再裝一次。",
                "This phone's root is {what}, which has to be installed again from its own manager."
            );
            y = widgets::footnote(c, t, f, y, &said);
        }
        // One the recovery patches, and the patch did not happen: its own
        // manager patches the image by hand.
        Some(Rooted::Missed { what }) => {
            let said = tr!(
                l,
                "{what} 的 root 没能自动补回去，原因记在日志里；在 {what} 管理器里选「选择并修补镜像」打一次，再刷回去。",
                "{what} 的 root 沒能自動補回去，原因記在日誌裡；在 {what} 管理器裡選「選擇並修補映像檔」打一次，再刷回去。",
                "{what}'s root was not put back automatically; the log says why. Patch the boot image from {what}'s install menu, then flash it back."
            );
            y = widgets::footnote(c, t, f, y, &said);
        }
        Some(Rooted::None) => {}
        // Nothing is known of the root yet: the page says the update took
        // it away while the patch is being tried.
        None => {
            let said = l.pick([
                "系统更新覆盖了启动镜像，正在把 root 补回去。",
                "系統更新覆寫了啟動映像檔，正在把 root 補回去。",
                "The update wrote over the boot images; root is being put back.",
            ]);
            y = widgets::footnote(c, t, f, y, said);
        }
    }
    y
}

/// What the install script showed, where there is any: its heading, and
/// under it the lines as the page's long list, the last thing on the page.
fn script_output(c: &mut Canvas, cx: &Ctx, y: f32, rows: &mut Option<Rows>) -> f32 {
    let lines = &cx.state.script_output;
    if lines.is_empty() {
        return y;
    }
    let y = widgets::section(c, cx.t, cx.f, y, cx.state.lang.pick(["安装脚本输出", "安裝指令碼輸出", "Install script output"]));
    *rows = Some(Rows { edges: widgets::output_edges(c, cx.t, cx.f, y, lines), action: None });
    y
}

fn pick_title(l: Lang, pick: Pick, current: Option<Slot>) -> String {
    let letter = |s: Option<Slot>| s.map(|s| format!(" {}", s.letter())).unwrap_or_default();
    match pick {
        Pick::Current => format!("{}{}", l.pick(CURRENT_SLOT), letter(current)),
        Pick::Other => format!("{}{}", l.pick(["另一个槽位", "另一個槽位", "Other slot"]), letter(current.map(Slot::other))),
        Pick::Both => l.pick(["两个槽位", "兩個槽位", "Both slots"]).into(),
    }
}

/// Words the installing page says of more than one of its steps.
const SWITCHING_USB: Text = ["正在切换 USB 模式", "正在切換 USB 模式", "Switching USB mode"];
const READING_FILE: Text = ["正在读取文件", "正在讀取檔案", "Reading the file"];
const WAITING: Text = ["等待电脑发送", "等待電腦傳送", "Waiting for the computer"];
const CONNECT: Text = ["请用数据线连接电脑，并在电脑上运行以下命令", "請用傳輸線連接電腦，並在電腦上執行以下指令", "Connect a computer with a cable, and run these commands on it"];
const RUN_ON_COMPUTER: Text = ["在电脑上运行", "在電腦上執行", "Run on the computer"];
const KEEP_CABLE: Text = ["读取完成前，请不要断开数据线。", "讀取完成前，請不要拔除傳輸線。", "Keep the cable in until it's read."];
const KEEP_DRIVE: Text = ["读取完成前，请不要拔出 U 盘。", "讀取完成前，請不要拔出 USB 隨身碟。", "Keep the USB drive in until it's read."];
const GETTING_READY: Text = ["正在准备", "正在準備", "Getting ready"];

/// Installing a package step by step: receiving what the computer sends
/// with `adb sideload`, or reading the file chosen, then installing it.
pub fn installing(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let computer = s.source == Source::Computer;
    let bottom = match &s.install {
        Install::Idle | Install::Starting | Install::Ending if computer => {
            let row = standing(l.pick(SWITCHING_USB), GREEN, Icon::Usb, l.pick(["从 adb 调试切换到 sideload 模式", "從 adb 偵錯切換到 sideload 模式", "From adb debugging to sideload"]));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending if s.source == Source::Rescue => {
            let row = standing(l.pick(SWITCHING_USB), GREEN, Icon::Usb, l.pick(["从 adb 调试切换到救援模式", "從 adb 偵錯切換到救援模式", "From adb debugging to rescue mode"]));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending => {
            widgets::list(&mut c, t, f, top, &[Row::new(l.pick(READING_FILE)).icon(GREEN, Icon::Install).trail(Trail::None)]).bottom
        }
        Install::Opening => {
            let opening = Row::new(l.pick(["正在打开 data 分区", "正在開啟 data 分割區", "Opening the data partition"])).icon(GREEN, Icon::Lock).trail(Trail::None);
            let y = widgets::list(&mut c, t, f, top, &[opening]).bottom;
            asked_package(&mut c, cx, y)
        }
        Install::Locked(_) => {
            let row = standing(l.pick(["暂无法读取更新包", "暫時無法讀取更新檔", "The package can't be read yet"]), ORANGE, Icon::Lock, l.pick(DECRYPT_NEEDED));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let open = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DECRYPT))]);
            out.extend(hits(&open.rows, &[Action::Open(Screen::Decrypt)]));
            let y = widgets::footnote(&mut c, t, f, open.bottom, l.pick(["解密后将自动安装。", "解密後將自動安裝。", "It installs once the data partition is decrypted."]));
            asked_package(&mut c, cx, y)
        }
        Install::Waiting if s.source == Source::Rescue => {
            let row = standing(l.pick(WAITING), GREEN, Icon::Usb, l.pick(CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(&mut c, t, f, y, &[
                (
                    l.pick(["adb rescue install 文件名", "adb rescue install 檔案名稱", "adb rescue install <file>"]),
                    l.pick([
                        "将文件名替换为系统更新包、刷机包或分区镜像",
                        "將檔案名稱替換為系統更新檔、刷機包或分割區映像檔",
                        "With a system update package, flashable zip or partition image as <file>",
                    ])
                    .into(),
                ),
                ("adb rescue getprop", l.pick(["读取本机的版本和设备信息", "讀取本機的版本和裝置資訊", "Reads this phone's versions and device details"]).into()),
                ("adb rescue wipe userdata", l.pick(["清除本机的全部数据", "清除本機的全部資料", "Wipes all of this phone's data"]).into()),
            ]);
            let note = l.pick([
                "需在电脑上安装 Android SDK Platform-Tools。返回后将退出救援模式，并将 USB 切换回 adb 调试。",
                "需在電腦上安裝 Android SDK Platform-Tools。返回後將結束救援模式，並將 USB 切換回 adb 偵錯。",
                "The computer needs Android SDK Platform-Tools. Going back leaves rescue mode and switches USB back to adb debugging.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Waiting => {
            let row = standing(l.pick(WAITING), GREEN, Icon::Usb, l.pick(CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(&mut c, t, f, y, &[(
                l.pick(["adb sideload 文件名", "adb sideload 檔案名稱", "adb sideload <file>"]),
                l.pick([
                    "将文件名替换为系统更新包、刷机包或分区镜像，例如 boot.img",
                    "將檔案名稱替換為系統更新檔、刷機包或分割區映像檔，例如 boot.img",
                    "With a system update package, flashable zip or partition image as <file>, such as boot.img",
                ])
                .into(),
            )]);
            let note = l.pick([
                "需在电脑上安装 Android SDK Platform-Tools。返回后将停止等待，并将 USB 切换回 adb 调试。",
                "需在電腦上安裝 Android SDK Platform-Tools。返回後將停止等待，並將 USB 切換回 adb 偵錯。",
                "The computer needs Android SDK Platform-Tools. Going back stops waiting and switches USB back to adb debugging.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { stage: Some(Stage::Signature { done, total }), .. } => {
            let read = share(*done, *total);
            let p = percent(read);
            let detail = tr!(l, "已读取 {p}%", "已讀取 {p}%", "{p}% read");
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(["正在验证签名", "正在驗證簽章", "Verifying the signature"]), &detail, Some(read));
            let note = l.pick(match s.source {
                Source::Computer => [
                    "读取完成前，请不要断开数据线。返回将取消安装，并结束电脑上的 adb sideload。",
                    "讀取完成前，請不要拔除傳輸線。返回將取消安裝，並結束電腦上的 adb sideload。",
                    "Keep the cable in until it's read. Going back cancels the install and ends adb sideload on the computer.",
                ],
                Source::Rescue => [
                    "读取完成前，请不要断开数据线。返回将取消安装，并退出救援模式。",
                    "讀取完成前，請不要拔除傳輸線。返回將取消安裝，並結束救援模式。",
                    "Keep the cable in until it's read. Going back cancels the install and leaves rescue mode.",
                ],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => ["返回将取消安装。", "返回將取消安裝。", "Going back cancels the install."],
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => [
                    "读取完成前，请不要拔出 U 盘。返回将取消安装。",
                    "讀取完成前，請不要拔出 USB 隨身碟。返回將取消安裝。",
                    "Keep the USB drive in until it's read. Going back cancels the install.",
                ],
            });
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { size, .. } => {
            let size = fmt::bytes(*size);
            let detail = tr!(l, "文件大小 {size}", "檔案大小 {size}", "File size {size}");
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(READING_FILE), GREEN, Icon::Install, &detail)]).bottom;
            match s.source {
                Source::Computer | Source::Rescue => widgets::footnote(&mut c, t, f, y, l.pick(KEEP_CABLE)),
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => y,
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => widgets::footnote(&mut c, t, f, y, l.pick(KEEP_DRIVE)),
            }
        }
        Install::Choosing(choice) => {
            let (name, holds) = fmt::image_kind(l, choice.kind);
            let size = fmt::bytes(choice.size);
            let detail = tr!(l, "{size}，{holds}", "{size}，{holds}", "{size} · {holds}");
            let y = widgets::list(&mut c, t, f, top, &[standing(name, GREEN, Icon::Install, &detail)]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(["分区", "分割區", "Partition"]));
            let rows: Vec<Row> = choice
                .partitions
                .iter()
                .enumerate()
                .map(|(k, p)| Row::new(p).subtitle(fmt::partition_role(l, p)).trail(Trail::Check(k == choice.partition)))
                .collect();
            let parts = widgets::list(&mut c, t, f, y, &rows);
            let choose: Vec<Action> = (0..choice.partitions.len()).map(Action::FlashPartition).collect();
            out.extend(hits(&parts.rows, &choose));
            let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
            let titles = Pick::ALL.map(|p| pick_title(l, p, i.slot));
            let rows: Vec<Row> =
                Pick::ALL.iter().zip(&titles).map(|(p, title)| Row::new(title).trail(Trail::Check(*p == choice.pick))).collect();
            let picks = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&picks.rows, &Pick::ALL.map(Action::FlashSlots)));
            let (label, danger) = confirm_label(Confirm::Flash, cx.state);
            let track = widgets::confirm(&mut c, t, f, picks.bottom + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::Flash) });
            let note = l.pick(match s.source {
                Source::Computer => [
                    "刷入将覆盖分区中的原有内容。返回将取消刷入，并结束电脑上的 adb sideload。",
                    "刷入將覆寫分割區中的原有內容。返回將取消刷入，並結束電腦上的 adb sideload。",
                    "Flashing writes over what the partition holds. Going back cancels it and ends adb sideload on the computer.",
                ],
                Source::Rescue => [
                    "刷入将覆盖分区中的原有内容。返回将取消刷入，并退出救援模式。",
                    "刷入將覆寫分割區中的原有內容。返回將取消刷入，並結束救援模式。",
                    "Flashing writes over what the partition holds. Going back cancels it and leaves rescue mode.",
                ],
                Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => [
                    "刷入将覆盖分区中的原有内容。返回将取消刷入。",
                    "刷入將覆寫分割區中的原有內容。返回將取消刷入。",
                    "Flashing writes over what the partition holds. Going back cancels it.",
                ],
            });
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Confirming { version, wipe } => {
            let (title, color, icon, detail, which) = if *wipe {
                let reset = factory_reset(s);
                let detail = match version {
                    Some(v) => tr!(l, "安装 {v} 后将{reset}。", "安裝 {v} 後將{reset}。", "Installing {v} then {reset}."),
                    None => tr!(l, "安装该更新后将{reset}。", "安裝這個更新後將{reset}。", "Installing this update then {reset}."),
                };
                (l.pick(["需清除全部数据", "需清除全部資料", "All data has to be wiped"]).to_string(), ORANGE, Icon::Wipe, detail, Confirm::InstallWipe)
            } else {
                let title = match version {
                    Some(v) => tr!(l, "安装 {v}", "安裝 {v}", "Install {v}"),
                    None => l.pick(["安装系统更新", "安裝系統更新", "Install the system update"]).into(),
                };
                let detail = match i.slot.map(|s| s.other().letter()) {
                    Some(slot) => tr!(
                        l,
                        "新系统将安装到槽位 {slot}，重启后将从该槽位启动。",
                        "新系統將安裝到槽位 {slot}，重新啟動後將從該槽位啟動。",
                        "The new system installs to slot {slot}, which starts after a reboot."
                    ),
                    None => l.pick(["重启后将从新系统启动。", "重新啟動後將從新系統啟動。", "The new system starts after a reboot."]).into(),
                };
                (title, GREEN, Icon::Install, detail, Confirm::Install)
            };
            let y = widgets::list(&mut c, t, f, top, &[standing(&title, color, icon, &detail)]).bottom;
            let (label, danger) = confirm_label(which, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(which) });
            let note = l.pick(["返回将取消安装，不会写入任何内容。", "返回將取消安裝，不會寫入任何內容。", "Going back cancels the install without writing anything."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Updating { version, stage, overwriting } => {
            let title = match version {
                Some(v) => tr!(l, "正在安装 {v}", "正在安裝 {v}", "Installing {v}"),
                None => l.pick(["正在安装系统更新", "正在安裝系統更新", "Installing the system update"]).into(),
            };
            let (detail, done) = update_stage(l, stage.as_ref(), Some(s.install_share));
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            // How much of it is written, and how fast: a nine-gigabyte
            // package is six minutes of writing, and a bar alone says
            // nothing of how much of it is left (2026-11-20).
            let y = match stage {
                Some(Stage::Writing { done: written, total }) if *total > 0 => {
                    let said = write_pace(l, *written, *total, s.install_rate);
                    widgets::footnote(&mut c, t, f, y, &said)
                }
                _ => y,
            };
            // What update_engine is doing, one line at a time, right under
            // the count: the bar hardly moves between the counts it reports,
            // and these lines keep arriving (2026-11-20).
            let y = widgets::install_lines(&mut c, t, f, y, &s.install_log);
            let until = until_installed(l, s.source);
            let note = match i.slot.map(|s| s.other().letter()) {
                Some(slot) => tr!(l, "{until}新系统将安装到槽位 {slot}。", "{until}新系統將安裝到槽位 {slot}。", "{until} The new system installs to slot {slot}."),
                None => until.to_string(),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            if *overwriting {
                let note = l.pick([
                    "正在覆盖当前系统，安装完成前重启将无法进入系统。",
                    "正在覆寫目前的系統，安裝完成前重新啟動將無法進入系統。",
                    "The current system is being written over, so a reboot before the install finishes starts no system.",
                ]);
                widgets::footnote(&mut c, t, f, y, note)
            } else {
                y
            }
        }
        Install::Merging { share: merged, .. } => {
            // The bar the writing had goes on with the merge: the slot that
            // starts next cannot start before it, so the install is not
            // done yet, whatever the writing said (2026-11-20).
            let p = percent(*merged);
            let detail = tr!(l, "已完成 {p}%", "已完成 {p}%", "{p}% done");
            let title = l.pick(["正在合并更新", "正在合併更新", "Merging the update"]);
            let y = widgets::progress_card(&mut c, t, f, top, title, &detail, Some(*merged));
            let note = l.pick([
                "合并完成前，请不要关机。合并完成后会停在这一页，由你手动重启进入新系统。",
                "合併完成前，請不要關機。合併完成後會停在這一頁，由你手動重新啟動進入新系統。",
                "Don't power off until it's merged. It stays on this page once it has, for you to restart into the new system.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Rooting { story, done, total, slot, .. } => {
            // The bar the merge had goes on with the boot image: the slot
            // that starts next cannot start with the phone's root until the
            // image it starts from has it, so the install is not done yet
            // (2026-11-21).
            let (detail, share) = root_stage(l, *story, *done, *total, slot.letter());
            let title = l.pick(["正在把 root 写回启动分区", "正在把 root 寫回啟動分割區", "Putting root back into the boot image"]);
            let y = widgets::progress_card(&mut c, t, f, top, title, &detail, share);
            let note = l.pick([
                "写回的是新系统自己的启动镜像，重启后仍然是 root。请不要关机。",
                "寫回的是新系統自己的啟動映像檔，重新啟動後仍然是 root。請不要關機。",
                "The image written back is the new system's own, so a restart still comes up with root. Don't power off.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Flashing { partition, stage } => {
            let (detail, done) = match stage {
                Some(Stage::Flashing { done, total }) => {
                    let s = share(*done, *total);
                    let p = percent(s);
                    let detail = match done * 2 < *total {
                        true => tr!(l, "正在写入 {p}%", "正在寫入 {p}%", "Writing {p}%"),
                        false => tr!(l, "正在校验 {p}%", "正在驗證 {p}%", "Verifying {p}%"),
                    };
                    (detail, Some(s))
                }
                _ => (l.pick(GETTING_READY).into(), None),
            };
            let title = tr!(l, "正在刷入 {partition}", "正在刷入 {partition}", "Flashing {partition}");
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            let keep = l.pick(match s.source {
                Source::Computer | Source::Rescue => ["刷入完成前，请不要断开数据线。", "刷入完成前，請不要拔除傳輸線。", "Keep the cable in until it's flashed."],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => {
                    ["刷入完成前，请不要关机。", "刷入完成前，請不要關機。", "Don't power off until it's flashed."]
                }
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
                    ["刷入完成前，请不要拔出 U 盘。", "刷入完成前，請不要拔出 USB 隨身碟。", "Keep the USB drive in until it's flashed."]
                }
            });
            widgets::footnote(&mut c, t, f, y, keep)
        }
        Install::SystemChoosing(choice) => {
            let version = &choice.version;
            let detail = tr!(l, "{version}，将安装为第二系统", "{version}，將安裝為第二系統", "{version}, to install as the second system");
            let y = widgets::list(&mut c, t, f, top, &[standing(&choice.name, PURPLE, Icon::Systems, &detail)]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(["分配空间", "分配空間", "Room for it"]));
            let sizes: Vec<String> = choice.sizes.iter().map(|&b| fmt::bytes(b)).collect();
            let least = l.pick(["系统所需的最小空间", "系統所需的最小空間", "The least the system takes"]);
            let rows: Vec<Row> = sizes
                .iter()
                .enumerate()
                .map(|(k, size)| {
                    let row = Row::new(size).trail(Trail::Check(k == choice.size));
                    if k == 0 { row.subtitle(least) } else { row }
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            let picks: Vec<Action> = (0..choice.sizes.len()).map(Action::SystemSize).collect();
            out.extend(hits(&list.rows, &picks));
            let note = l.pick([
                "所选空间将从 data 分区中划出，供第二系统存放文件，安装后无法更改；删除第二系统后归还。主系统的分区不会被写入。",
                "所選空間將從 data 分割區中劃出，供第二系統存放檔案，安裝後無法變更；刪除第二系統後歸還。主系統的分割區不會被寫入。",
                "The room chosen is set aside in the data partition for the second system's files, can't be changed once it's installed, and comes back when the second system is removed. Nothing of the main system's partitions is written.",
            ]);
            let y = widgets::footnote(&mut c, t, f, list.bottom, note);
            let (label, danger) = confirm_label(Confirm::InstallSystem, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::InstallSystem) });
            let note = l.pick(["返回将取消安装，不会写入任何内容。", "返回將取消安裝，不會寫入任何內容。", "Going back cancels the install without writing anything."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Systeming { name, stage } => {
            let title = tr!(l, "正在安装 {name}", "正在安裝 {name}", "Installing {name}");
            let (detail, done) = system_stage(l, stage.as_ref());
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            widgets::footnote(&mut c, t, f, y, until_installed(l, s.source))
        }
        Install::ConfirmingScript => {
            let name = s.package_name.as_deref().unwrap_or(l.pick(["刷机包", "刷機包", "Flashable zip"]));
            let detail = l.pick([
                "该刷机包将运行其中的安装脚本，安装脚本可以改写本机的任何分区。",
                "這個刷機包將執行其中的安裝指令碼，安裝指令碼可以改寫本機的任何分割區。",
                "This zip runs the install script it holds, which can rewrite any partition of this phone.",
            ]);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(name, ORANGE, Icon::Install, detail)]).bottom;
            if let Some(note) = fmt::snapshot_script_note(l, &i.snapshot) {
                y = widgets::footnote(&mut c, t, f, y, note);
            }
            let (label, danger) = confirm_label(Confirm::RunScript, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RunScript) });
            let note = l.pick(["返回将取消安装，不会运行安装脚本。", "返回將取消安裝，不會執行安裝指令碼。", "Going back cancels the install without running the script."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Scripting { stage, .. } => {
            let (detail, done) = script_stage(l, stage.as_ref(), &s.script_output);
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(["正在安装", "正在安裝", "Installing"]), detail, done);
            let y = widgets::footnote(&mut c, t, f, y, until_installed(l, s.source));
            script_output(&mut c, cx, y, &mut rows)
        }
        Install::Wiping => {
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(WIPING), &capitalized(l, &factory_reset(s)), None);
            widgets::footnote(&mut c, t, f, y, l.pick(KEEP_POWER))
        }
        Install::Wiped(Ok(())) => {
            let made_anew = l.pick(["下次开机时将重新创建 data 分区", "下次開機時將重新建立 data 分割區", "The data partition is made anew when the phone next starts"]);
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(["已清除", "已清除", "Wiped"]), GREEN, Icon::Wipe, made_anew)]).bottom;
            widgets::footnote(&mut c, t, f, y, l.pick(MORE_COMMANDS))
        }
        Install::Wiped(Err(stop)) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(l.pick(WIPE_INCOMPLETE), RED, Icon::Wipe, &what)]).bottom;
            if let Some(d) = detail {
                y = widgets::footnote(&mut c, t, f, y, &d);
            }
            widgets::footnote(&mut c, t, f, y, l.pick(MORE_COMMANDS))
        }
        Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, *damage, &mut out, &mut rows),
    };
    // What the system asked for says why, where it said.
    let bottom = match (s.source, &s.asked.reason) {
        (Source::Asked { .. }, Some(reason)) => widgets::info_card(&mut c, t, f, bottom + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]),
        _ => bottom,
    };
    Frame { rows, ..Frame::new(t, fit(c, t, bottom), out, install_title(l, s.source), top) }
}

/// What a request the system made gives as why.
const REASON: Text = ["原因", "原因", "Reason"];

const WIPING: Text = ["正在清除", "正在清除", "Wiping"];
const WIPE_INCOMPLETE: Text = ["清除未完成", "清除未完成", "Wipe incomplete"];
const KEEP_POWER: Text = ["清除完成前，请不要关机。", "清除完成前，請不要關機。", "Don't power off until it's wiped."];

/// What rescue says once a command is done with: the computer may send
/// another.
const MORE_COMMANDS: Text = ["你可以继续在电脑上发送命令。", "你可以繼續在電腦上傳送指令。", "You can send more commands from the computer."];

/// The package the system asked for, by the path it gave, at `y`.
fn asked_package(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let name = cx.state.package_name.clone().unwrap_or_default();
    widgets::info_card(c, cx.t, cx.f, y + cx.t.m.dp(12.0), &[(cx.state.lang.pick(["更新包", "更新檔", "Package"]), name)])
}

/// What not to do while a package from `source` is installed.
fn until_installed(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer | Source::Rescue => {
            ["安装完成前，请不要断开数据线或关机。", "安裝完成前，請不要拔除傳輸線或關機。", "Keep the cable in and don't power off until it's installed."]
        }
        Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => {
            ["安装完成前，请不要关机。", "安裝完成前，請不要關機。", "Don't power off until it's installed."]
        }
        Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
            ["安装完成前，请不要拔出 U 盘或关机。", "安裝完成前，請不要拔出 USB 隨身碟或關機。", "Keep the USB drive in and don't power off until it's installed."]
        }
    })
}

/// A phrase that opens a line on its own: in English with its first letter
/// a capital.
fn capitalized(l: Lang, phrase: &str) -> String {
    let mut chars = phrase.chars();
    match (l, chars.next()) {
        (Lang::En, Some(first)) => first.to_uppercase().chain(chars).collect(),
        _ => phrase.to_string(),
    }
}

/// How an install ended, and where to go from there: another package from
/// the computer, or back to choose another file.
fn finished(
    c: &mut Canvas,
    cx: &Ctx,
    top: f32,
    outcome: &Outcome,
    damage: Option<Damage>,
    out: &mut Vec<Hit>,
    rows: &mut Option<Rows>,
) -> f32 {
    let (t, f, source, l) = (cx.t, cx.f, cx.state.source, cx.state.lang);
    let gap = t.m.card_gap;
    let choose_another = ["选择其他文件", "選擇其他檔案", "Choose another file"];
    let (again, another) = match source {
        Source::Computer => (Row::new(l.pick(["再次发送", "再次傳送", "Send again"])).icon(GREEN, Icon::Usb), Action::Sideload),
        Source::File(_) | Source::Opened { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Back),
        Source::Asked { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Open(Screen::Install)),
        Source::Rescue => (Row::new(l.pick(["退出救援模式", "結束救援模式", "Leave rescue mode"])).icon(GREEN, Icon::Usb), Action::Back),
    };
    let restart = Row::new(l.pick(["重启到系统", "重新啟動到系統", "Reboot to system"])).icon(ORANGE, Icon::Power);
    let installed = l.pick(["安装成功", "安裝成功", "Installed"]);
    let nothing_written = l.pick(["未写入任何内容。", "未寫入任何內容。", "Nothing was written."]);
    match outcome {
        Outcome::Installed { slot, .. } => {
            let slot = slot.letter();
            let detail = tr!(
                l,
                "新系统已安装到槽位 {slot}，重启后将从该槽位启动。",
                "新系統已安裝到槽位 {slot}，重新啟動後將從該槽位啟動。",
                "The new system is installed to slot {slot}, which starts after a reboot."
            );
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Install, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System)]));
            let y = reflashed(c, cx, next.bottom);
            let y = after_update(c, cx, y);
            afterwards_of(c, cx, y, true)
        }
        Outcome::Flashed { partition, slots } => {
            let slots = fmt::slots(l, slots);
            let detail = tr!(l, "{partition} 已刷入{slots}。", "{partition} 已刷入{slots}。", "Flashed to {partition} in {slots}.");
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["刷入成功", "刷入成功", "Flashed"]), GREEN, Icon::Install, &detail)]).bottom;
            let mut rows = vec![restart];
            let mut acts = vec![Action::RebootTo(Target::System)];
            if *partition == "recovery" {
                rows.push(Row::new(l.pick(["重启到恢复模式", "重新啟動到復原模式", "Reboot to recovery"])).icon(ORANGE, Icon::Power));
                acts.push(Action::RebootTo(Target::Recovery));
            }
            rows.push(again);
            acts.push(another);
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Ran { .. } => {
            let ran = l.pick(["安装脚本已运行完成。", "安裝指令碼已執行完成。", "The install script ran to its end."]);
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Install, ran)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart, again]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System), another]));
            let y = reflashed(c, cx, next.bottom);
            let y = afterwards_of(c, cx, y, true);
            script_output(c, cx, y, rows)
        }
        Outcome::SystemInstalled { name } => {
            let detail = tr!(
                l,
                "{name} 已安装为第二系统，可在「双系统」中切换。",
                "{name} 已安裝為第二系統，可在「雙系統」中切換。",
                "{name} is installed as the second system. Switch to it from Dual boot."
            );
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Systems, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems)]);
            out.extend(hits(&next.rows, &[Action::Open(Screen::Systems)]));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::ModuleInstalled { id } => {
            // 模块不写分区，也不是刷机包：它的 update-binary 是 Magisk 自己跑的
            // 那一段，装进 /data/adb/modules 就等下次启动由 root 方案加载。
            let detail = tr!(
                l,
                "模块 {id} 已放入 /data/adb/modules，下次启动时加载。",
                "模組 {id} 已放入 /data/adb/modules，下次啟動時載入。",
                "The module {id} is in /data/adb/modules and loads at the next start."
            );
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["模块已安装", "模組已安裝", "Module installed"]), GREEN, Icon::Install, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart, again]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System), another]));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Cancelled => {
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["已取消", "已取消", "Cancelled"]), SLATE, Icon::Install, nothing_written)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again]);
            out.extend(hits(&next.rows, &[another]));
            next.bottom
        }
        Outcome::Refused(r) => {
            let reason = fmt::refusal(l, r, &cx.info.codename, source, cx.state.zone());
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["无法安装", "無法安裝", "Can't install"]), ORANGE, Icon::Install, &reason)]).bottom;
            // What the system asked for and did not go in leaves the phone
            // where it was, from which it may go back to the system.
            let (rows, acts) = match source {
                Source::Asked { .. } => (vec![again, restart], vec![another, Action::RebootTo(Target::System)]),
                _ => (vec![again], vec![another]),
            };
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            let y = widgets::footnote(c, t, f, next.bottom, nothing_written);
            afterwards_of(c, cx, y, false)
        }
        Outcome::Failed(failure) => {
            let (what, detail) = fmt::failure(l, failure, source);
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["安装失败", "安裝失敗", "Install failed"]), RED, Icon::Install, &what)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again, restart]);
            out.extend(hits(&next.rows, &[another, Action::RebootTo(Target::System)]));
            // The package is sent again, or installed or flashed again.
            let (install, flash) = match source {
                Source::Computer | Source::Rescue => (["重新发送", "重新傳送", "Send"], ["重新发送", "重新傳送", "Send"]),
                Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => (["重新安装", "重新安裝", "Install"], ["重新刷入", "重新刷入", "Flash"]),
            };
            let (install, flash) = (l.pick(install), l.pick(flash));
            let warning = match damage {
                Some(Damage::System) => tr!(
                    l,
                    "当前系统已被部分覆盖，重启后将无法进入系统。请{install}完整更新包，安装完成后再重启。",
                    "目前的系統已被部分覆寫，重新啟動後將無法進入系統。請{install}完整更新檔，安裝完成後再重新啟動。",
                    "The current system was partly written over, so a reboot starts no system. {install} the full OTA package again, and reboot once it's installed."
                ),
                Some(Damage::Partition(p)) => tr!(
                    l,
                    "{p} 仅写入了部分内容，重启后可能无法正常启动。请{flash}该镜像，刷入完成后再重启。",
                    "{p} 僅寫入了部分內容，重新啟動後可能無法正常啟動。請{flash}這個映像檔，刷入完成後再重新啟動。",
                    "{p} was only partly written, so the phone may not start properly after a reboot. {flash} the image again, and reboot once it's flashed."
                ),
                Some(Damage::Script) => tr!(
                    l,
                    "本机的分区可能已被部分改写，重启后可能无法正常启动。请{install}该刷机包，安装完成后再重启。",
                    "本機的分割區可能已被部分改寫，重新啟動後可能無法正常啟動。請{install}這個刷機包，安裝完成後再重新啟動。",
                    "This phone's partitions may have been partly rewritten, so it may not start properly after a reboot. {install} the zip again, and reboot once it's installed."
                ),
                None => l.pick(["当前系统未受影响。", "目前的系統未受影響。", "The current system is untouched."]).into(),
            };
            let mut y = widgets::footnote(c, t, f, next.bottom, &warning);
            if let Some(d) = detail {
                y = widgets::footnote(c, t, f, y, &d);
            }
            let y = reflashed(c, cx, y);
            let y = afterwards_of(c, cx, y, false);
            script_output(c, cx, y, rows)
        }
    }
}

/// What comes after an install, where it is not the user's to choose:
/// rescue waits for the computer's next command, and a package the system
/// asked for, once in, has the phone leave on its own.
fn afterwards_of(c: &mut Canvas, cx: &Ctx, y: f32, installed: bool) -> f32 {
    let l = cx.state.lang;
    let leaving = match cx.state.asked.shutdown_after {
        true => ["即将自动关机。", "即將自動關機。", "Powering off shortly."],
        false => ["即将自动重启。", "即將自動重新啟動。", "Rebooting shortly."],
    };
    match cx.state.source {
        Source::Rescue => widgets::footnote(c, cx.t, cx.f, y, l.pick(MORE_COMMANDS)),
        Source::Asked { .. } if installed => widgets::footnote(c, cx.t, cx.f, y, l.pick(leaving)),
        _ => y,
    }
}

/// What the wipe page lists of [`WIPES`] beyond everything, under the
/// partition each is in.
const WIPE_GROUPS: [(Text, &[Wipe]); 2] = [
    (DATA_PART, &[Wipe::AppData, Wipe::Media, Wipe::AppCaches, Wipe::SystemCaches, Wipe::Art, Wipe::Logs, Wipe::Updates]),
    (["cache 分区", "cache 分割區", "cache partition"], &[Wipe::Cache]),
];

/// What a factory reset erases, as the wipe page and an update that asks
/// for one tell it; in English as it goes on from what leads to it, its
/// first letter small.
fn factory_reset(s: &State) -> String {
    let also = s.lang.pick(match s.cache_partition {
        true => ["metadata 和 cache 分区", "metadata 和 cache 分割區", "the metadata and cache partitions"],
        false => ["metadata 分区", "metadata 分割區", "the metadata partition"],
    });
    tr!(
        s.lang,
        "清空 data 分区，包括应用、账户、设置和内部存储，并格式化 {also}",
        "清空 data 分割區，包括應用程式、帳戶、設定和內部儲存空間，並格式化 {also}",
        "empties the data partition, apps, accounts, settings and internal storage included, and formats {also}"
    )
}

/// A choice of the wipe page as it lists it: its title, and exactly what it
/// erases.
fn wipe_row(w: Wipe, s: &State) -> (&'static str, String) {
    let l = s.lang;
    let (title, what): (Text, String) = match w {
        Wipe::All => (["全部数据", "全部資料", "All data"], capitalized(l, &factory_reset(s))),
        Wipe::AppData => (
            ["应用数据", "應用程式資料", "App data"],
            l.pick([
                "/data/data、/data/user、/data/user_de，以及内部存储中的 Android/data、Android/media，应用将恢复到初始状态",
                "/data/data、/data/user、/data/user_de，以及內部儲存空間中的 Android/data、Android/media，應用程式將恢復到初始狀態",
                "/data/data, /data/user, /data/user_de, and Android/data and Android/media in internal storage. Apps go back to how they were first installed",
            ])
            .into(),
        ),
        Wipe::Media => (
            ["内部存储", "內部儲存空間", "Internal storage"],
            l.pick(["/data/media，即 /sdcard 中的照片、视频和文件", "/data/media，即 /sdcard 中的相片、影片和檔案", "/data/media: the photos, videos and files in /sdcard"]).into(),
        ),
        Wipe::AppCaches => (
            ["应用缓存", "應用程式快取", "App caches"],
            l.pick([
                "各应用数据中的 cache、code_cache，以及 Android/data 中的 cache，应用将按需重新生成",
                "各應用程式資料中的 cache、code_cache，以及 Android/data 中的 cache，應用程式將視需要重新產生",
                "cache and code_cache in each app's data, and cache in Android/data. Apps make them again as they need them",
            ])
            .into(),
        ),
        Wipe::SystemCaches => (
            ["系统缓存", "系統快取", "System caches"],
            l.pick([
                "/data/system/package_cache、/data/resource-cache、/data/cache，下次开机时重新生成",
                "/data/system/package_cache、/data/resource-cache、/data/cache，下次開機時重新產生",
                "/data/system/package_cache, /data/resource-cache and /data/cache, made again at the next start",
            ])
            .into(),
        ),
        Wipe::Art => (
            ["ART 缓存", "ART 快取", "ART cache"],
            l.pick([
                "/data/dalvik-cache、/data/misc/apexdata/com.android.art/dalvik-cache、/data/ota，下次开机时重新编译，启动时间较长",
                "/data/dalvik-cache、/data/misc/apexdata/com.android.art/dalvik-cache、/data/ota，下次開機時重新編譯，啟動時間較長",
                "/data/dalvik-cache, /data/misc/apexdata/com.android.art/dalvik-cache and /data/ota, compiled again at the next start, which then takes longer",
            ])
            .into(),
        ),
        Wipe::Logs => {
            let own = match (l, s.phone_logs) {
                (_, 0) => String::new(),
                (Lang::En, 1) => ", and this phone's own log folder".into(),
                (_, n) => tr!(l, "，以及本机的 {n} 个日志文件夹", "，以及本機的 {n} 個記錄資料夾", ", and this phone's own {n} log folders"),
            };
            let what = tr!(
                l,
                "/data/tombstones、/data/anr、/data/system/dropbox、/data/misc/logd 等系统日志和崩溃记录，各用户的错误报告{own}",
                "/data/tombstones、/data/anr、/data/system/dropbox、/data/misc/logd 等系統記錄和當機記錄，各使用者的錯誤報告{own}",
                "System logs and crash records, such as /data/tombstones, /data/anr, /data/system/dropbox and /data/misc/logd, each user's bug reports{own}"
            );
            (["日志", "記錄", "Logs"], what)
        }
        Wipe::Updates => (
            ["下载的更新包", "下載的更新檔", "Downloaded updates"],
            l.pick(["/data/ota_package 中的系统更新包", "/data/ota_package 中的系統更新檔", "System update packages in /data/ota_package"]).into(),
        ),
        Wipe::Cache => (
            ["全部内容", "全部內容", "Everything"],
            l.pick(["格式化 cache 分区，保留 recovery 文件夹中的日志", "格式化 cache 分割區，保留 recovery 資料夾中的記錄", "Formats the cache partition, keeping the logs in its recovery folder"]).into(),
        ),
    };
    (l.pick(title), what)
}

/// What to erase: everything, as a factory reset does, or parts of it on
/// their own, the cache partition among them where the phone has one; then
/// a swipe erases what is chosen.
pub fn wipe(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let chosen = |w: Wipe| s.wipe[WIPES.iter().position(|&x| x == w).expect("every choice is one of WIPES")];
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["恢复出厂设置", "恢復原廠設定", "Factory reset"]));
    let (name, sub) = wipe_row(Wipe::All, s);
    let all = widgets::list(&mut c, t, f, y, &[Row::new(name).subtitle(&sub).trail(Trail::Check(chosen(Wipe::All)))]);
    out.extend(hits(&all.rows, &[Action::WipeItem(Wipe::All)]));
    let note = l.pick([
        "系统和已安装的更新不受影响，清除后需重新设置手机。",
        "系統和已安裝的更新不受影響，清除後需重新設定手機。",
        "The system and installed updates stay. The phone has to be set up again afterwards.",
    ]);
    let mut y = widgets::footnote(&mut c, t, f, all.bottom, note);
    for (title, items) in WIPE_GROUPS {
        let shown: Vec<Wipe> = items.iter().copied().filter(|&w| w != Wipe::Cache || s.cache_partition).collect();
        if shown.is_empty() {
            continue;
        }
        let below = widgets::section(&mut c, t, f, y, l.pick(title));
        let texts: Vec<(&str, String)> = shown.iter().map(|&w| wipe_row(w, s)).collect();
        let rows: Vec<Row> =
            shown.iter().zip(&texts).map(|(&w, (name, sub))| Row::new(name).subtitle(sub).trail(Trail::Check(chosen(w)))).collect();
        let list = widgets::list(&mut c, t, f, below, &rows);
        out.extend(hits(&list.rows, &shown.iter().map(|&w| Action::WipeItem(w)).collect::<Vec<_>>()));
        y = list.bottom;
    }
    let note = l.pick([
        "清除应用数据和应用缓存前，需先完成「解密」。清除后无法恢复。",
        "清除應用程式資料和應用程式快取前，需先完成「解密」。清除後無法復原。",
        "Wiping app data or app caches needs the data partition decrypted first. What is wiped can't be brought back.",
    ]);
    let y = widgets::footnote(&mut c, t, f, y, note);
    let (label, danger) = confirm_label(Confirm::Wipe, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Wipe) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(["清除数据", "清除資料", "Wipe data"]), top)
}

/// 输入框里该画的东西：已经落下的字 + 正在打的拼音。26 键按下去就是字母，
/// 攒着的那一串直接接上。
fn with_preview(s: &State, text: &str) -> String {
    if !s.pinyin.on || s.pinyin.composing.is_empty() {
        return text.to_string();
    }
    format!("{text}{}", s.pinyin.composing)
}

/// 这一页此刻有没有那条候选：中文、收中文、而且正在打拼音。
fn pinyin_bar_shown(s: &State) -> bool {
    s.pinyin.on && !s.pinyin.composing.is_empty()
}

/// A page laid out above the on-screen keyboard: where the keyboard's top
/// is, and its keys as touch targets.
fn keyboard_hits(c: &mut Canvas, t: &Theme, f: &Fonts, s: &State, enter: &str, chinese: bool, out: &mut Vec<Hit>) -> f32 {
    let top = t.m.h as f32 - widgets::keyboard_height(t);
    // 键盘是哪一套：默认看这一页收不收汉字（`chinese`），但用户按过「中/英」
    // 就按他切的那一套（2026-11-24）。中文是 26 键大写，英文是 26 键小写。
    // `chinese` 这个参数留着是给调用点少改动，真正说了算的是输入法在不在中文。
    // 只有收汉字的页面才摆中文那一套（大写 26 键）：密码、网址这些页面始终
    // 小写，字母直接落（2026-11-24）。
    let chinese_here = s.pinyin.on && chinese;
    let layer = match (chinese_here, s.layer) {
        (true, widgets::Layer::Letters | widgets::Layer::Shifted) => widgets::Layer::Shifted,
        (false, widgets::Layer::Shifted) => widgets::Layer::Letters,
        (_, layer) => layer,
    };
    // 拼音条只在正在打拼音时出现：条上是候选，打的字母在输入框里。
    let page = match pinyin_bar_shown(s) {
        true => s.pinyin_page(),
        false => Vec::new(),
    };
    let (_, pages) = s.pinyin_view();
    let bar = match page.is_empty() {
        true => None,
        false => Some(widgets::PinyinBar { page: &page, at: s.pinyin.page.min(pages - 1), pages }),
    };
    // 中/英 那一枚要看「这一页收不收中文」，与条在不在无关：英文模式下条不画，
    // 键还在，手指才切得回中文。
    let keys = widgets::keyboard(c, t, f, top, layer, s.key_held(), enter, bar, chinese_here);
    // 键盘与候选条压在列表上面：`Frame::hit` 从后往前找，所以它们排在最后。
    // （2026-11-23 真机：候选条那三格要先答，不然点「备份」会把底下的模块页
    // 打开 —— 顺序反了就是这样。）
    out.extend(keys.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
    top
}

/// Finding a feature or partition: the field where the title would be, the
/// button that leaves the search beside it, what it finds, and the
/// keyboard.
pub fn search(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = page(cx);
    let mut out = Vec::new();
    let (at, cancel) = widgets::search_layout(t);
    // 进来找什么，框里就提示什么：找模块、找应用，还是找功能。
    let hint = match s.search_scope {
        SearchScope::Apps => l.pick(["搜索应用", "搜尋應用程式", "Search apps"]),
        SearchScope::Modules => l.pick(["搜索模块", "搜尋模組", "Search modules"]),
        SearchScope::Files => l.pick(["搜索文件", "搜尋檔案", "Search files"]),
        SearchScope::Features => l.pick(SEARCH_HINT),
    };
    let typing = with_preview(s, &s.search);
    let field = widgets::search_field(&mut c, t, f, at.x, at.w, at.y + at.h / 2.0, &typing, hint);
    widgets::cancel_button(&mut c, t, cancel);
    out.push(Hit { rect: cancel, action: Action::CloseSearch });
    let top = field.field.y + field.field.h + m.dp(12.0);
    let kb_top = m.h as f32 - widgets::keyboard_height(t);
    // 找什么由进来的地方说：整机的功能，或者某一页里的应用、模块。
    let query = s.search.trim().to_lowercase();
    // 空表放在外面：行借的是它们里面的字符串，得比 rows 活得久。
    let no_apps: Vec<crate::app::dataapp::DataApp> = Vec::new();
    // 行借的是它里面的字，得比 rows 活得久；只在没找到东西那一支里赋值。
    let nothing_found: String;
    let no_modules: Vec<crate::app::ModuleRow> = Vec::new();
    let no_files: Vec<crate::app::FileRow> = Vec::new();
    // 每行的大小得比 rows 活得久：先算好，再让行借它。
    let file_sizes: Vec<String>;
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    match s.search_scope {
        SearchScope::Apps => {
            let apps = match &s.apps {
                Some(Ok(list)) => list,
                _ => &no_apps,
            };
            for (k, a) in apps.iter().enumerate() {
                if !query.is_empty() && !(a.label.to_lowercase().contains(&query) || a.package.to_lowercase().contains(&query)) {
                    continue;
                }
                let mut row = Row::new(&a.label).icon(PURPLE, Icon::Layers);
                row.sub = Some(&a.package);
                rows.push(row);
                acts.push(Action::DataAppOpen(k));
            }
        }
        SearchScope::Files => {
            let dir = match &s.files.entries {
                Ok(list) => list,
                Err(_) => &no_files,
            };
            file_sizes = dir.iter().map(|e| fmt::bytes(e.size)).collect();
            for (k, e) in dir.iter().enumerate() {
                if !query.is_empty() && !e.name.to_lowercase().contains(&query) {
                    continue;
                }
                let folder = matches!(e.kind, FileKind::Folder { .. } | FileKind::Link { folder: true, .. });
                let icon = if folder { Icon::Folder } else { Icon::Log };
                let mut row = Row::new(&e.name).icon(if folder { BLUE } else { ORANGE }, icon);
                if !folder {
                    row.sub = Some(&file_sizes[k]);
                }
                rows.push(row);
                acts.push(Action::SearchFileOpen(k));
            }
        }
        SearchScope::Modules => {
            let mods = match &s.modules {
                Some(Ok(list)) => list,
                _ => &no_modules,
            };
            for (k, m) in mods.iter().enumerate() {
                if !query.is_empty() && !(m.name.to_lowercase().contains(&query) || m.id.to_lowercase().contains(&query)) {
                    continue;
                }
                rows.push(Row::new(&m.name).icon(BLUE, Icon::Layers).trail(Trail::Switch(m.enabled)));
                acts.push(Action::ModuleOpen(k));
            }
        }
        SearchScope::Features => {
            if query.is_empty() {
                let how = l.pick([
                    "输入功能或分区名称，支持拼音全拼和首字母，例如输入 bf 可找到「备份」。",
                    "輸入功能或分割區名稱，支援英文和漢語拼音的全拼與首字母，例如輸入 bf 可找到「備份」。",
                    "Type the name of a feature or partition, such as backup to find Backup.",
                ]);
                widgets::footnote(&mut c, t, f, top, how);
            } else {
                let found = search::find_on(&s.search, l, |screen| s.offers(screen));
                if found.is_empty() {
                    let typed = s.search.trim();
                    nothing_found = tr!(l, "未找到\"{typed}\"", "找不到「{typed}」", "Nothing found for “{typed}”");
                    rows.push(Row::new(&nothing_found).trail(Trail::None));
                } else {
                    for e in &found {
                        rows.push(Row::new(l.pick(e.title)).subtitle(l.pick(e.place)));
                        acts.push(Action::Open(e.screen));
                    }
                }
            }
        }
    }
    // 没输入时先说一句这页有多少东西可找。
    if query.is_empty() && s.search_scope != SearchScope::Features {
        let how = match (s.search_scope, &s.apps, &s.modules) {
            (SearchScope::Apps, Some(Ok(list)), _) => {
                let n = list.len();
                tr!(l, "共 {n} 个应用，输入名称或包名筛选", "共 {n} 個應用程式，輸入名稱或包名篩選", "{n} apps; type a name or package")
            }
            (SearchScope::Modules, _, Some(Ok(list))) => {
                let n = list.len();
                tr!(l, "共 {n} 个模块，输入名称或目录名筛选", "共 {n} 個模組，輸入名稱或目錄名篩選", "{n} modules; type a name or folder")
            }
            _ => l.pick(["输入名称筛选", "輸入名稱篩選", "Type a name to narrow it"]).to_string(),
        };
        widgets::footnote(&mut c, t, f, top, &how);
    }
    if !rows.is_empty() {
        let list = widgets::list(&mut c, t, f, top, &rows);
        // 键盘（和中文时压在它上面的候选条）盖住的地方不接点击：候选条就在
        // 最后那一行上面，谁的命中排在后面谁赢，不如干脆让行让开（2026-11-23
        // 真机：点候选「备份」把底下的模块页打开了）。
        let covered = kb_top - if pinyin_bar_shown(s) { m.dp(widgets::PINYIN_BAR_DP) } else { 0.0 };
        for (&rect, action) in list.rows.iter().zip(&acts) {
            if rect.y + rect.h <= covered {
                out.push(Hit { rect, action: *action });
            }
        }
    }
    let find = l.pick(["搜索", "搜尋", "Search"]);
    // 搜索认的是拼音与首字母（`search.rs` 的 keys），汉字进去反而找不到：
    // 这一页的键盘不攒拼音（2026-11-21）。
    keyboard_hits(&mut c, t, f, s, find, true, &mut out);
    if let Some(rect) = field.clear {
        out.push(Hit { rect, action: Action::ClearSearch });
    }
    Frame { back: false, header: false, caret: Some(field.caret), ..Frame::new(t, c, out, find, 0.0) }
}

/// Backing up: what to take, where it goes, and how.
pub fn backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["备份内容", "備份內容", "Contents"]));
    let system = fmt::snapshot_backup_note(l, &i.snapshot).unwrap_or(l.pick(["当前槽位在 super 中的分区", "目前槽位在 super 中的分割區", "The current slot's partitions in super"]));
    let data = l.pick(match s.data == Data::Open {
        true => ["应用、账户和设置，不包括内部存储、锁屏密码和指纹", "應用程式、帳戶和設定，不包括內部儲存空間、螢幕鎖定密碼和指紋", "Apps, accounts and settings, without internal storage, the screen lock or fingerprints"],
        false => [
            "应用、账户和设置，不包括内部存储、锁屏密码和指纹，需先完成「解密」",
            "應用程式、帳戶和設定，不包括內部儲存空間、螢幕鎖定密碼和指紋，需先完成「解密」",
            "Apps, accounts and settings, without internal storage, the screen lock or fingerprints. Decrypt first",
        ],
    });
    let boot = l.pick(["boot、init_boot、vendor_boot、dtbo、vbmeta 和 recovery", "boot、init_boot、vendor_boot、dtbo、vbmeta 和 recovery", "boot, init_boot, vendor_boot, dtbo, vbmeta and recovery"]);
    let modem = l.pick(["IMEI、基带设置和传感器校准数据", "IMEI、基頻設定和感應器校正資料", "IMEI, modem settings and sensor calibration"]);
    let subs = [boot, system, data, modem];
    let rows: Vec<Row> = BACKUP_PARTS
        .iter()
        .zip(subs)
        .enumerate()
        .map(|(k, (name, sub))| Row::new(l.pick(*name)).subtitle(sub).trail(Trail::Check(s.backup[k])))
        .collect();
    let parts = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&parts.rows, &[0, 1, 2, 3].map(Action::BackupPart)));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["存放位置", "存放位置", "Save to"]));
    let storage = match (s.data.storage_open(), i.storage.free.map(fmt::gb)) {
        (true, Some(free)) => tr!(l, "剩余 {free}", "剩餘 {free}", "{free} free"),
        (true, None) => l.pick(["无法读取剩余空间", "無法讀取剩餘空間", "Free space unknown"]).to_string(),
        (false, _) if s.data == Data::Erased => l.pick(DATA_ERASED).to_string(),
        (false, _) => l.pick(DECRYPT_NEEDED).to_string(),
    };
    let usb = l.pick(match &s.usb_drive {
        Some(UsbDrive::Shown) => ["已连接", "已連接", "Connected"],
        Some(UsbDrive::Unreadable(_)) => USB_DRIVE_UNREADABLE,
        None => USB_DRIVE_WANTED,
    });
    let places = widgets::list(&mut c, t, f, y, &[
        Row::new(place_name(l, Place::Storage)).subtitle(&storage).trail(Trail::Check(s.backup_to == Place::Storage)),
        Row::new(place_name(l, Place::UsbDrive)).subtitle(usb).trail(Trail::Check(s.backup_to == Place::UsbDrive)),
    ]);
    out.extend(hits(&places.rows, &[Action::BackupTo(Place::Storage), Action::BackupTo(Place::UsbDrive)]));
    let y = widgets::section(&mut c, t, f, places.bottom, l.pick(["选项", "選項", "Options"]));
    let options = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["压缩", "壓縮", "Compress"])).subtitle(l.pick(["备份更小，耗时更长", "備份更小，耗時更長", "Smaller, and slower to make"])).trail(Trail::Switch(s.compress)),
        Row::new(l.pick(["完成后校验", "完成後驗證", "Verify when done"])).subtitle(l.pick(["确认备份文件完整", "確認備份檔案完整", "Checks the backup's files are whole"])).trail(Trail::Switch(s.verify)),
    ]);
    out.extend(hits(&options.rows, &[Action::Toggle(Switch::Compress), Action::Toggle(Switch::Verify)]));
    let note = l.pick([
        "备份将保存在 Skyward/backups 文件夹中，以开始备份的时间命名。",
        "備份將儲存在 Skyward/backups 資料夾中，以開始備份的時間命名。",
        "Backups go in the Skyward/backups folder, named by when they began.",
    ]);
    let y = widgets::footnote(&mut c, t, f, options.bottom, note);
    let (label, danger) = confirm_label(Confirm::Backup, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Backup) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(["备份", "備份", "Backup"]), top)
}

/// The backups found, newest first, to restore one; those that cannot be
/// read are listed after them, to be deleted.
pub fn restore(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["备份", "備份", "Backups"]));
    let bottom = if s.backups.is_empty() {
        widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["暂无备份", "沒有備份", "No backups"])).trail(Trail::None)]).bottom
    } else {
        let texts: Vec<(String, String)> = s
            .backups
            .iter()
            .map(|b| match &b.held {
                Ok(h) => (fmt::datetime(l, h.made, s.zone()), format!("{} · {} · {}", fmt::parts(l, &h.parts), fmt::bytes(h.size), place_name(l, b.place))),
                Err(_) => (b.name.clone(), format!("{} · {}", l.pick(UNREADABLE), place_name(l, b.place))),
            })
            .collect();
        let rows: Vec<Row> = texts.iter().map(|(title, sub)| Row::new(title).subtitle(sub)).collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(list.rows.iter().enumerate().map(|(k, &rect)| Hit { rect, action: Action::OpenBackup(k) }));
        list.bottom
    };
    let note = l.pick([
        "将在内部存储和 U 盘中的 Skyward/backups 文件夹查找备份，查找内部存储需先完成「解密」。",
        "將在內部儲存空間和 USB 隨身碟中的 Skyward/backups 資料夾尋找備份，尋找內部儲存空間需先完成「解密」。",
        "Backups are looked for in the Skyward/backups folders of internal storage and the USB drive. Internal storage needs the data partition decrypted first.",
    ]);
    let bottom = widgets::footnote(&mut c, t, f, bottom, note);
    Frame::new(t, fit(c, t, bottom), out, l.pick(["恢复", "還原", "Restore"]), top)
}

/// One backup: when and where it was made, and which of its parts to
/// restore. One made on another phone is not restored here, and one that
/// cannot be read can only be deleted.
pub fn restore_backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let (entry, chosen) = s.restore.as_ref().expect("the restore page shows a backup");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let location = l.pick(["位置", "位置", "Location"]);
    let y = match &entry.held {
        Err(why) => {
            let facts = [(l.pick(["文件夹", "資料夾", "Folder"]), entry.name.clone()), (location, place_name(l, entry.place).to_string())];
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            let note = tr!(
                l,
                "无法读取该备份，备份可能未完成或已损坏。{why}",
                "無法讀取這份備份，備份可能未完成或已損壞。{why}",
                "This backup can't be read. It may not have finished, or it may be damaged. {why}"
            );
            widgets::footnote(&mut c, t, f, y, &note)
        }
        Ok(held) => {
            let mut facts = vec![
                (l.pick(["备份时间", "備份時間", "Made"]), fmt::datetime(l, held.made, s.zone())),
                (l.pick(["大小", "大小", "Size"]), fmt::bytes(held.size)),
                (location, place_name(l, entry.place).to_string()),
            ];
            if let Some(system) = &held.system {
                facts.push((l.pick(["系统版本", "系統版本", "System version"]), system.clone()));
            }
            if let Some(slot) = held.slot {
                facts.push((l.pick(["槽位", "槽位", "Slot"]), slot.letter().to_string()));
            }
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            if held.device != i.codename {
                let device = &held.device;
                let note = tr!(
                    l,
                    "该备份来自其他设备（{device}），无法恢复到本机。",
                    "這份備份來自其他裝置（{device}），無法還原到本機。",
                    "This backup is from another device ({device}) and can't be restored to this phone."
                );
                widgets::footnote(&mut c, t, f, y, &note)
            } else {
                let y = widgets::section(&mut c, t, f, y, l.pick(["恢复内容", "還原內容", "Contents"]));
                let kept: Vec<usize> = (0..BACKUP_PARTS.len()).filter(|&k| held.parts[k]).collect();
                let rows: Vec<Row> = kept.iter().map(|&k| Row::new(l.pick(BACKUP_PARTS[k])).trail(Trail::Check(chosen[k]))).collect();
                let list = widgets::list(&mut c, t, f, y, &rows);
                out.extend(list.rows.iter().zip(&kept).map(|(&rect, &k)| Hit { rect, action: Action::RestorePart(k) }));
                let note = l.pick(["恢复后，所选内容的现有数据将被备份覆盖。", "還原後，所選內容的現有資料將被備份覆寫。", "Restoring writes the backup over what the parts chosen hold now."]);
                let mut y = widgets::footnote(&mut c, t, f, list.bottom, note);
                if chosen[2] {
                    let note = l.pick([
                        "data 分区将恢复应用、账户和设置，内部存储、锁屏密码和指纹保持不变。恢复出厂设置后，无法再恢复此前的 data 分区备份。",
                        "data 分割區將還原應用程式、帳戶和設定，內部儲存空間、螢幕鎖定密碼和指紋保持不變。恢復原廠設定後，無法再還原此前的 data 分割區備份。",
                        "The data partition gets its apps, accounts and settings back, and internal storage, the screen lock and fingerprints stay as they are. After a factory reset, an earlier backup of the data partition can't be restored.",
                    ]);
                    y = widgets::footnote(&mut c, t, f, y, note);
                }
                // The boot partitions' vbmeta checks the system's: one of
                // another version without the other may not start.
                if let (Some(was), Some(now)) = (&held.system, &i.os_version)
                    && was != now
                    && chosen[0] != chosen[1]
                {
                    let warn = tr!(
                        l,
                        "该备份来自 {was}，本机当前为 {now}。仅恢复启动分区或系统分区中的一项，可能导致无法开机。",
                        "這份備份來自 {was}，本機目前為 {now}。僅還原啟動分割區或系統分割區中的一項，可能導致無法開機。",
                        "This backup is from {was}, and this phone runs {now}. Restoring only one of the boot and system partitions may keep the phone from starting."
                    );
                    y = widgets::footnote(&mut c, t, f, y, &warn);
                }
                let (label, danger) = confirm_label(Confirm::Restore, cx.state);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::Restore) });
                track.y + track.h
            }
        }
    };
    let delete = widgets::list(&mut c, t, f, y + t.m.dp(28.0), &[Row::new(l.pick(["删除备份", "刪除備份", "Delete backup"])).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&delete.rows, &[Action::AskDelete]));
    let mut bottom = delete.bottom;
    if s.asking_delete {
        let ask = l.pick(["删除该备份？删除后无法恢复。", "刪除這份備份？刪除後無法復原。", "Delete this backup? It can't be brought back."]);
        let y = widgets::footnote(&mut c, t, f, delete.bottom, ask);
        let (label, danger) = confirm_label(Confirm::DeleteBackup, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::DeleteBackup) });
        bottom = track.y + track.h;
    }
    Frame::new(t, fit(c, t, bottom), out, l.pick(["恢复备份", "還原備份", "Restore backup"]), top)
}

/// The share of its pass the part under way has done.
fn part_share(a: &BackupAt) -> f32 {
    if a.of == 0 {
        1.0
    } else {
        share(a.done, a.of).min(1.0)
    }
}

/// How far backing up or restoring has come over all, 0 to 1, and the
/// bytes of the parts the pass under way has gone through, the part under
/// way by its share. Each part counts by its bytes in each pass: a restore
/// checks all its parts first, and a backup checks them last.
fn job_progress(job: &BackupJob, a: &BackupAt) -> (f32, u64) {
    let total: u64 = job.plan.iter().map(|(_, n)| n).sum();
    let before: u64 = job.plan[..a.item].iter().map(|(_, n)| n).sum();
    let passed = before as f64 + f64::from(part_share(a)) * job.plan[a.item].1 as f64;
    let second = job.checks && a.checking != job.restore;
    let passes = if job.checks { 2.0 } else { 1.0 };
    let over = (if second { total as f64 } else { 0.0 } + passed) / (passes * total.max(1) as f64);
    (over as f32, passed as u64)
}

/// What a backup, or a restore, has done to a part, and does to it, in
/// each language.
fn job_words(restore: bool) -> (Text, Text) {
    match restore {
        true => (["已恢复", "已還原", "Restored"], ["正在恢复", "正在還原", "Restoring"]),
        false => (["已备份", "已備份", "Backed up"], ["正在备份", "正在備份", "Backing up"]),
    }
}

const VERIFIED: Text = ["已校验", "已驗證", "Verified"];
const VERIFYING: Text = ["正在校验", "正在驗證", "Verifying"];
const WAITING_TURN: Text = ["等待中", "等待中", "Waiting"];
const DONE: Text = ["完成", "完成", "Done"];

/// Where part `k` of a backup or restore stands, as its row says it.
fn part_state(l: Lang, job: &BackupJob, k: usize) -> String {
    use std::cmp::Ordering;
    let (did, doing) = job_words(job.restore);
    let ended = job.outcome.is_some();
    let a = match (&job.outcome, job.at) {
        (Some(BackupEnd::Saved(_)), _) => return l.pick(if job.checks { VERIFIED } else { did }).to_string(),
        (Some(BackupEnd::Restored { .. }), _) => return l.pick(did).to_string(),
        (_, None) => return l.pick(if ended { ["未开始", "未開始", "Not started"] } else { WAITING_TURN }).to_string(),
        (_, Some(a)) => a,
    };
    let state = match k.cmp(&a.item) {
        Ordering::Less if a.checking => VERIFIED,
        Ordering::Less => did,
        Ordering::Equal if ended => ["未完成", "未完成", "Unfinished"],
        Ordering::Equal => return format!("{} {}%", l.pick(if a.checking { VERIFYING } else { doing }), percent(part_share(&a))),
        // Ahead of the pass under way: a backup checks its parts once all
        // are written, and a restore writes them once all are checked.
        Ordering::Greater => match (a.checking, job.restore, ended) {
            (true, false, false) => did,
            (true, false, true) => ["未校验", "未驗證", "Not verified"],
            (false, false, false) | (true, true, false) => WAITING_TURN,
            (false, false, true) => ["未备份", "未備份", "Not backed up"],
            (false, true, false) => VERIFIED,
            (_, true, true) => ["未恢复", "未還原", "Not restored"],
        },
    };
    l.pick(state).to_string()
}

/// Backing up or restoring: how far it has come over all, each part and
/// where it stands, and a way to stop while stopping breaks nothing. Once
/// it has ended, how, and where to go.
pub fn backup_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.backup_job.as_ref().expect("the backup page shows work");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let (did, doing) = job_words(job.restore);
    let icon = if job.restore { Icon::Restore } else { Icon::Backup };
    let writing = job.at.is_some_and(|a| !a.checking);
    let mut y = match &job.outcome {
        None => {
            let (title, detail, done) = match job.at {
                Some(a) => {
                    let (over, passed) = job_progress(job, &a);
                    let total = fmt::bytes(job.plan.iter().map(|(_, n)| n).sum());
                    let (verb, past) = if a.checking { (l.pick(VERIFYING), l.pick(VERIFIED)) } else { (l.pick(doing), l.pick(did)) };
                    let (name, passed) = (&job.plan[a.item].0, fmt::bytes(passed));
                    (format!("{verb} {name}"), tr!(l, "{past} {passed}，共 {total}", "{past} {passed}，共 {total}", "{past} {passed} of {total}"), over)
                }
                None if job.restore => (
                    l.pick(["正在读取备份", "正在讀取備份", "Reading the backup"]).to_string(),
                    l.pick(["正在核对备份与本机", "正在核對備份與本機", "Checking the backup against this phone"]).to_string(),
                    0.0,
                ),
                None => (l.pick(GETTING_READY).to_string(), l.pick(["正在计算备份大小", "正在計算備份大小", "Working out the backup's size"]).to_string(), 0.0),
            };
            widgets::progress_card(&mut c, t, f, top, &title, &detail, Some(done))
        }
        Some(end) => {
            let gone = l.pick(if job.restore { ["未写入任何内容。", "未寫入任何內容。", "Nothing was written."] } else { ["未保存任何备份。", "未儲存任何備份。", "No backup was saved."] });
            let failed = l.pick(if job.restore { RESTORE_FAILED } else { ["备份失败", "備份失敗", "Backup failed"] });
            let (title, color, detail) = match end {
                BackupEnd::Saved(name) => {
                    let saved = match job.place {
                        Place::Storage => tr!(l, "已保存到内部存储的 Skyward/backups/{name}", "已儲存到內部儲存空間的 Skyward/backups/{name}", "Saved to Skyward/backups/{name} in internal storage"),
                        Place::UsbDrive => tr!(l, "已保存到 U 盘的 Skyward/backups/{name}", "已儲存到 USB 隨身碟的 Skyward/backups/{name}", "Saved to Skyward/backups/{name} on the USB drive"),
                    };
                    (l.pick(BACKUP_COMPLETE), GREEN, saved)
                }
                BackupEnd::Restored { .. } => {
                    (l.pick(RESTORE_COMPLETE), GREEN, l.pick(["重启手机后，将使用恢复的内容。", "重新啟動手機後，將使用還原的內容。", "The phone uses what was restored once it reboots."]).to_string())
                }
                BackupEnd::Stopped => (l.pick(["已停止", "已停止", "Stopped"]), SLATE, gone.to_string()),
                BackupEnd::Refused(r) => {
                    let title = l.pick(if job.restore { ["无法恢复", "無法還原", "Can't restore"] } else { ["无法备份", "無法備份", "Can't back up"] });
                    (title, ORANGE, fmt::backup_refusal(l, r, job.restore))
                }
                BackupEnd::Failed(_) if job.restore && writing => {
                    let detail = l.pick([
                        "部分内容已写入，重启后可能无法正常启动。请重新恢复，完成后再重启。",
                        "部分內容已寫入，重新啟動後可能無法正常啟動。請重新還原，完成後再重新啟動。",
                        "Part of it was written, so the phone may not start properly after a reboot. Restore again, and reboot once it's done.",
                    ]);
                    (failed, RED, detail.to_string())
                }
                BackupEnd::Failed(_) => (failed, RED, gone.to_string()),
            };
            widgets::list(&mut c, t, f, top, &[standing(title, color, icon, &detail)]).bottom
        }
    };
    let keep = l.pick(match job.place {
        Place::Storage => ["请不要关机。", "請不要關機。", "Don't power off"],
        Place::UsbDrive => ["请不要拔出 U 盘或关机。", "請不要拔出 USB 隨身碟或關機。", "Keep the USB drive in and don't power off"],
    });
    match &job.outcome {
        None => {
            let note = match (job.restore, job.at) {
                (false, _) => tr!(l, "备份完成前，{keep}停止后将删除未完成的备份。", "備份完成前，{keep}停止後將刪除未完成的備份。", "{keep} until the backup is done. Stopping deletes the unfinished backup."),
                (true, Some(a)) if !a.checking => tr!(l, "恢复完成前，{keep}", "還原完成前，{keep}", "{keep} until the restore is done."),
                (true, _) => l.pick(["校验备份完成前，不会写入任何内容。", "驗證備份完成前，不會寫入任何內容。", "Nothing is written until the backup is verified."]).to_string(),
            };
            y = widgets::footnote(&mut c, t, f, y, &note);
            // A restore that has begun writing goes on to its end.
            if !(job.restore && writing) {
                let label = l.pick(if job.stopping { ["正在停止", "正在停止", "Stopping"] } else { STOP });
                let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
                if !job.stopping {
                    out.extend(hits(&stop.rows, &[Action::StopBackup]));
                }
                y = stop.bottom;
            }
        }
        Some(end) => {
            match end {
                BackupEnd::Restored { unmade } if !unmade.is_empty() => {
                    let folders = unmade.join(l.pick(["、", "、", ", "]));
                    let note = tr!(
                        l,
                        "以下文件夹所属的用户已不存在，其中的内容未恢复：{folders}",
                        "以下資料夾所屬的使用者已不存在，其中的內容未還原：{folders}",
                        "The users these folders belonged to no longer exist, so what they hold wasn't restored: {folders}"
                    );
                    y = widgets::footnote(&mut c, t, f, y, &note);
                }
                BackupEnd::Failed(why) => y = widgets::footnote(&mut c, t, f, y, why),
                _ => {}
            }
            let (rows, acts): (Vec<Row>, Vec<Action>) = match end {
                BackupEnd::Restored { .. } => (
                    vec![Row::new(l.pick(["重启到系统", "重新啟動到系統", "Reboot to system"])).icon(ORANGE, Icon::Power), Row::new(l.pick(DONE)).trail(Trail::None)],
                    vec![Action::RebootTo(Target::System), Action::Back],
                ),
                _ => (vec![Row::new(l.pick(DONE)).trail(Trail::None)], vec![Action::Back]),
            };
            let next = widgets::list(&mut c, t, f, y + t.m.card_gap, &rows);
            out.extend(hits(&next.rows, &acts));
            y = next.bottom;
        }
    }
    // The parts come after what can be done, which a long list would push
    // off the screen.
    if !job.plan.is_empty() {
        let heading = l.pick(if job.restore { ["恢复内容", "還原內容", "Contents"] } else { ["备份内容", "備份內容", "Contents"] });
        let y0 = widgets::section(&mut c, t, f, y, heading);
        let states: Vec<String> = (0..job.plan.len()).map(|k| part_state(l, job, k)).collect();
        let rows: Vec<Row> = job.plan.iter().zip(&states).map(|((name, _), state)| Row::new(name).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y0, &rows).bottom;
    }
    let title = l.pick(match (&job.outcome, job.restore) {
        (None, restore) => job_words(restore).1,
        (Some(BackupEnd::Saved(_)), _) => BACKUP_COMPLETE,
        (Some(BackupEnd::Restored { .. }), _) => RESTORE_COMPLETE,
        (Some(_), false) => ["备份未完成", "備份未完成", "Backup incomplete"],
        (Some(_), true) => ["恢复未完成", "還原未完成", "Restore incomplete"],
    });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, fit(c, t, y), out, title, top) }
}

const BACKUP_COMPLETE: Text = ["备份完成", "備份完成", "Backup complete"];
const RESTORE_COMPLETE: Text = ["恢复完成", "還原完成", "Restore complete"];
const RESTORE_FAILED: Text = ["恢复失败", "還原失敗", "Restore failed"];
const STOP: Text = ["停止", "停止", "Stop"];

/// Tools for the experienced.
/// The folder the file manager shows, served to a browser on the same
/// network: the folder, the switch that serves it, and where it is.
pub fn serve(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = widgets::section(&mut c, t, f, m.content_top, l.pick(["传输", "傳輸", "Transfer"]));
    let folder = &s.files.path;
    let row = Row::new(l.pick(["文件夹", "資料夾", "Folder"]))
        .icon(BLUE, Icon::Folder)
        .subtitle(folder)
        .trail(Trail::Value(l.pick(["更改", "更改", "Change"])));
    let pick = widgets::list(&mut c, t, f, y, &[row]);
    out.extend(hits(&pick.rows, &[Action::ServePick]));
    y = widgets::section(&mut c, t, f, pick.bottom, l.pick(["服务", "服務", "Server"]));
    let on = matches!(s.serving, Serving::Running { .. });
    let mut line = Row::new(l.pick(["启动服务", "啟動服務", "Serve"]))
        .icon(if on { GREEN } else { SLATE }, Icon::Globe)
        .trail(Trail::Switch(on));
    if let Serving::Failed(why) = &s.serving {
        line = line.subtitle(why);
    }
    let run = widgets::list(&mut c, t, f, y, &[line]);
    out.extend(hits(&run.rows, &[Action::ServeToggle]));
    let below = run.bottom;
    let bottom;
    if let Serving::Running { at, served } = &s.serving {
        let said = tr!(l, "已应答 {served} 次请求", "已回應 {served} 次請求", "{served} requests answered");
        // 没有连上 WLAN 时端口其实是开着的，只是没有地址可以给别人：
        // 编一个 `http://<phone>:8080/` 出来，照着敲只会「找不到主机」
        // （2026-11-25）。
        let address = match at {
            Some(at) => at.clone(),
            None => l
                .pick([
                    "WLAN 未连接，连上后再看地址",
                    "Wi-Fi 未連線，連上後再看位址",
                    "Not joined to Wi-Fi; the address shows once it is",
                ])
                .to_owned(),
        };
        bottom = widgets::info_card(&mut c, t, f, below + m.card_gap, &[
            (l.pick(["地址", "位址", "Address"]), address),
            (l.pick(["请求", "請求", "Requests"]), said),
        ]);
    } else {
        let hint = l.pick([
            "启动服务后，在同一网络下用浏览器打开给出的地址，即可上传或下载这个文件夹里的文件。WLAN 需先连接。",
            "啟動服務後，在同一網路下用瀏覽器開啟給出的位址，即可上傳或下載這個資料夾裡的檔案。Wi-Fi 需先連線。",
            "Once the server is on, open the address it gives in a browser on the same network to move files in and out of this folder. Wi-Fi has to be joined first.",
        ]);
        bottom = widgets::footnote(&mut c, t, f, below, hint);
    }
    Frame::new(t, fit(c, t, bottom), out, l.pick(SERVE), m.content_top)
}

const SERVE: Text = ["无线传输", "無線傳輸", "Wireless transfer"];
const FETCH: Text = ["网络下载", "網路下載", "Download"];
const AVB: Text = ["AVB 校验", "AVB 驗證", "AVB verification"];

/// Verified boot's two checks in the slot the phone runs from, each turned by
/// its own row, and written once the swipe below is dragged to its end.
pub fn charge(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let ch = &s.charge;
    let b = cx.info.battery.as_ref();

    // What the charger is giving, and the most it has given this charge: the
    // two numbers that say how fast this charge is running.
    let now = match ch.now {
        Some(tenths) => fmt::watts(tenths),
        None if b.is_some_and(|b| b.plugged) => l.pick(["读取中", "讀取中", "Reading"]).to_owned(),
        None => l.pick(["未接充电器", "未接充電器", "No charger"]).to_owned(),
    };
    let most = match ch.most {
        Some(tenths) => fmt::watts(tenths),
        None => "\u{2014}".to_owned(),
    };
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["现在", "現在", "Now"]));
    let rows = [Row::new(&now).icon(BLUE, Icon::Battery).trail(Trail::Value(&most))];
    let list = widgets::list(&mut c, t, f, y, &rows);
    let mut y = list.bottom;

    // How fast the phone is let to charge, chosen before the charger and the
    // battery are told of: it is the one thing on this page that changes what
    // the phone does rather than what it says. The programs that settle it are
    // started as the recovery starts, so a step chosen here reaches the phone
    // at the next start, and the page says so rather than letting the steps
    // look as if they had taken at once.
    {
        let steps: [(ChargeMode, Text, Text); 3] = [
            (
                ChargeMode::Plain,
                ["默认", "預設", "Default"],
                [
                    "不启动充电服务，内核把电流限在它给未认证电池的上限，5W+。",
                    "不啟動充電服務，核心把電流限在它給未認證電池的上限，5W+。",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                ],
            ),
            (
                ChargeMode::Fast,
                ["快充", "快充", "Fast"],
                [
                    "启动小米充电服务，内核被告知电池已通过认证，10W+。",
                    "啟動小米充電服務，核心被告知電池已通過認證，10W+。",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                ],
            ),
            (
                ChargeMode::Fastest,
                ["超快充", "超快充", "Fastest"],
                [
                    "再启动小米的认证进程，充电器通过校验后可升压到 PPS 档，40W+。",
                    "再啟動小米的認證行程，充電器通過校驗後可升壓到 PPS 檔，40W+。",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                ],
            ),
        ];
        y = widgets::section(&mut c, t, f, y, l.pick(["充电档位", "充電檔位", "Charge step"]));        let rows: Vec<Row> = steps
            .iter()
            .map(|(mode, name, detail)| Row::new(l.pick(*name)).subtitle(l.pick(*detail)).trail(Trail::Check(s.charge_mode == *mode)))
            .collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        y = list.bottom;
        // The step already chosen is not a hit: tapping it would do nothing,
        // and offering it as something to tap would say otherwise.
        for (i, (mode, _, _)) in steps.iter().enumerate() {
            if s.charge_mode != *mode {
                out.extend(hits(&list.rows[i..=i], &[Action::Toggle(Switch::ChargeStep(*mode))]));
            }
        }
        y = widgets::footnote(
            &mut c,
            t,
            f,
            y + t.m.dp(8.0),
            l.pick([
                "档位在下次开机时生效：它决定 recovery 启动哪些小米服务，而这些服务只在启动时决定一次。",
                "檔位在下次開機時生效：它決定 recovery 啟動哪些小米服務，而這些服務只在啟動時決定一次。",
                "A step takes effect the next time the recovery starts: it decides which of Xiaomi's services the recovery starts, and those are decided once, as it starts.",
            ]),
        );
    }

    // The thermal guard, above the charger it watches: how warm the battery
    // may get before the charge is stepped down, and starting the recovery
    // again to make the step take. It is neither offered nor obeyed on the
    // plain step, where nothing a charge draws can warm the battery enough to
    // matter, so the slider is drawn dimmed and takes no touch.
    {
        let guarded = s.charge_mode.guarded();
        y = widgets::section(&mut c, t, f, y + t.m.dp(8.0), l.pick(["温控", "溫控", "Thermal guard"]));
        let span = f32::from(THERMAL_LIMIT_MOST - THERMAL_LIMIT_LEAST);
        let level = (f32::from(s.thermal_limit - THERMAL_LIMIT_LEAST) / span).clamp(0.0, 1.0);
        // The limit is written into the label, since the card has no value of
        // its own: the switch below it is the guard itself.
        let label = format!(
            "{} · {} °C",
            l.pick(["电池温度上限", "電池溫度上限", "Battery temperature limit"]),
            s.thermal_limit
        );
        let below = Row::new(l.pick(["超出后降低充电档位", "超出後降低充電檔位", "Step the charge down when past"]))
            .trail(Trail::Switch(s.thermal_guard && guarded));
        let card = widgets::slider_card(&mut c, t, f, y + t.m.dp(8.0), Caption::Thermometer, &label, level, Some(below));
        // The limit is the same number whatever the step, so it is drawn as it
        // stands even where it cannot be changed: a slider drawn at its left
        // end would say the limit had been moved there.
        //
        // A limit is only worth setting while the guard that reads it is on,
        // so the slider takes no touch where the switch is off, or where the
        // step starts nothing the guard has any say over. The switch itself is
        // still the guard's to turn, wherever a step is guarded at all.
        let lit = guarded && s.thermal_guard;
        if !lit {
            // Laid over the whole card, the way a finger's press is laid over a
            // row it holds: it says the card takes no touch without hiding what
            // it holds.
            let top = y + t.m.dp(8.0);
            c.fill_round_rect(t.m.margin, top, t.m.w as f32 - t.m.margin * 2.0, card.bottom - top, t.m.card_radius, t.pal.row_press);
        } else {
            out.push(Hit { rect: card.track, action: Action::Slider(Slider::ThermalLimit) });
        }
        if guarded && let Some(r) = card.below {
            out.push(Hit { rect: r, action: Action::Toggle(Switch::ThermalGuard) });
        }
        y = card.bottom;
        y = widgets::footnote(
            &mut c,
            t,
            f,
            y + t.m.dp(8.0),
            if guarded {
                l.pick([
                    "开关和温度都是即时生效的：开启后温度才可调，调完就按新温度盯着，电池一到这个温度就把充电档位降一档，然后重启回 recovery 按新档位充电。默认档不启动任何服务，充电很慢，不会让电池过热，所以默认档下这一项不可调，也不生效。",
                    "開關和溫度都是即時生效的：開啟後溫度才可調，調完就按新溫度盯著，電池一到這個溫度就把充電檔位降一檔，然後重啟回 recovery 按新檔位充電。預設檔不啟動任何服務，充電很慢，不會讓電池過熱，所以預設檔下這一項不可調，也不生效。",
                    "The switch and the temperature both take at once: the limit may be set only while the guard is on, and it is watched as it is set, so a battery that reaches it has the charge stepped down there and then, starting the recovery again to charge at the new step. The default step starts no service at all, so its charge is slow enough not to overheat the battery: the guard is neither offered nor obeyed there.",
                ])
            } else {
                l.pick([
                    "默认档不启动任何服务，充电很慢，不会让电池过热，所以这一项不可调，也不生效。",
                    "預設檔不啟動任何服務，充電很慢，不會讓電池過熱，所以這一項不可調，也不生效。",
                    "The default step starts no service at all, so its charge is slow enough not to overheat the battery: this is neither offered nor obeyed there.",
                ])
            },
        );
    }

    // What the charger negotiated, and what it says it can give. These are
    // the kernel's own words, left as it writes them: a protocol name is a
    // name, and translating it would only make it harder to look up.
    let mut charger: Vec<(&str, String)> = Vec::new();
    if let Some(protocol) = &ch.protocol {
        charger.push((l.pick(["协议", "協定", "Protocol"]), protocol.clone()));
    }
    if let Some(rated) = ch.rated {
        charger.push((l.pick(["充电器标称", "充電器標稱", "Charger rated"]), fmt::watts(rated * 10)));
    }
    if let Some(uv) = ch.volts {
        charger.push((l.pick(["输入电压", "輸入電壓", "Input voltage"]), fmt::volts(uv)));
    }
    if let Some(ua) = ch.amps {
        charger.push((l.pick(["输入电流", "輸入電流", "Input current"]), fmt::amps(ua)));
    }
    if !charger.is_empty() {
        y = widgets::section(&mut c, t, f, y, l.pick(["充电器", "充電器", "Charger"]));
        y = widgets::info_card(&mut c, t, f, y, &charger);
    }

    // The battery being filled: what it is at, how warm it is, and what the
    // charge is doing to it. The warmth matters more here than anywhere else
    // on the pages, since a fast charge is what makes it.
    if let Some(b) = b {
        let mut cells = vec![(l.pick(["电量", "電量", "Level"]), fmt::level(l, b))];
        if let Some(v) = b.temp {
            cells.push((l.pick(["温度", "溫度", "Temperature"]), fmt::celsius(v)));
        }
        if let Some(w) = b.watts() {
            cells.push((l.pick(["电池功率", "電池功率", "Battery power"]), fmt::watts((w * 10.0).round() as u32)));
        }
        if let Some(v) = b.voltage {
            cells.push((l.pick(["电池电压", "電池電壓", "Battery voltage"]), fmt::volts(v)));
        }
        if let Some(v) = fmt::wear(l, b) {
            cells.push((l.pick(["健康", "健康狀態", "Health"]), v));
        }
        if let Some(v) = b.cycles {
            cells.push((l.pick(["充电循环", "充電循環", "Charge cycles"]), tr!(l, "{v} 次", "{v} 次", "{v}")));
        }
        y = widgets::section(&mut c, t, f, y, l.pick(["电池", "電池", "Battery"]));
        y = widgets::info_card(&mut c, t, f, y, &cells);
    }

    // What the page cannot do, said plainly, since a page about charging that
    // stayed silent would look as if it could make the phone charge faster.
    // The step above settles which of the phone's own programs run; the
    // protocol itself is still the charger's and the kernel's to settle.
    let note = l.pick([
        "档位只决定 recovery 启动哪些小米服务；能到多少功率仍由充电器与内核协商，并不保证达到某个数值。",
        "檔位只決定 recovery 啟動哪些小米服務；能到多少功率仍由充電器與核心協商，並不保證達到某個數值。",
        "A step settles which of Xiaomi's services the recovery runs: how much power comes of it is still the charger's and the kernel's to settle, and no particular figure is promised.",
    ]);
    y = widgets::footnote(&mut c, t, f, y + t.m.dp(10.0), note);

    Frame::new(t, fit(c, t, y), out, l.pick(CHARGE), top)
}

pub fn avb(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    let (now_verity, now_verification) = match &s.avb {
        Some(Ok(now)) => *now,
        _ => (false, false),
    };
    // Before the swipe, the rows show what is about to be written.
    let (verity, verification) = s.avb_pending.unwrap_or((now_verity, now_verification));
    let rows = widgets::list(&mut c, t, f, top, &[
        Row::new(l.pick(["逐块校验（dm-verity）", "逐塊校驗（dm-verity）", "Block checks (dm-verity)"]))
            .icon(RED, Icon::Shield)
            .trail(Trail::Switch(verity)),
        Row::new(l.pick(["启动校验（verification）", "啟動驗證（verification）", "Boot checks (verification)"]))
            .icon(RED, Icon::Shield)
            .trail(Trail::Switch(verification)),
    ]);
    out.extend(hits(&rows.rows, &[Action::Avb(false), Action::Avb(true)]));
    let note = l.pick([
        "关闭后，改过的系统才能启动：启动校验关掉，bootloader 不再检查分区；逐块校验关掉，读到的块不再与签名比对。两者都要解锁 bootloader。",
        "關閉後，改過的系統才能啟動：關閉啟動驗證後 bootloader 不再檢查分割區；關閉逐塊校驗後讀到的區塊不再與簽章比對。兩者都需要解鎖 bootloader。",
        "Off, a changed system can start: with boot checks off the bootloader no longer checks the partitions, and with block checks off a block read is no longer compared with its signature. Both need an unlocked bootloader.",
    ]);
    let y = widgets::footnote(&mut c, t, f, rows.bottom, note);
    let y = match &s.avb_notice {
        Some(text) => widgets::footnote(&mut c, t, f, y, text),
        None => y,
    };
    let y = match &s.avb {
        Some(Err(why)) => widgets::footnote(&mut c, t, f, y, why),
        _ => y,
    };
    let (label, danger) = confirm_label(Confirm::Avb, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Avb) });
    let mut y = track.y + track.h;

    // vbmeta 备份：存一份当前的，或者把存过的写回去。
    y = widgets::section(&mut c, t, f, y + t.m.dp(10.0), l.pick(["vbmeta 备份", "vbmeta 備份", "vbmeta copies"]));
    let keep = widgets::list(&mut c, t, f, y, &[Row::new(l.pick([
        "备份当前槽的 vbmeta",
        "備份目前槽的 vbmeta",
        "Keep this slot's vbmeta",
    ]))
    .icon(BLUE, Icon::Backup)
    .subtitle(l.pick([
        "存到内部存储的 Skyward/vbmeta，写回前整份核对",
        "存到內部儲存空間的 Skyward/vbmeta，寫回前整份核對",
        "Into Skyward/vbmeta on internal storage; the whole file is checked before it goes back",
    ]))]);
    out.extend(hits(&keep.rows, &[Action::VbmetaDump]));
    // 两张卡片之间要留一道：`list` 的 bottom 是最后一张的底边，直接接上
    // 就成了上下贴死的一坨（2026-11-25，真机上看「备份当前槽的 vbmeta」
    // 与「A · 回滚 0」挤在一起）。
    let mut y = keep.bottom + t.m.card_gap;
    let chosen_at = s.vbmeta_choice;
    match &s.vbmeta_kept {
        Some(Ok(list)) if !list.is_empty() => {
            let titles: Vec<String> = list
                .iter()
                .map(|one| format!("{} · {} {}", one.slot.to_uppercase(), l.pick(["回滚", "回滾", "rollback"]), one.rollback_index))
                .collect();
            let subs: Vec<String> = list.iter().map(|one| format!("{} · {}", mb(one.bytes), one.name)).collect();
            let chosen = l.pick(["已选", "已選", "chosen"]);
            let listed: Vec<Row> = titles
                .iter()
                .zip(subs.iter())
                .enumerate()
                .map(|(k, (title, sub))| {
                    let picked = chosen_at == Some(k);
                    Row::new(title.as_str())
                        .icon(if picked { GREEN } else { SLATE }, Icon::Shield)
                        .trail(Trail::Text(if picked { chosen } else { "" }))
                        .subtitle(sub.as_str())
                })
                .collect();
            let table = widgets::list(&mut c, t, f, y, &listed);
            out.extend(hits(&table.rows, &(0..table.rows.len()).map(Action::VbmetaPick).collect::<Vec<_>>()));
            y = table.bottom;
        }
        Some(Ok(_)) => {
            y = widgets::footnote(&mut c, t, f, y, l.pick([
                "还没有备份。上面那条存一份，以后写回就靠它。",
                "還沒有備份。上面那條存一份，以後寫回就靠它。",
                "No copy kept yet: the row above makes one to put back later.",
            ]));
        }
        Some(Err(why)) => y = widgets::footnote(&mut c, t, f, y, why),
        None => {}
    }
    // 选中一份之后才出现写回的滑条。
    if s.vbmeta_choice.is_some() {
        let (label, danger) = confirm_label(Confirm::Vbmeta, s);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(20.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Vbmeta) });
        y = track.y + track.h;
    }
    if let Some(text) = &s.vbmeta_note {
        y = widgets::footnote(&mut c, t, f, y, text);
    }
    Frame::new(t, fit(c, t, y), out, l.pick(AVB), top)
}

/// A length of bytes in the units the pages speak.
fn mb(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1u64 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1u64 << 10) as f64),
        b => format!("{b} B"),
    }
}

/// A file from an address typed on the page's keyboard: the field and the
/// button, the way an address too long to type is taken from a file or from
/// another device through the page served here, and how the download stands.
pub fn fetch(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = page(cx);
    let mut out = Vec::new();
    let wide = m.w as f32 - m.margin * 2.0;
    let top = m.content_top - m.dp(66.0);
    let hint = l.pick(["粘贴或输入完整网址", "貼上或輸入完整網址", "Type or paste a full address"]);
    let field = widgets::search_field(&mut c, t, f, m.margin, wide, top + m.dp(24.0), &s.fetch_url, hint);
    let running = matches!(s.fetch, Fetch::Running { .. });
    let label = match running {
        true => l.pick(["停止下载", "停止下載", "Stop"]),
        false => l.pick(["开始下载", "開始下載", "Download"]),
    };
    let (bh, bw) = (m.dp(48.0), m.dp(220.0).min(wide));
    let mut y = field.field.y + field.field.h + m.dp(26.0);
    let (bx, by) = ((m.w as f32 - bw) / 2.0, y);
    c.fill_round_rect(bx, by, bw, bh, bh / 2.0, t.pal.accent);
    c.text_center(&f.heading, label, bx + bw / 2.0, by + bh / 2.0, m.row_title_px, Rgba::rgb(255, 255, 255));
    out.push(Hit {
        rect: Rect::new(bx, by, bw, bh),
        action: match running { true => Action::FetchStop, false => Action::FetchStart },
    });
    y = by + bh + m.dp(16.0);
    let from_file = l.pick([
        "从 /sdcard/Download/link.txt 读链接",
        "從 /sdcard/Download/link.txt 讀連結",
        "Take the link from /sdcard/Download/link.txt",
    ]);
    c.text_center(&f.body, from_file, m.w as f32 / 2.0, y + m.dp(18.0), m.sub_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(m.margin, y, wide, m.dp(36.0)), action: Action::FetchFromFile });
    y += m.dp(36.0) + m.dp(10.0);
    // A link too long for this keyboard can be typed on another device, which
    // opens a page this recovery serves; the code is where that page is.
    // 二维码那块位置是固定的：门户的端口晚一步才有，它一出现就把下面的字
    // 推下去，看着就是「文字上下跳」（2026-11-23 用户报的）。所以不论画不画，
    // 先占住同样的高度，说明和状态卡都在它下面固定的位置。
    let side = m.dp(168.0).min(wide);
    let qr_top = y;
    if let Some(at) = s.fetch_portal.clone() {
        let x0 = (m.w as f32 - side) / 2.0;
        let quiet = 2.0f32;
        if let Ok(code) = qrcodegen::QrCode::encode_text(&at, qrcodegen::QrCodeEcc::Medium) {
            let n = code.size();
            let cell = side / (n as f32 + quiet * 2.0);
            c.fill_round_rect(x0 - m.dp(12.0), y - m.dp(6.0), side + m.dp(24.0), side + m.dp(24.0), m.dp(14.0), Rgba::rgb(255, 255, 255));
            for row in 0..n {
                for col in 0..n {
                    if code.get_module(col, row) {
                        let px = x0 + (col as f32 + quiet) * cell;
                        let py = qr_top + m.dp(6.0) + (row as f32 + quiet) * cell;
                        c.fill_rect(px, py, cell + 0.5, cell + 0.5, Rgba::rgb(12, 12, 16));
                    }
                }
            }
        }
    }
    // 不论门户起来没有，这块高度都算进去：以前只在有门户时才推进 y，于是它
    // 一起来，下面的字整段往下挪一次（2026-11-23，用户录屏里还在跳）。
    y = qr_top + side + m.dp(20.0);
    // 这段说明无条件画：它的行数也是固定的，门户起没起来都不影响下面的位置。
    let how = l.pick([
        "用另一台手机扫这个码，在网页里输入网址，发回来就填进上面的框",
        "用另一台手機掃這個碼，在網頁裡輸入網址，傳回來就填進上面的框",
        "Scan this on another device, type the address there, and it fills the field above",
    ]);
    // 用 `footnote` 自己报的底边推进，不再按固定的 40 dp 猜 —— 折成两行时
    // 状态卡会压在字上。
    y = widgets::footnote(&mut c, t, f, y, how);
    match &s.fetch {
        Fetch::Idle => {
            let how = l.pick([
                "文件保存到 /sdcard/Download，名字取网址里最后一段。",
                "檔案儲存到 /sdcard/Download，名字取網址裡最後一段。",
                "Files are saved in /sdcard/Download, named after the last part of the address.",
            ]);
            widgets::footnote(&mut c, t, f, y, how);
        }
        Fetch::Running { done, total } => {
            let share = total.map(|t| (*done as f32 / (t.max(1) as f32)).clamp(0.0, 1.0));
            let detail = match (total, share) {
                (Some(total), Some(share)) => {
                    let (done_s, total_s, pct) = (mb(*done), mb(*total), percent(share));
                    tr!(l, "{done_s} / {total_s}（{pct}%）", "{done_s} / {total_s}（{pct}%）", "{done_s} of {total_s} ({pct}%)")
                }
                _ => mb(*done),
            };
            widgets::progress_card(&mut c, t, f, y, l.pick(["正在下载", "正在下載", "Downloading"]), &detail, share);
        }
        Fetch::Done { path, bytes } => {
            let (where_to, size_s) = (path.clone(), mb(*bytes));
            let detail = tr!(l, "已保存 {where_to}（{size_s}）", "已儲存 {where_to}（{size_s}）", "Saved {where_to} ({size_s})");
            widgets::info_card(&mut c, t, f, y, &[(l.pick(["完成", "完成", "Done"]), detail)]);
        }
        Fetch::Failed(why) => {
            widgets::info_card(&mut c, t, f, y, &[(l.pick(["失败", "失敗", "Failed"]), why.clone())]);
        }
        Fetch::Stopped => {
            widgets::info_card(&mut c, t, f, y, &[(l.pick(["已停止", "已停止", "Stopped"]), String::new())]);
        }
    }
    let title = l.pick(FETCH);
    let enter = match running {
        true => l.pick(["停止下载", "停止下載", "Stop"]),
        false => l.pick(["开始下载", "開始下載", "Download"]),
    };
    keyboard_hits(&mut c, t, f, s, enter, false, &mut out);
    if let Some(rect) = field.clear {
        out.push(Hit { rect, action: Action::ClearSearch });
    }
    Frame { header: false, caret: Some(field.caret), ..Frame::new(t, c, out, title, 0.0) }
}

/// Root 自己的页：KernelSU 与 Magisk 各一行看现状，下面两张卡直接在线写入。
pub fn root(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;

    let (kernel_su, magisk) = (kernel_su_saying(&s), magisk_saying(&s));
    let rows = [
        Row::new("KernelSU").icon(BLUE, Icon::Shield).trail(Trail::Value(&kernel_su)),
        Row::new("Magisk").icon(PURPLE, Icon::Layers).trail(Trail::Value(&magisk)),
        Row::new(l.pick(["模块管理", "模組管理", "Modules"])).icon(GREEN, Icon::Layers),
    ];
    let opens = [Action::Open(Screen::KernelSu), Action::Open(Screen::Magisk), Action::Open(Screen::Modules)];
    let first = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&first.rows, &opens));
    y = first.bottom + t.m.card_gap;

    y = widgets::section(&mut c, t, f, y, l.pick(["在线安装", "線上安裝", "Install online"]));
    let (ks_label, mg_label) = (
        tr!(l, "滑动以在线写入 KernelSU", "滑動以線上寫入 KernelSU", "Slide to write KernelSU online"),
        tr!(l, "滑动以在线写入 Magisk", "滑動以線上寫入 Magisk", "Slide to write Magisk online"),
    );
    let (ks_progress, ks_text) = write_state(s.kernel_su_job.as_ref(), s.root_bytes, l, &ks_label);
    let (mg_progress, mg_text) = write_state(s.magisk_job.as_ref(), s.root_bytes, l, &mg_label);
    for (label, progress, which) in [(ks_text, ks_progress, Confirm::KernelSu), (mg_text, mg_progress, Confirm::Magisk)] {
        let track = widgets::confirm(&mut c, t, f, y, &label, false, progress);
        out.push(Hit { rect: track, action: Action::Confirm(which) });
        y = track.y + track.h + t.m.card_gap;
    }

    let note = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "素材在线取：ksud、对应 KMI 的模块与管理器 APK 都从 GitHub 下载；写之前先把原厂 boot 备份到 Skyward/root。只写入当前槽。",
            "素材線上取：ksud、對應 KMI 的模組與管理器 APK 都從 GitHub 下載；寫之前先把原廠 boot 備份到 Skyward/root。只寫入目前槽。",
            "Payloads come from GitHub: ksud, the matching KMI module and the manager APK. The stock boot image is backed up to Skyward/root before anything is written. Only the current slot is written.",
        ]),
    );
    Frame::new(t, c.crop_height(fitted(t, note)), out, l.pick(["Root 管理", "Root 管理", "Root"]), t.m.content_top)
}

/// KernelSU 的页：现状、要写的 KMI，以及写进当前槽的 boot 镜像。
pub fn kernel_su(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;

    let status = s.kernel_su.as_ref().and_then(|r| r.as_ref().ok());
    let why = match s.kernel_su.as_ref() {
        Some(Err(why)) => Some(why.clone()),
        _ => None,
    };
    let mut rows = Vec::new();
    let version = status.and_then(|st| st.version.clone()).unwrap_or_else(|| l.pick(["无", "無", "None"]).into());
    let (slot_label, lock_label, latest_label) = (
        tr!(l, "当前槽位", "目前槽位", "Current slot"),
        tr!(l, "开机锁", "開機鎖", "Bootloader"),
        tr!(l, "在线最新", "線上最新", "Latest online"),
    );
    let latest = online_saying(l, s.latest_kernelsu.as_deref(), status.and_then(|st| st.version.as_deref()), !status.is_some_and(|st| st.installed));
    match status {
        Some(st) => {
            let state_row = if st.installed {
                l.pick(["已安装", "已安裝", "Installed"])
            } else {
                l.pick(["未安装", "未安裝", "Not installed"])
            };
            rows.push(Row::new(state_row).trail(Trail::Value(&version)));
            rows.push(Row::new(&latest_label).trail(Trail::Value(&latest)));
            if let Some(target) = &st.backup_target {
                rows.push(Row::new(l.pick(["已写入", "已寫入", "Written to"])).trail(Trail::Value(target)));
            }
            if let Some(slot) = &st.slot {
                rows.push(Row::new(&slot_label).trail(Trail::Value(slot)));
            }
            let locked = if st.unlocked {
                l.pick(["已解锁", "已解鎖", "Unlocked"])
            } else {
                l.pick(["已锁定", "已鎖定", "Locked"])
            };
            rows.push(Row::new(&lock_label).trail(Trail::Value(locked)));
        }
        None => rows.push(Row::new(l.pick(["读取失败", "讀取失敗", "Couldn't read"])).trail(Trail::None)),
    }
    let list = widgets::list(&mut c, t, f, y, &rows);
    y = list.bottom + t.m.card_gap;

    let said = match s.kernel_su_job.as_ref().map(|j| j.stage) {
        Some(_) if s.kernel_su_job.as_ref().is_some_and(|j| j.outcome.is_none()) => {
            l.pick(["正在写入 KernelSU…", "正在寫入 KernelSU…", "Writing KernelSU…"])
        }
        _ => l.pick(["写入 KernelSU", "寫入 KernelSU", "Write KernelSU"]),
    };
    let stage_note = widgets::footnote(&mut c, t, f, y, said);
    y = stage_note + t.m.card_gap;
    let idle = tr!(l, "滑动以写入 KernelSU", "滑動以寫入 KernelSU", "Slide to write KernelSU");
    let (progress, write_label) = write_state(s.kernel_su_job.as_ref(), s.root_bytes, l, &idle);
    let track = widgets::confirm(&mut c, t, f, y, &write_label, false, progress);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::KernelSu) });
    y = track.y + track.h + t.m.card_gap;

    if let Some(st) = status {
        y = widgets::section(&mut c, t, f, y, l.pick(["KMI", "KMI", "KMI"]));
        let kmis: Vec<Row> = st
            .kmis
            .iter()
            .enumerate()
            .map(|(k, row)| Row::new(&row.id).subtitle(&row.target).trail(Trail::Check(s.kernel_su_kmi == Some(k) || st.default == Some(k))))
            .collect();
        let picks: Vec<Action> = (0..kmis.len()).map(Action::KernelSuKmi).collect();
        let chosen = widgets::list(&mut c, t, f, y, &kmis);
        out.extend(hits(&chosen.rows, &picks));
        y = chosen.bottom + t.m.card_gap;
    }
    let note = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "默认按内核版本选择 KMI；如果本机需要别的 KMI，可以手动选择。只写入当前槽。",
            "預設按核心版本選擇 KMI；如果本機需要別的 KMI，可以手動選擇。只寫入目前槽。",
            "The KMI is picked by the kernel's version; choose another by hand where this phone needs it. Only the current slot is written.",
        ]),
    );
    y = note + t.m.card_gap;
    let online = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "从 GitHub 下载最新的 ksud、匹配的模块和管理器；需要网络。",
            "從 GitHub 下載最新的 ksud、符合的模組與管理器；需要網路。",
            "The newest ksud, the matching module and the manager are downloaded from GitHub; it needs the network.",
        ]),
    );
    y = online + t.m.card_gap;
    if let Some(why) = &why {
        let at = widgets::footnote(&mut c, t, f, y, why);
        y = at + t.m.card_gap;
    }

    // 只有本恢复模式真备份过原厂 boot，才有东西可恢复；这台机器的
    // KernelSU 若是别的工具装的（没有 kernelsu.conf），这里就说清楚，
    // 不摆一行点下去必然失败的「已恢复」（2026-11-26 真机）。
    let bottom = if status.is_some_and(|st| st.has_backup) {
        let target = st_target(status.and_then(|st| st.backup_target.clone()));
        let saved_label = format!("{} {target}", tr!(l, "恢复原厂", "恢復原廠", "Restore stock"));
        let saved = widgets::section(&mut c, t, f, y, &saved_label);
        let row = Row::new(l.pick(["已恢复", "已恢復", "Restored"])).icon(SLATE, Icon::Restore);
        let saved = widgets::list(&mut c, t, f, saved, &[row]);
        out.extend(hits(&saved.rows, &[Action::RestoreKernelSu]));
        saved.bottom
    } else {
        widgets::footnote(
            &mut c,
            t,
            f,
            y,
            l.pick([
                "这台机器的 KernelSU 不是本恢复模式装的，没有原厂 boot 备份可恢复。",
                "這台機器的 KernelSU 不是本復原模式裝的，沒有原廠 boot 備份可還原。",
                "KernelSU here was not installed by this recovery, so it has no stock boot backup to restore.",
            ]),
        )
    };
    Frame::new(t, c.crop_height(fitted(t, bottom)), out, l.pick(["KernelSU", "KernelSU", "KernelSU"]), t.m.content_top)
}

/// 写入卡该停在哪、写什么：滑到底开始后停在底并说「正在写入」；写完停在底说
/// 「已写入」；失败回到原位，让手指知道可以再滑一次（2026-11-26）。
fn write_state(job: Option<&KernelSuJob>, bytes: Option<(u64, Option<u64>, u64)>, l: Lang, idle: &str) -> (f32, String) {
    match job {
        Some(job) if job.outcome.is_none() => {
            let worked = match bytes {
                Some((done, Some(total), rate)) if total > 0 => format!("{}% · {}", done.saturating_mul(100) / total, per_second(rate)),
                Some((done, None, rate)) => format!("{} · {}", size_text(done), per_second(rate)),
                _ => tr!(l, "准备中…", "準備中…", "Preparing…"),
            };
            (1.0, format!("{} {}", tr!(l, "正在在线写入", "正在線上寫入", "Writing online"), worked))
        }
        Some(job) if let Some(KernelSuEnd::Failed(why)) = &job.outcome => {
            (0.0, format!("{} · {}", tr!(l, "未写入", "未寫入", "Not written"), clip_saying(why)))
        }
        Some(_) => (1.0, tr!(l, "已写入", "已寫入", "Written")),
        None => (0.0, idle.to_string()),
    }
}

/// 在线那一版怎么写在页面上：比装着的这一版新就写「可升级」，一样或旧写
/// 「已是最新」，装都没装就只写版本号（2026-11-26）。
fn online_saying(l: Lang, online: Option<&str>, installed: Option<&str>, not_installed: bool) -> String {
    let Some(online) = online else {
        return tr!(l, "读取中…", "讀取中…", "Reading…");
    };
    if not_installed {
        return online.to_string();
    }
    match is_newer(installed, Some(online)) {
        Some(true) => format!("{online} · {}", tr!(l, "可升级", "可升級", "Update available")),
        Some(false) => format!("{online} · {}", tr!(l, "已是最新", "已是最新", "Up to date")),
        None => online.to_string(),
    }
}

/// 每秒多少字节，写成人看得懂的样子。
fn per_second(rate: u64) -> String {
    if rate == 0 {
        "…".to_string()
    } else if rate >= 1 << 20 {
        format!("{:.1} MiB/s", rate as f64 / (1u64 << 20) as f64)
    } else {
        format!("{} KiB/s", rate / 1024)
    }
}

/// 一份文件多大，写成人看得懂的样子。
fn size_text(bytes: u64) -> String {
    if bytes >= 1 << 20 {
        format!("{:.1} MiB", bytes as f64 / (1u64 << 20) as f64)
    } else {
        format!("{} KiB", bytes / 1024)
    }
}

/// The KMI the KernelSU page writes: the list, each one checked or not.
pub fn kernel_su_kmi(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let y = t.m.content_top;
    let st = s.kernel_su.as_ref().and_then(|r| r.as_ref().ok());
    let rows: Vec<Row> = st
        .map(|st| {
            st.kmis
                .iter()
                .enumerate()
                .map(|(k, row)| Row::new(&row.id).subtitle(&row.target).trail(Trail::Check(s.kernel_su_kmi == Some(k) || st.default == Some(k))))
                .collect()
        })
        .unwrap_or_default();
    let picks: Vec<Action> = (0..rows.len()).map(Action::KernelSuKmi).collect();
    let list = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&list.rows, &picks));
    Frame::new(t, c.crop_height(fitted(t, list.bottom)), out, l.pick(["KMI", "KMI", "KMI"]), t.m.content_top)
}

/// Magisk 的页：现状与写进当前槽的 boot 镜像。
pub fn magisk(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;
    let st = s.magisk.as_ref().and_then(|r| r.as_ref().ok());
    let version = st.and_then(|st| st.version.clone()).unwrap_or_else(|| l.pick(["无", "無", "None"]).into());
    let (latest_label, latest) = (
        tr!(l, "在线最新", "線上最新", "Latest online"),
        online_saying(l, s.latest_magisk.as_deref(), st.and_then(|st| st.version.as_deref()), !st.is_some_and(|st| st.installed)),
    );
    let mut rows = Vec::new();
    match st {
        Some(st) => {
            let state_row = if st.installed {
                l.pick(["已安装", "已安裝", "Installed"])
            } else {
                l.pick(["未安装", "未安裝", "Not installed"])
            };
            rows.push(Row::new(state_row).trail(Trail::Value(&version)));
            rows.push(Row::new(&latest_label).trail(Trail::Value(&latest)));
            if let Some(target) = &st.backup_target {
                rows.push(Row::new(l.pick(["已写入", "已寫入", "Written to"])).trail(Trail::Value(target)));
            }
            rows.push(Row::new(l.pick(["目标分区", "目標分割區", "Target partition"])).trail(Trail::Value(&st.target)));
            if let Some(slot) = &st.slot {
                rows.push(Row::new(l.pick(["当前槽位", "目前槽位", "Current slot"])).trail(Trail::Value(slot)));
            }
        }
        None => rows.push(Row::new(l.pick(["读取失败", "讀取失敗", "Couldn't read"])).trail(Trail::None)),
    }
    let list = widgets::list(&mut c, t, f, y, &rows);
    y = list.bottom + t.m.card_gap;
    let note = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "Magisk 会修改当前槽的 boot 镜像。先做备份；只写入当前槽。",
            "Magisk 會修改目前槽的 boot 鏡像。先做備份；只寫入目前槽。",
            "Magisk patches the current slot's boot image. The stock image is backed up first; only the current slot is written.",
        ]),
    );
    y = note + t.m.card_gap;
    let online = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "从 GitHub 下载最新的 Magisk APK，补丁器就在里面；需要网络。",
            "從 GitHub 下載最新的 Magisk APK，補丁器就在裡面；需要網路。",
            "The newest Magisk APK is downloaded from GitHub, with its boot patcher inside; it needs the network.",
        ]),
    );
    y = online + t.m.card_gap;
    let idle = tr!(l, "滑动以写入 Magisk", "滑動以寫入 Magisk", "Slide to write Magisk");
    let (progress, write_label) = write_state(s.magisk_job.as_ref(), s.root_bytes, l, &idle);
    let track = widgets::confirm(&mut c, t, f, y, &write_label, false, progress);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Magisk) });
    y = track.y + track.h + t.m.card_gap;
    let bottom = if st.is_some_and(|st| st.has_backup) {
        let target = st_target(st.map(|s| s.target.clone()));
        let saved_label = format!("{} {target}", tr!(l, "恢复原厂", "恢復原廠", "Restore stock"));
        let saved = widgets::section(&mut c, t, f, y, &saved_label);
        let row = Row::new(l.pick(["已恢复", "已恢復", "Restored"])).icon(SLATE, Icon::Restore);
        let saved = widgets::list(&mut c, t, f, saved, &[row]);
        out.extend(hits(&saved.rows, &[Action::RestoreMagisk]));
        saved.bottom
    } else {
        widgets::footnote(
            &mut c,
            t,
            f,
            y,
            l.pick([
                "没有 Magisk 的原厂 boot 备份可恢复：本恢复模式还没在这里写过 Magisk。",
                "沒有 Magisk 的原廠 boot 備份可還原：本復原模式還沒在這裡寫過 Magisk。",
                "There is no stock boot backup to restore: this recovery has not written Magisk here.",
            ]),
        )
    };
    Frame::new(t, c.crop_height(fitted(t, bottom)), out, l.pick(["Magisk", "Magisk", "Magisk"]), t.m.content_top)
}

/// 版本串只留第一段：`3.3.1-56-gcfc60f8d (uapi: 6) (32736)` 后面那截是
/// 内核接口与编号，右栏放不下，会把左边的名字挤成「K...」。
fn short_version(text: &str) -> String {
    clip_saying(text.split_whitespace().next().unwrap_or(text))
}

/// What the KernelSU row of the root page says beside its name.
fn kernel_su_saying(s: &State) -> String {
    match s.kernel_su.as_ref() {
        Some(Ok(st)) => st.version.as_deref().map(short_version).unwrap_or_else(|| s.lang.pick(["未安装", "未安裝", "Not installed"]).to_owned()),
        Some(Err(why)) => clip_saying(why),
        None => s.lang.pick(["读取失败", "讀取失敗", "Couldn't read"]).to_owned(),
    }
}

/// 一行说法最多这么长，理由太长就截断，别把右侧那一栏撑满。
fn clip_saying(why: &str) -> String {
    let line = why.lines().next().unwrap_or(why).trim();
    let mut out: String = line.chars().take(28).collect();
    if out.chars().count() < line.chars().count() {
        out.push('…');
    }
    out
}

/// What the Magisk row of the root page says beside its name.
fn magisk_saying(s: &State) -> String {
    match s.magisk.as_ref() {
        Some(Ok(st)) => st.version.as_deref().map(short_version).unwrap_or_else(|| s.lang.pick(["未安装", "未安裝", "Not installed"]).to_owned()),
        Some(Err(why)) => clip_saying(why),
        None => s.lang.pick(["读取失败", "讀取失敗", "Couldn't read"]).to_owned(),
    }
}

/// The partition a backup belongs to, as the restore row's title names it.
fn st_target(target: Option<String>) -> String {
    match target {
        Some(t) => t,
        None => "\u{2014}".to_owned(),
    }
}

pub fn advanced(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    // 长页：条目多了以后一屏放不下，整页跟着手指滚。
    let mut c = long_page(cx);
    let top = t.m.content_top;
    // Every day's work first -- what is running, the data partition, the
    // partitions, the files' labels and a shell -- then what reaches the
    // network, then what checks or records the phone, and last what repairs
    // it or stops it.
    // 充电排在录屏下面：插着线时右侧给这一刻送进电池的功率，与充电页上那个
    // 数字是同一个；拔了线就只剩名字。
    let charging = match &cx.info.battery {
        Some(b) if b.plugged => Some(cx.state.charge.now.map_or_else(|| l.pick(["充电中", "充電中", "Charging"]).to_owned(), fmt::watts)),
        _ => None,
    };
    let charge = match &charging {
        Some(value) => Row::new(l.pick(CHARGE)).icon(GREEN, Icon::Battery).trail(Trail::Value(value)),
        None => Row::new(l.pick(CHARGE)).icon(SLATE, Icon::Battery),
    };
    let tools = widgets::list(&mut c, t, f, top, &[
        Row::new(l.pick(["任务", "任務", "Tasks"])).icon(BLUE, Icon::Install),
        Row::new(l.pick(["分区管理", "分割區管理", "Partitions"])).icon(SLATE, Icon::Mount),
        Row::new(l.pick(["录屏", "錄影", "Screen recording"])).icon(RED, Icon::Record),
        charge,
        Row::new(l.pick(DECRYPT)).icon(BLUE, Icon::Lock).trail(Trail::Value(fmt::data(l, &cx.state.data))),
        Row::new(l.pick(["挂载", "掛載", "Mount"])).icon(SLATE, Icon::Mount),
        Row::new(l.pick(TERMINAL)).icon(SLATE, Icon::Terminal),
        Row::new(l.pick(FILESYSTEMS)).icon(RED, Icon::Mount),
        Row::new(l.pick(SERVE)).icon(BLUE, Icon::Globe),
        Row::new(l.pick(FETCH)).icon(GREEN, Icon::Globe),
        Row::new(l.pick(AVB)).icon(RED, Icon::Shield),
        Row::new(l.pick(["Root 管理", "Root 管理", "Root"])).icon(PURPLE, Icon::Shield),
        Row::new(l.pick(UPDATE)).icon(GREEN, Icon::Layers).trail(Trail::Value(update_value(cx))),
        Row::new(l.pick(LOGS)).icon(ORANGE, Icon::Log),
        Row::new(l.pick(["U 盘管理", "USB 隨身碟管理", "USB drive"])).icon(PURPLE, Icon::UsbDrive),
        Row::new(l.pick(RELABEL)).icon(PURPLE, Icon::Shield),
        Row::new(l.pick(["关机", "關機", "Power off"])).icon(RED, Icon::Power),
    ]);
    let out = hits(&tools.rows, &[
        Action::Open(Screen::Tasks),
        Action::Open(Screen::Partitions),
        Action::Open(Screen::Record),
        Action::Open(Screen::Charge),
        Action::Open(Screen::Decrypt),
        Action::Open(Screen::Mount),
        Action::Open(Screen::Terminal),
        Action::Open(Screen::Filesystems),
        Action::Open(Screen::Serve),
        Action::Open(Screen::Fetch),
        Action::Open(Screen::Avb),
        Action::Open(Screen::Root),
        Action::Open(Screen::Update),
        Action::Open(Screen::Logs),
        Action::FilesOtg,
        Action::Open(Screen::Relabel),
        Action::Open(Screen::Shutdown),
    ]);
    Frame::new(t, fit(c, t, tools.bottom), out, l.pick(ADVANCED), top)
}

/// 任务页：安装进行到哪一步、安装脚本最后说了什么、系统更新在下什么。
/// 三条路各自通向能动手的那一页，这里只把状态摊开。
pub fn tasks(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    // 安装：进行中看百分比，停下时看它最后一句。
    let writing = s.install.writes();
    let percent = format!("{}%", (s.install_share * 100.0).round().clamp(0.0, 100.0) as u32);
    let install = if writing {
        percent
    } else {
        l.pick(["未开始", "未開始", "Not started"]).to_string()
    };
    let last = s
        .install_log
        .last()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| l.pick(["还没有开始安装", "還沒有開始安裝", "Nothing has been installed"]).to_string());
    // 安装脚本：脚本跑过才有话，最后一行就是它最后说的。
    let script = s
        .script_output
        .last()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| l.pick(["没有跑过安装脚本", "沒有執行過安裝腳本", "No install script has run"]).to_string());
    // 系统更新：下载到哪一步、写完了没有。
    let (fetch, fetch_note) = match &s.fetch {
        Fetch::Idle => (l.pick(["空闲", "閒置", "Idle"]).to_string(), String::new()),
        Fetch::Running { done, total } => match total {
            Some(total) if *total > 0 => (format!("{}%", (done * 100 / total).min(100)), format!("{} / {}", mb(*done), mb(*total))),
            _ => (mb(*done), String::new()),
        },
        Fetch::Done { bytes, .. } => (l.pick(["已下载", "已下載", "Downloaded"]).to_string(), mb(*bytes)),
        Fetch::Failed(why) => (l.pick(["失败", "失敗", "Failed"]).to_string(), why.clone()),
        Fetch::Stopped => (l.pick(["已停止", "已停止", "Stopped"]).to_string(), String::new()),
    };
    let rows = [
        Row::new(l.pick(["安装进度", "安裝進度", "Install"]))
            .icon(BLUE, Icon::Install)
            .trail(Trail::Value(&install))
            .subtitle(&last),
        Row::new(l.pick(["安装脚本最后一行", "安裝腳本最後一行", "The install script's last line"]))
            .icon(SLATE, Icon::Terminal)
            .trail(Trail::Arrow)
            .subtitle(&script),
        Row::new(l.pick(["系统下载", "系統下載", "System download"]))
            .icon(GREEN, Icon::Globe)
            .trail(Trail::Value(&fetch))
            .subtitle(&fetch_note),
    ];
    let list = widgets::list(&mut c, t, f, top, &rows);
    let out = hits(&list.rows, &[Action::Open(Screen::Install), Action::Open(Screen::Install), Action::Open(Screen::Update)]);
    Frame::new(t, c, out, l.pick(["任务", "任務", "Tasks"]), top)
}

/// 分区管理：整盘的三条路（导出、写回、文件系统）在上，分区表在下；
/// 分区管理：`by-name` 里的每个分区连同它的大小，几个整块的事同页给出：
/// 刷入镜像、文件系统、挂载，以及分区镜像的导出与写回。
pub fn partitions(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let rows = [
        Row::new(l.pick(["刷入分区镜像", "刷入分割區映像檔", "Flash a partition image"])).icon(BLUE, Icon::Install),
        Row::new(l.pick(FILESYSTEMS)).icon(PURPLE, Icon::Mount),
        Row::new(l.pick(["挂载管理", "掛載管理", "Mounts"])).icon(BLUE, Icon::Layers),
        Row::new(l.pick(IMAGE_BACKUP)).icon(GREEN, Icon::Backup),
    ];
    let list = widgets::list(&mut c, t, f, top, &rows);
    out.extend(hits(
        &list.rows,
        &[
            Action::Open(Screen::Choose(Place::Storage)),
            Action::Open(Screen::Filesystems),
            Action::Open(Screen::Mount),
            Action::Open(Screen::ImageBackup),
        ],
    ));
    let mut y = list.bottom + t.m.card_gap;
    y = widgets::section(&mut c, t, f, y, l.pick(["分区", "分割區", "Partitions"]));
    match &s.partitions {
        Some(Ok(list_rows)) => {
            let names: Vec<String> = list_rows.iter().map(|(name, _)| name.clone()).collect();
            let sizes: Vec<String> = list_rows.iter().map(|(_, size)| fmt::bytes(*size)).collect();
            let shown: Vec<Row> = names
                .iter()
                .zip(&sizes)
                .enumerate()
                .map(|(k, (name, size))| {
                    let row = Row::new(name).icon(PURPLE, Icon::Layers);
                    if s.dump_pick == Some(k) { row.trail(Trail::Check(true)) } else { row.trail(Trail::Text(size)) }
                })
                .collect();
            let pick = widgets::list(&mut c, t, f, y, &shown);
            out.extend(hits(&pick.rows, &(0..names.len()).map(Action::PickDump).collect::<Vec<_>>()));
            y = pick.bottom + t.m.card_gap;
            match s.dump_pick.and_then(|k| names.get(k).cloned()) {
                Some(name) => {
                    let note = tr!(l, "导出「{name}」到内部存储", "匯出「{name}」到內部儲存", "Export “{name}” to internal storage");
                    let one = widgets::list(&mut c, t, f, y, &[Row::new(&note).icon(ORANGE, Icon::Backup)]);
                    out.extend(hits(&one.rows, &[Action::DumpNow]));
                    y = one.bottom + t.m.card_gap;
                }
                None => {
                    y = widgets::footnote(&mut c, t, f, y, l.pick(["点一个分区选中它，再导出成镜像。", "點一個分割區選中它，再匯出成映像檔。", "Tap a partition to choose it, then export it as an image."]));
                }
            }
        }
        Some(Err(why)) => y = widgets::footnote(&mut c, t, f, y, why),
        None => y = widgets::footnote(&mut c, t, f, y, l.pick(["正在读取", "正在讀取", "Reading"])),
    }
    Frame::new(t, fit(c, t, y), out, l.pick(["分区管理", "分割區管理", "Partitions"]), top)
}

/// 分区镜像：把一个分区原样导出成 `.img`（旁边留一份 sha256），把已有的镜像
/// 整份核对后写回它自己的分区，或者删掉。
pub fn image_backup(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    // 导出：选一个分区，滑一下。
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["导出成镜像", "匯出成映像檔", "Export as an image"]));
    let y = match &s.partitions {
        None => widgets::footnote(&mut c, t, f, y, l.pick(["正在读取分区", "正在讀取分割區", "Reading partitions"])),
        Some(Err(error)) => widgets::footnote(&mut c, t, f, y, error),
        Some(Ok(parts)) => {
            let sizes: Vec<String> = parts.iter().map(|(name, size)| image_size_note(l, name, *size)).collect();
            let rows: Vec<Row> = parts
                .iter()
                .zip(&sizes)
                .enumerate()
                .map(|(k, ((name, _), size))| Row::new(name).subtitle(size).trail(Trail::Check(s.dump_pick == Some(k))))
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&list.rows, &(0..parts.len()).map(Action::PickDump).collect::<Vec<_>>()));
            let picked = s.dump_pick.and_then(|k| parts.get(k)).map(|(name, _)| name.clone());
            match picked {
                Some(name) => {
                    let note = tr!(l, "导出「{name}」", "匯出「{name}」", "Export “{name}”");
                    let one = widgets::list(&mut c, t, f, list.bottom, &[Row::new(&note).icon(GREEN, Icon::Backup)]);
                    out.extend(hits(&one.rows, &[Action::DumpNow]));
                    one.bottom
                }
                None => widgets::footnote(&mut c, t, f, list.bottom, l.pick(["点一个分区选中它。", "點一個分割區選中它。", "Tap a partition to choose it."])),
            }
        }
    };
    // 放哪儿：跟备份用的是同一组地方。
    let y = widgets::section(&mut c, t, f, y, l.pick(["存放位置", "存放位置", "Save to"]));
    let places = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(place_name(l, Place::Storage)).trail(Trail::Check(s.backup_to == Place::Storage)),
            Row::new(place_name(l, Place::UsbDrive)).trail(Trail::Check(s.backup_to == Place::UsbDrive)),
        ],
    );
    out.extend(hits(&places.rows, &[Action::BackupTo(Place::Storage), Action::BackupTo(Place::UsbDrive)]));
    // 已有的镜像：选一份，再决定写回还是删除。
    let y = widgets::section(&mut c, t, f, places.bottom, l.pick(["已有的镜像", "已有的映像檔", "Images kept"]));
    let y = match &s.image_copies {
        None => widgets::footnote(&mut c, t, f, y, l.pick(["正在读取", "正在讀取", "Reading"])),
        Some(Err(error)) => widgets::footnote(&mut c, t, f, y, error),
        Some(Ok(list)) if list.is_empty() => widgets::footnote(&mut c, t, f, y, l.pick(["这里还没有镜像", "這裡還沒有映像檔", "No images here yet"])),
        Some(Ok(list)) => {
            let details: Vec<String> = list.iter().map(|copy| image_copy_note(l, s, copy)).collect();
            let rows: Vec<Row> = list
                .iter()
                .zip(&details)
                .enumerate()
                .map(|(k, (copy, detail))| Row::new(&copy.partition).subtitle(detail).trail(Trail::Check(s.image_pick == Some(k))))
                .collect();
            let found = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&found.rows, &(0..list.len()).map(Action::ImagePick).collect::<Vec<_>>()));
            found.bottom
        }
    };
    // 一滑动做什么。
    let y = widgets::section(&mut c, t, f, y, l.pick(["对选中的镜像", "對選中的映像檔", "With the image chosen"]));
    let choices = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["写回它的分区", "寫回它的分割區", "Write it back"]))
                .subtitle(l.pick(["整份核对过再写，核对不过就不写", "整份核對過再寫，核對不過就不寫", "Checked whole before anything is written"]))
                .trail(Trail::Check(s.image_action == ImageAction::Restore)),
            Row::new(l.pick(["删除这份镜像", "刪除這份映像檔", "Delete it"])).trail(Trail::Check(s.image_action == ImageAction::Delete)),
        ],
    );
    out.extend(hits(
        &choices.rows,
        &[Action::ImageAction(ImageAction::Restore), Action::ImageAction(ImageAction::Delete)],
    ));
    // 再读一遍：从电脑上拷进来的镜像，不用离开这一页就能看见。
    let again = widgets::list(
        &mut c,
        t,
        f,
        choices.bottom,
        &[Row::new(l.pick(["重新读取镜像", "重新讀取映像檔", "Read the images again"])).icon(BLUE, Icon::Restore)],
    );
    out.extend(hits(&again.rows, &[Action::ImageRefresh]));
    let y = widgets::footnote(
        &mut c,
        t,
        f,
        again.bottom + t.m.card_gap,
        l.pick([
            "导出会写进 Skyward/backup，并在旁边留一份 sha256；写回之前整份核对一遍。",
            "匯出會寫進 Skyward/backup，並在旁邊留一份 sha256；寫回之前整份核對一遍。",
            "Exports go into Skyward/backup with a sha256 beside them, and are checked whole before they are written back.",
        ]),
    );
    let which = match s.image_action {
        ImageAction::Restore => Confirm::ImageRestore,
        ImageAction::Delete => Confirm::ImageDelete,
    };
    let (label, danger) = confirm_label(which, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(which) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(IMAGE_BACKUP), top)
}

/// 分区在导出那一行下面的一句：多重，super 另外说明。
fn image_size_note(l: Lang, name: &str, size: u64) -> String {
    if name == "super" {
        format!(
            "{} · {}",
            fmt::bytes(size),
            l.pick(["共享分区，包含双槽逻辑分区", "共用分割區，包含雙槽邏輯分割區", "Shared, holds both slots' logical partitions"])
        )
    } else {
        fmt::bytes(size)
    }
}

/// 一份镜像下面的一句：多大、什么时候写的、有没有校验。
fn image_copy_note(l: Lang, s: &State, copy: &ImageCopy) -> String {
    let checked = match copy.sha256 {
        Some(_) => l.pick(["有校验", "有核對", "checked"]),
        None => l.pick(["无校验", "無核對", "no checksum"]),
    };
    let when = match copy.made {
        0 => "—".to_string(),
        seconds => fmt::datetime(l, seconds, s.zone()),
    };
    format!("{} · {when} · {checked}", fmt::bytes(copy.size))
}

/// 关机页：先说清楚手上没做完的活会被切断，再滑到底关机。
pub fn shutdown(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let mut y = widgets::section_first(&mut c, t, f, top, l.pick(["关机前", "關機前", "Before it goes off"]));
    if s.install.writes() {
        y = widgets::paragraph(&mut c, t, f, y, l.pick([
            "安装还在写：现在关机，那个槽位会停在写到一半的位置，可能开不了机。",
            "安裝還在寫：現在關機，那個槽位會停在寫到一半的位置，可能開不了機。",
            "An install is still writing: off now, that slot stops half written and may not start.",
        ]));
    }
    if s.backup_job.is_some() {
        y = widgets::paragraph(&mut c, t, f, y, l.pick([
            "备份或恢复还在跑：现在关机，这一次不会留下完整的一份。",
            "備份或還原還在跑：現在關機，這一次不會留下完整的一份。",
            "A backup or a restore is running: off now, this one is left without a whole copy.",
        ]));
    }
    y = widgets::paragraph(&mut c, t, f, y, l.pick([
        "已经写进分区的数据不会自己回来；下次按住电源键就能重新开机，进的是恢复。",
        "已經寫進分割區的資料不會自己回來；下次按住電源鍵就能重新開機，進的是復原模式。",
        "What has gone into the partitions stays as it is; hold the power key to start again, into the recovery.",
    ]));
    let (label, danger) = confirm_label(Confirm::Shutdown, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
    let out = vec![Hit { rect: track, action: Action::Confirm(Confirm::Shutdown) }];
    Frame::new(t, c, out, l.pick(["关机", "關機", "Power off"]), top)
}

/// The modules the phone's own system loads — KernelSU's and Magisk's keep them
/// the same way — each with the switch that turns it on or off.
pub fn modules(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let mut out = Vec::new();
    // 与应用管理同一套：一屏一页，行高按「名称 + 版本」那一行量。
    let probe = Canvas::band(t.m.w, 1, 0..1, 0..t.m.w, t.pal.bg);
    let sample_h = {
        let mut sample = Row::new("Ag").icon(BLUE, Icon::Layers);
        sample.sub = Some("v1.0 · someone");
        widgets::one_height(&probe, t, f, &sample)
    };
    let room = t.m.h as f32 - top - t.m.margin * 2.0 - sample_h / 2.0;
    let per = ((room / sample_h).floor() as usize).max(1);
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    let mut pages = 1usize;
    match &s.modules {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(Err(why)) => {
            let mut row = Row::new(l.pick(["读取失败", "讀取失敗", "Not read"]));
            row.sub = Some(why.as_str());
            rows.push(row);
        }
        Some(Ok(list)) if list.is_empty() => rows.push(Row::new(l.pick([
            "没有安装模块",
            "沒有安裝模組",
            "No modules installed",
        ]))),
        Some(Ok(list)) => {
            pages = list.len().div_ceil(per).max(1);
            let page = s.module_page.min(pages - 1);
            let from = page * per;
            let to = (from + per).min(list.len());
            for (k, m) in list.iter().enumerate().take(to).skip(from) {
                let mut row = Row::new(&m.name).icon(if m.enabled { BLUE } else { SLATE }, Icon::Layers).trail(Trail::Switch(m.enabled));
                let off = l.pick(["已关闭", "已關閉", "Off"]);
                let removing = l.pick(["下次启动时移除", "下次啟動時移除", "removed next start"]);
                row.sub = Some(match (m.enabled, m.removing, m.note.is_empty()) {
                    (false, true, _) => removing,
                    (false, false, _) => off,
                    (true, true, _) => removing,
                    (true, false, true) => m.about.as_str(),
                    (true, false, false) => m.note.as_str(),
                });
                rows.push(row);
                acts.push(Action::ModuleOpen(k));
            }
        }
    }
    // 返回箭头那一行：右上角「安装」「搜索模块」；下一行只放翻页与页数。
    let ph = t.m.dp(widgets::PILL_DP);
    // 搜索在返回箭头那一行，翻页与标题「应用管理」同一行。
    let py = t.m.back_cy - ph / 2.0;
    // 与标题同一水平线。实测：标题的墨迹从 title_top 起、占到 0.92×字号，
    // 中心就在 0.46×字号处（f.title.ink 给的是相对基线的值，用它会偏上）。
    let pager_y = t.m.title_top + t.m.title_px * 0.46 - ph / 2.0;
    let list_top = top;
    let page = match &s.modules {
        Some(Ok(list)) if !list.is_empty() => s.module_page.min(list.len().div_ceil(per).max(1) - 1),
        _ => 0,
    };
    let gaps = t.m.dp(widgets::PILL_GAP_DP);
    let mut x_right = t.m.w as f32 - t.m.margin;
    let search = l.pick(["搜索模块", "搜尋模組", "Search modules"]);
    let w = widgets::pill_width(&c, t, f, search);
    x_right -= w;
    let wd = widgets::pill(&mut c, t, f, x_right, py, search, t.pal.search, t.pal.text);
    out.push(pill_hit(t, x_right, py, wd, ph, Action::SearchModules));
    x_right -= gaps;
    let install = l.pick(["安装", "安裝", "Install"]);
    let w = widgets::pill_width(&c, t, f, install);
    x_right -= w;
    let wd = widgets::pill(&mut c, t, f, x_right, py, install, t.pal.search, t.pal.text);
    out.push(pill_hit(t, x_right, py, wd, ph, Action::Open(Screen::Files)));
    if pages <= 1 {
        let only = l.pick(["只有一页", "只有一頁", "One page"]);
        let w = widgets::pill_width(&c, t, f, only);
        let x = t.m.w as f32 - t.m.margin - w;
        widgets::pill(&mut c, t, f, x, pager_y, only, t.pal.search, t.pal.text);
    }
    if pages > 1 {
        // 与应用页一样：三格位置固定，不可用的只画淡。
        let gaps = t.m.dp(widgets::PILL_GAP_DP);
        let counter = format!("{}/{}", page + 1, pages);
        let back = l.pick(["上一页", "上一頁", "Newer"]);
        let fwd = l.pick(["下一页", "下一頁", "Older"]);
        let w_prev = widgets::pill_width(&c, t, f, back);
        let w_next = widgets::pill_width(&c, t, f, fwd);
        let w_counter = c.text_width(&f.value, &counter, t.m.sub_px) + t.m.dp(8.0);
        let x_next = t.m.w as f32 - t.m.margin - w_next;
        let x_counter = x_next - gaps - w_counter;
        let x_prev = x_counter - gaps - w_prev;
        let faint = t.pal.subtitle;
        if page + 1 < pages {
            out.push(Hit {
                rect: Rect::new(t.m.w as f32 * 0.66, pager_y - t.m.dp(12.0), t.m.w as f32 * 0.34 - t.m.margin, ph + t.m.dp(24.0)),
                action: Action::ModulePage(1),
            });
        }
        if page > 0 {
            let wd = widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, t.pal.text);
            let left = x_prev - t.m.dp(16.0);
            let right = (x_prev + wd + t.m.dp(8.0)).min(t.m.w as f32 * 0.62);
            out.push(Hit {
                rect: Rect::new(left, pager_y - t.m.dp(10.0), (right - left).max(wd), ph + t.m.dp(20.0)),
                action: Action::ModulePage(-1),
            });
        } else {
            widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, faint);
        }
        c.text_center(&f.value, &counter, x_counter + w_counter / 2.0, pager_y + ph / 2.0, t.m.sub_px, faint);
        if page + 1 < pages {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, t.pal.text);
        } else {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, faint);
        }
    }
    let listed = widgets::list(&mut c, t, f, list_top, &rows);
    out.extend(hits(&listed.rows, &acts));
    Frame::new(t, c, out, l.pick(MODULES), top)
}

/// One module: what it says of itself, and the two things that can be done
/// with it — switched off or on, and taken away at the next start.
pub fn module_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let at = s.module_at.unwrap_or(0);
    let picked = match &s.modules {
        Some(Ok(list)) => list.get(at),
        _ => None,
    };
    let Some(m) = picked else {
        return Frame::new(t, c, Vec::new(), l.pick(MODULES), top);
    };
    let state_note = match (m.enabled, m.removing) {
        (true, true) => l.pick(["已开启 · 下次启动时移除", "已開啟 · 下次啟動時移除", "On · removed next start"]),
        (true, false) => l.pick(["已开启", "已開啟", "On"]),
        (false, _) => l.pick(["已关闭", "已關閉", "Off"]),
    };
    let mut facts: Vec<(&str, String)> = vec![(l.pick([["版本", "版本", "Version"]][0]), m.note.clone())];
    if !m.about.is_empty() {
        facts.push((l.pick([["说明", "說明", "About"]][0]), m.about.clone()));
    }
    facts.push((l.pick([["状态", "狀態", "State"]][0]), state_note.to_string()));
    facts.push((l.pick([["位置", "位置", "Where"]][0]), format!("/data/adb/modules/{}", m.id)));
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    let switch_label = if m.enabled {
        l.pick(["关闭模块", "關閉模組", "Turn it off"])
    } else {
        l.pick(["开启模块", "開啟模組", "Turn it on"])
    };
    let remove_label = if m.removing {
        l.pick(["取消卸载", "取消卸載", "Keep it"])
    } else {
        l.pick(["卸载模块", "卸載模組", "Uninstall"])
    };
    let acts = widgets::list(&mut c, t, f, y + t.m.card_gap, &[
        Row::new(switch_label).icon(BLUE, Icon::Layers),
        Row::new(remove_label).icon(RED, Icon::Wipe).tint(t.pal.danger),
    ]);
    let out = hits(&acts.rows, &[Action::ModuleToggle(at), Action::ModuleRemove(at)]);
    Frame::new(t, c, out, l.pick(MODULES), top)
}


const MODULES: Text = ["模块管理", "模組管理", "Modules"];

/// 小药丸的点击区：竖直放宽到 44dp，手指略有抖动也算点在它上面。
fn pill_hit(t: &Theme, x: f32, y: f32, w: f32, h: f32, action: Action) -> Hit {
    let pad = (t.m.dp(44.0) - h).max(0.0) / 2.0;
    Hit { rect: Rect::new(x - t.m.dp(4.0), y - pad, w + t.m.dp(8.0), h + pad * 2.0), action }
}

const DATA_APPS: Text = ["应用管理", "應用程式管理", "Apps"];

/// The apps the phone has in its data partition, newest first.
pub fn data_apps(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let mut out = Vec::new();
    // 一屏能放几行：按“名称 + 包名”那一行的实际高度算，再多留半行余量，
    // 免得最后一行被裁掉半截。
    let probe = Canvas::band(t.m.w, 1, 0..1, 0..t.m.w, t.pal.bg);
    let sample_h = {
        let mut sample = Row::new("Ag").icon(PURPLE, Icon::Layers);
        sample.sub = Some("com.example.app");
        widgets::one_height(&probe, t, f, &sample)
    };
    let room = t.m.h as f32 - top - t.m.margin * 2.0 - sample_h / 2.0;
    let per = ((room / sample_h).floor() as usize).max(1);
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    let mut pages = 1usize;
    // 开关：用户应用 / 系统应用。看一眼数，选中的那颗高亮。
    match &s.apps {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(Err(why)) => {
            let mut row = Row::new(l.pick(["读取失败", "讀取失敗", "Not read"]));
            row.sub = Some(why.as_str());
            rows.push(row);
        }
        Some(Ok(list)) if list.is_empty() => rows.push(Row::new(l.pick([
            "数据分区里没有应用",
            "資料分區裡沒有應用程式",
            "No apps in the data partition",
        ]))),
        Some(Ok(list)) => {
            pages = list.len().div_ceil(per).max(1);
            let page = s.app_page.min(pages - 1);
            let from = page * per;
            let to = (from + per).min(list.len());
            for (k, a) in list.iter().enumerate().take(to).skip(from) {
                let mut row = Row::new(&a.label).icon(PURPLE, Icon::Layers);
                row.sub = Some(&a.package);
                rows.push(row);
                acts.push(Action::DataAppOpen(k));
            }
        }
    }
    // 返回箭头那一行只放「搜索应用」，靠右；翻页键与页数另起一行放在它下面。
    let ph = t.m.dp(widgets::PILL_DP);
    // 搜索在返回箭头那一行；翻页与标题同一水平线（墨迹中心在 0.46×字号处）。
    let py = t.m.back_cy - ph / 2.0;
    let pager_y = t.m.title_top + t.m.title_px * 0.46 - ph / 2.0;
    let list_top = top;
    let page = s.app_page.min(pages.saturating_sub(1));
    let mut x = t.m.w as f32 - t.m.margin;
    let search_label = l.pick(["搜索应用", "搜尋應用程式", "Search apps"]);
    let w = widgets::pill_width(&c, t, f, search_label);
    x -= w;
    let wd = widgets::pill(&mut c, t, f, x, py, search_label, t.pal.search, t.pal.text);
    out.push(pill_hit(t, x, py, wd, ph, Action::SearchApps));
    // 「系统应用」：只看的清单，单独一页。
    let sys_label = l.pick(["系统应用", "系統應用程式", "System apps"]);
    let w = widgets::pill_width(&c, t, f, sys_label);
    x -= t.m.dp(widgets::PILL_GAP_DP) + w;
    let wd = widgets::pill(&mut c, t, f, x, py, sys_label, t.pal.search, t.pal.subtitle);
    out.push(pill_hit(t, x, py, wd, ph, Action::Open(Screen::SystemApps)));
    // 「已停用」：停用过的那几个收在一处，一键启用回来（2026-11-24 用户要的）。
    let stop_label = l.pick(["已停用", "已停用", "Disabled"]);
    let w = widgets::pill_width(&c, t, f, stop_label);
    x -= t.m.dp(widgets::PILL_GAP_DP) + w;
    let wd = widgets::pill(&mut c, t, f, x, py, stop_label, t.pal.search, t.pal.subtitle);
    out.push(pill_hit(t, x, py, wd, ph, Action::Open(Screen::StoppedApps)));
    // 只有一页时也说一句，别留一块空。
    if pages <= 1 {
        let only = l.pick(["只有一页", "只有一頁", "One page"]);
        let w = widgets::pill_width(&c, t, f, only);
        let x = t.m.w as f32 - t.m.margin - w;
        widgets::pill(&mut c, t, f, x, pager_y, only, t.pal.search, t.pal.text);
    }
    if pages > 1 {
        // 三格位置固定：上一页、页码、下一页。不可用的那格只画淡，不参与点击，
        // 也不移动——否则翻页后按钮挪位，下一次点会点到另一个键上（2026-11-18）。
        let gaps = t.m.dp(widgets::PILL_GAP_DP);
        let counter = format!("{}/{}", page + 1, pages);
        let w_prev = widgets::pill_width(&c, t, f, l.pick(["上一页", "上一頁", "Newer"]));
        let w_next = widgets::pill_width(&c, t, f, l.pick(["下一页", "下一頁", "Older"]));
        let w_counter = c.text_width(&f.value, &counter, t.m.sub_px) + t.m.dp(8.0);
        let x_next = t.m.w as f32 - t.m.margin - w_next;
        let x_counter = x_next - gaps - w_counter;
        let x_prev = x_counter - gaps - w_prev;
        let faint = t.pal.subtitle;
        let back = l.pick(["上一页", "上一頁", "Newer"]);
        let fwd = l.pick(["下一页", "下一頁", "Older"]);
        // 屏幕右侧三分之一都是「下一页」：手指偏一点也照样翻。
        if page + 1 < pages {
            out.push(Hit {
                rect: Rect::new(t.m.w as f32 * 0.66, pager_y - t.m.dp(12.0), t.m.w as f32 * 0.34 - t.m.margin, ph + t.m.dp(24.0)),
                action: Action::AppPage(1),
            });
        }
        if page > 0 {
            let wd = widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, t.pal.text);
            // 上一页只到屏幕 62% 处，右边那一大片留给「下一页」。
            let left = x_prev - t.m.dp(16.0);
            let right = (x_prev + wd + t.m.dp(8.0)).min(t.m.w as f32 * 0.62);
            out.push(Hit {
                rect: Rect::new(left, pager_y - t.m.dp(10.0), (right - left).max(wd), ph + t.m.dp(20.0)),
                action: Action::AppPage(-1),
            });
        } else {
            widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, faint);
        }
        c.text_center(&f.value, &counter, x_counter + w_counter / 2.0, pager_y + ph / 2.0, t.m.sub_px, faint);
        if page + 1 < pages {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, t.pal.text);
        } else {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, faint);
        }
    }
    let listed = widgets::list(&mut c, t, f, list_top, &rows);
    out.extend(hits(&listed.rows, &acts));
    // 记下这次画出来的总页数：点空时能核对翻页键是否已在画面上。
    // （pages 只在函数内，借用 &Ctx 只读，所以这里写到别处由调用方转记。）
    Frame::new(t, c, out, l.pick(DATA_APPS), top)
}

/// 系统分区里的预装应用：与用户应用同一套排布，点得开详情；那里能看它
/// 装在哪、占多大、把安装包导出来。系统分区是只读的，卸载不在这一页
/// （2026-11-19）。
pub fn system_apps(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let probe = Canvas::band(t.m.w, 1, 0..1, 0..t.m.w, t.pal.bg);
    let sample_h = {
        let mut sample = Row::new("Ag").icon(SLATE, Icon::Layers);
        sample.sub = Some("/system/app");
        widgets::one_height(&probe, t, f, &sample)
    };
    let room = t.m.h as f32 - top - t.m.margin * 2.0 - sample_h / 2.0;
    let per = ((room / sample_h).floor() as usize).max(1);
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    let mut pages = 1usize;
    match &s.sys_apps {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(list) if list.is_empty() => rows.push(Row::new(l.pick([
            "没有读到系统应用",
            "沒有讀到系統應用程式",
            "No system apps read",
        ]))),
        Some(list) => {
            pages = list.len().div_ceil(per).max(1);
            let page = s.sys_page.min(pages - 1);
            let from = page * per;
            let to = (from + per).min(list.len());
            for (k, a) in list.iter().enumerate().take(to).skip(from) {
                let mut row = Row::new(&a.label).icon(SLATE, Icon::Layers);
                // 名字还没读出来时标题就是包名，副标题再写一遍没意义：改写成
                // 它装在哪个分区，比重复一行有用。
                row.sub = Some(if a.label == a.package { a.dir.as_str() } else { a.package.as_str() });
                rows.push(row);
                acts.push(Action::SysAppOpen(k));
            }
        }
    }
    let page = s.sys_page.min(pages.saturating_sub(1));
    let ph = t.m.dp(widgets::PILL_DP);
    let pager_y = t.m.title_top + t.m.title_px * 0.46 - ph / 2.0;
    let mut out = Vec::new();
    if pages <= 1 {
        let only = l.pick(["只有一页", "只有一頁", "One page"]);
        let w = widgets::pill_width(&c, t, f, only);
        let x = t.m.w as f32 - t.m.margin - w;
        widgets::pill(&mut c, t, f, x, pager_y, only, t.pal.search, t.pal.text);
    } else {
        let gaps = t.m.dp(widgets::PILL_GAP_DP);
        let counter = format!("{}/{}", page + 1, pages);
        let back = l.pick(["上一页", "上一頁", "Newer"]);
        let fwd = l.pick(["下一页", "下一頁", "Older"]);
        let w_prev = widgets::pill_width(&c, t, f, back);
        let w_next = widgets::pill_width(&c, t, f, fwd);
        let w_counter = c.text_width(&f.value, &counter, t.m.sub_px) + t.m.dp(8.0);
        let x_next = t.m.w as f32 - t.m.margin - w_next;
        let x_counter = x_next - gaps - w_counter;
        let x_prev = x_counter - gaps - w_prev;
        let faint = t.pal.subtitle;
        if page + 1 < pages {
            out.push(Hit {
                rect: Rect::new(t.m.w as f32 * 0.66, pager_y - t.m.dp(12.0), t.m.w as f32 * 0.34 - t.m.margin, ph + t.m.dp(24.0)),
                action: Action::SysPage(1),
            });
        }
        if page > 0 {
            let wd = widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, t.pal.text);
            let left = x_prev - t.m.dp(16.0);
            let right = (x_prev + wd + t.m.dp(8.0)).min(t.m.w as f32 * 0.62);
            out.push(Hit {
                rect: Rect::new(left, pager_y - t.m.dp(10.0), (right - left).max(wd), ph + t.m.dp(20.0)),
                action: Action::SysPage(-1),
            });
        } else {
            widgets::pill(&mut c, t, f, x_prev, pager_y, back, t.pal.search, faint);
        }
        c.text_center(&f.value, &counter, x_counter + w_counter / 2.0, pager_y + ph / 2.0, t.m.sub_px, faint);
        if page + 1 < pages {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, t.pal.text);
        } else {
            widgets::pill(&mut c, t, f, x_next, pager_y, fwd, t.pal.search, faint);
        }
    }
    let listed = widgets::list(&mut c, t, f, top, &rows);
    out.extend(hits(&listed.rows, &acts));
    Frame::new(t, c, out, l.pick(SYSTEM_APPS), top)
}

const STOPPED_APPS: Text = ["已停用的应用", "已停用的應用程式", "Disabled apps"];

/// 被停用过的系统应用：`pm-state` 里记着哪几个，从这里一键启用。
///
/// 停用的应用仍躺在系统分区里（系统应用页也还看得见、详情页会给「启用应用」），
/// 但要在几十页里把它翻出来太费事 —— 这一页把它们收在一处（2026-11-24 用户要的）。
pub fn stopped_apps(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let mut out = Vec::new();
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    // 只列「停用」的那些（`pm-state` 里 true）；还没跑完的排队的也算。
    let mut stopped: Vec<String> = Vec::new();
    if let Some(map) = &s.app_disabled {
        for (package, off) in map {
            if *off {
                stopped.push(package.clone());
            }
        }
    }
    if stopped.is_empty() {
        let mut row = Row::new(l.pick(["没有停用的应用", "沒有停用的應用程式", "Nothing is disabled"]));
        row.sub = Some(l.pick([
            "在系统应用里点「停用应用」，重启系统后会出现在这里",
            "在系統應用程式裡點「停用應用程式」，重啟系統後會出現在這裡",
            "Disable one from the system apps; it shows up here after the system starts",
        ]));
        rows.push(row);
    } else {
        for (k, package) in stopped.iter().enumerate() {
            let queued = s.app_state_queued.get(package).copied();
            let mut row = Row::new(package.as_str()).icon(ORANGE, Icon::Shield).tint(t.pal.subtitle);
            row.sub = Some(match queued {
                Some(false) => l.pick(["已排队启用，下次启动系统时执行", "已排隊啟用，下次啟動系統時執行", "Queued to be enabled"]),
                _ => l.pick(["点这里启用回来", "點這裡啟用回來", "Tap to enable it again"]),
            });
            rows.push(row);
            acts.push(Action::StoppedEnable(k));
        }
    }
    let listed = widgets::list(&mut c, t, f, top, &rows);
    out.extend(hits(&listed.rows, &acts));
    if !stopped.is_empty() {
        let note = l.pick([
            "启用由系统在下次启动时执行，完成后这一页就空了。",
            "啟用由系統在下次啟動時執行，完成後這一頁就空了。",
            "Enabling runs when the system next starts; this page empties out then.",
        ]);
        widgets::footnote(&mut c, t, f, listed.bottom, note);
    }
    Frame::new(t, c, out, l.pick(STOPPED_APPS), top)
}

const SYSTEM_APPS: Text = ["系统应用", "系統應用程式", "System apps"];

/// One app: what it is, when it came, how much room it takes, and what can be
/// done with it — taking a data app off, or exporting a system app's package,
/// which sits on a read-only partition and cannot be removed here.
pub fn data_app_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    // 详情页两种来源：系统应用页点进来的是 sys_at，用户应用页点进来的是
    // app_at。系统应用只读，动作是导出安装包（2026-11-19）。
    let from_system = s.sys_at.is_some();
    let at = s.sys_at.or(s.app_at).unwrap_or(0);
    let picked = match &s.sys_at {
        Some(k) => s.sys_apps.as_ref().and_then(|list| list.get(*k)),
        None => match &s.apps {
            Some(Ok(list)) => list.get(at),
            _ => None,
        },
    };
    let Some(a) = picked else {
        return Frame::new(t, c, Vec::new(), l.pick(if from_system { SYSTEM_APPS } else { DATA_APPS }), top);
    };
    let facts: Vec<(&str, String)> = if from_system {
        vec![
            (l.pick(["包名", "套件名稱", "Package"]), a.package.clone()),
            (l.pick(["位置", "位置", "Where"]), a.dir.clone()),
            (l.pick(["占用", "佔用", "Takes"]), crate::fmt::file_size(a.size)),
            (l.pick(["类型", "類型", "Kind"]), l.pick(["系统分区（只读）", "系統分割區（唯讀）", "System partition (read-only)"]).to_string()),
        ]
    } else {
        vec![
            ("包名", a.package.clone()),
            ("名称", a.label.clone()),
            ("位置", a.dir.clone()),
            ("占用", crate::fmt::file_size(a.size)),
            ("安装", crate::fmt::datetime(l, a.installed, s.zone())),
            ("更新", crate::fmt::datetime(l, a.updated, s.zone())),
        ]
    };
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    // 系统分区的应用不从这里删：分区只读，删了系统也起不来。能给的是把
    // 安装包导到内部存储，装到别的机器上或者留个底。
    let (rows, acts): (Vec<Row>, Vec<Action>) = if from_system {
        let export = Row::new(l.pick(["导出安装包", "匯出安裝包", "Export the package"]))
            .icon(PURPLE, Icon::Layers)
            .tint(t.pal.accent);
        let why = Row::new(l.pick(["系统应用，不能在这里卸载", "系統應用程式，不能在這裡卸載", "A system app: not removed here"]))
            .icon(SLATE, Icon::Shield);
        let said = l.pick([
            "系统应用在只读的系统分区里，删了系统起不来，只能导出安装包",
            "系統應用程式在唯讀的系統分割區裡，刪了系統起不來，只能匯出安裝包",
            "System apps sit on a read-only partition: it can be exported, not removed here",
        ]);
        // 停用/启用：系统应用删不掉，但可以停用。包管理器的状态在系统的
        // data 里，recovery 改不了，于是把请求排队，系统启动后由它自己执行
        // （2026-11-23 用户要的）。
        let known = s.app_disabled.as_ref().and_then(|m| m.get(&a.package).copied()).unwrap_or(false);
        let off = s.app_state_queued.get(&a.package).copied().unwrap_or(known);
        let (label, action) = if off {
            (l.pick(["启用应用", "啟用應用程式", "Enable the app"]), Action::SysAppEnable(at))
        } else {
            (l.pick(["停用应用", "停用應用程式", "Disable the app"]), Action::SysAppDisable(at))
        };
        let toggle = Row::new(label)
            .icon(ORANGE, Icon::Shield)
            .tint(if off { t.pal.accent } else { t.pal.danger })
            .subtitle(if s.app_state_queued.contains_key(&a.package) {
                l.pick(["已排队，下次启动系统时执行", "已排隊，下次啟動系統時執行", "Queued for the system's next start"])
            } else {
                l.pick(["在系统下次启动时执行", "在系統下次啟動時執行", "Done when the system next starts"])
            });
        (vec![export, why, toggle], vec![Action::SysAppExport(at), Action::Unavailable(said), action])
    } else if a.system {
        (
            vec![Row::new(l.pick(["系统应用，不能这样卸载", "系統應用程式，不能這樣卸載", "A system app: not removed here"])).icon(SLATE, Icon::Shield)],
            vec![Action::Unavailable(l.pick(["系统应用不能这样卸载", "系統應用程式不能這樣卸載", "A system app is not removed here"]))],
        )
    } else {
        (
            vec![Row::new(l.pick(["卸载应用", "卸載應用程式", "Uninstall"])).icon(RED, Icon::Wipe).tint(t.pal.danger)],
            vec![Action::DataAppRemove(at)],
        )
    };
    let listed = widgets::list(&mut c, t, f, y + t.m.card_gap, &rows);
    let out = hits(&listed.rows, &acts);
    if from_system {
        let note = l.pick([
            "停用与启用由系统在下次启动时执行，完成后状态写在系统里；停用的应用不在桌面上显示，需要回来时可以在这里启用。",
            "停用與啟用由系統在下次啟動時執行，完成後狀態寫在系統裡；停用的應用程式不在桌面上顯示，需要回來時可以在這裡啟用。",
            "Disabling and enabling run when the system next starts, and the state stays in it. A disabled app drops off the home screen; enable it here to bring it back.",
        ]);
        widgets::footnote(&mut c, t, f, listed.bottom, note);
    }
    Frame::new(t, c, out, l.pick(if from_system { SYSTEM_APPS } else { DATA_APPS }), top)
}

const RELABEL: Text = ["修复 SELinux 上下文", "修復 SELinux 上下文", "Fix SELinux contexts"];
const FILESYSTEMS: Text = ["文件系统", "檔案系統", "Filesystems"];
const STOPPED: Text = ["已停止", "已停止", "Stopped"];
const STOPPING: Text = ["正在停止", "正在停止", "Stopping"];

/// Giving the data partition's files the SELinux labels the installed
/// system's rules give them: what it does, and a swipe to start it; then
/// how far it has come, a way to stop it, and how it ended.
pub fn relabel(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(RELABEL);
    let Some(job) = &s.relabel else {
        let what = l.pick([
            "按已安装系统的规则，重新设置 data 分区中文件的 SELinux 上下文，应用数据保持不变。",
            "按已安裝系統的規則，重新設定 data 分割區中檔案的 SELinux 上下文，應用程式資料保持不變。",
            "Gives the files in the data partition the SELinux contexts the installed system's rules give them. App data stays as it is.",
        ]);
        let mut y = widgets::paragraph(&mut c, t, f, top, what);
        let note = match s.data {
            Data::Open | Data::Locked { .. } => None,
            Data::Erased => Some(["data 分区已清除。", "data 分割區已清除。", "The data partition was wiped."]),
            Data::Closed | Data::Opening | Data::Failed(_) => {
                Some(["暂无法修复，需先在「解密」中打开 data 分区。", "暫時無法修復，需先在「解密」中開啟 data 分割區。", "Open the data partition in Decrypt first."])
            }
        };
        if let Some(note) = note {
            y = widgets::footnote(&mut c, t, f, y, l.pick(note));
        }
        let (label, danger) = confirm_label(Confirm::Relabel, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Relabel) });
        return Frame::new(t, c, out, title, top);
    };
    let repair = Verb::of(&crate::app::PartKind::Repair);
    let heading = match &job.outcome {
        None => repair.going(l),
        Some(JobEnd::Done) => repair.complete(l),
        Some(JobEnd::Stopped) => l.pick(STOPPED).into(),
        Some(JobEnd::Failed(_)) => repair.failed(l),
    };
    let detail = match (job.total, job.checked) {
        (0, _) if job.outcome.is_none() => l.pick(GETTING_READY).to_string(),
        (total, checked) => tr!(l, "已检查 {checked} 项，共 {total} 项", "已檢查 {checked} 項，共 {total} 項", "{checked} of {total} checked"),
    };
    let y = widgets::progress_card(&mut c, t, f, top, &heading, &detail, Some(share(job.checked, job.total)));
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { STOPPING } else { STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopRelabel]));
            }
        }
        Some(end) => {
            let changed = job.changed;
            let note = match end {
                JobEnd::Done if changed == 0 => l
                    .pick(["全部文件的 SELinux 上下文都正确，无需修复。", "全部檔案的 SELinux 上下文都正確，無需修復。", "Every file's SELinux context is right, so nothing needed fixing."])
                    .to_string(),
                JobEnd::Done => tr!(
                    l,
                    "已按规则重新设置 {changed} 项的 SELinux 上下文。",
                    "已按規則重新設定 {changed} 項的 SELinux 上下文。",
                    "{changed} entries got the SELinux context the rules give them."
                ),
                JobEnd::Stopped => tr!(l, "已停止，已重新设置 {changed} 项。", "已停止，已重新設定 {changed} 項。", "Stopped, with {changed} entries set again."),
                JobEnd::Failed(why) => tr!(l, "修复失败：{why}", "修復失敗：{why}", "Repair failed: {why}"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// The partitions whose filesystems can be repaired or made afresh as
/// another, each with what it holds and its size.
pub fn filesystems(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["分区", "分割區", "Partitions"]));
    let bottom = match &s.filesystems {
        None => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["正在读取分区", "正在讀取分割區", "Reading the partitions"])).trail(Trail::None)]).bottom,
        Some(Err(why)) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["无法读取分区", "無法讀取分割區", "Couldn't read the partitions"])).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Some(Ok(rows)) if rows.is_empty() => {
            widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["暂无可修复的分区", "沒有可修復的分割區", "No partitions to repair"])).trail(Trail::None)]).bottom
        }
        Some(Ok(rows)) => {
            let subs: Vec<String> = rows.iter().map(|r| format!("{} · {}", fs_held(l, r, &s.data), fmt::bytes(r.size))).collect();
            let shown: Vec<Row> = rows.iter().zip(&subs).map(|(r, sub)| Row::new(&r.name).subtitle(sub)).collect();
            let list = widgets::list(&mut c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..rows.len()).map(Action::OpenFilesystem).collect::<Vec<_>>()));
            let note = l.pick(["仅显示本机分区表中可格式化的分区。", "僅顯示本機分割區表中可格式化的分割區。", "Only the partitions in this phone's partition table that can be formatted show here."]);
            widgets::footnote(&mut c, t, f, list.bottom, note)
        }
    };
    Frame::new(t, fit(c, t, bottom), out, l.pick(FILESYSTEMS), top)
}

/// What a partition holds, as its row and page name it: the data
/// partition's is read only once it is open, and until then the row says
/// where it stands, as the home page does.
fn fs_held(l: Lang, row: &FsRow, data: &Data) -> String {
    match (&row.filesystem, row.name == DATA_PARTITION) {
        (Some(filesystem), _) => filesystem.clone(),
        (None, true) => fmt::data(l, data).into(),
        (None, false) => l.pick(["无法识别", "無法辨識", "Unrecognized"]).into(),
    }
}

/// One partition: what it holds, what can be done to it, and a swipe to do
/// what is chosen.
pub fn filesystem(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let Some(Ok(rows)) = &s.filesystems else { unreachable!("a partition's page opens from the list read") };
    let row = &rows[s.fs_chosen];
    let data = row.name == DATA_PARTITION;
    let facts = [
        (l.pick(["挂载点", "掛載點", "Mount point"]), row.mount_point.clone()),
        (l.pick(["文件系统", "檔案系統", "Filesystem"]), fs_held(l, row, &s.data)),
        (l.pick(["大小", "大小", "Size"]), fmt::bytes(row.size)),
    ];
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    let chosen = |action: &FsAction| s.fs_action == *action;
    let remakes: Vec<String> = row.others.iter().map(|fs| tr!(l, "更换为 {fs}", "更換為 {fs}", "Change to {fs}")).collect();
    let made: Vec<String> = row.others.iter().map(|_| remade(row, s)).collect();
    let repair = Row::new(l.pick(["修复文件系统", "修復檔案系統", "Repair the filesystem"]))
        .subtitle(l.pick(["检查整个文件系统，并修复发现的错误", "檢查整個檔案系統，並修復發現的錯誤", "Checks the whole filesystem and fixes the errors found"]))
        .trail(Trail::Check(chosen(&FsAction::Repair)));
    let mut shown = vec![repair];
    let mut actions = vec![Action::ChooseFs(FsChoice::Repair)];
    for (k, fs) in row.others.iter().enumerate() {
        shown.push(Row::new(&remakes[k]).subtitle(&made[k]).trail(Trail::Check(chosen(&FsAction::Remake(fs.clone())))));
        actions.push(Action::ChooseFs(FsChoice::Remake(k)));
    }
    let list = widgets::list(&mut c, t, f, y, &shown);
    out.extend(hits(&list.rows, &actions));
    let mut y = list.bottom;
    if row.others.is_empty() {
        let note = l.pick(["暂不支持更换该分区的文件系统。", "尚不支援更換這個分割區的檔案系統。", "This partition's filesystem can't be changed."]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if data {
        let note = l.pick([
            "修复前将关闭 data 分区，完成后需在「解密」中重新打开。",
            "修復前將關閉 data 分割區，完成後需在「解密」中重新開啟。",
            "The data partition closes for the repair, and opens again from Decrypt once it's done.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if !row.others.is_empty() {
        y = widgets::footnote(&mut c, t, f, y, l.pick(["更换后，分区中的内容无法恢复。", "更換後，分割區中的內容無法復原。", "Once changed, what the partition held can't be brought back."]));
    }
    let (label, danger) = confirm_label(Confirm::Filesystem, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Filesystem) });
    Frame::new(t, fit(c, t, track.y + track.h), out, partition_title(l, &row.name), top)
}

/// The title of the page of a partition the filesystem page lists.
fn partition_title(l: Lang, name: &str) -> &'static str {
    l.pick(match name {
        DATA_PARTITION => DATA_PART,
        "metadata" => ["metadata 分区", "metadata 分割區", "metadata partition"],
        "cache" => ["cache 分区", "cache 分割區", "cache partition"],
        _ => ["分区", "分割區", "Partition"],
    })
}

/// What making `row` afresh as another filesystem erases: the data
/// partition goes as a factory reset goes, with /metadata and the cache
/// partition beside it.
fn remade(row: &FsRow, s: &State) -> String {
    match row.name == DATA_PARTITION {
        true => {
            let parts = s.lang.pick(match s.cache_partition {
                true => ["data、cache、metadata 分区", "data、cache、metadata 分割區", "the data, cache and metadata partitions"],
                false => ["data、metadata 分区", "data、metadata 分割區", "the data and metadata partitions"],
            });
            tr!(s.lang, "格式化 {parts}，清空本机的全部数据", "格式化 {parts}，清空本機的全部資料", "Formats {parts}, emptying all of this phone's data")
        }
        false => {
            let name = &row.name;
            tr!(s.lang, "格式化 {name} 分区，清空其中的全部内容", "格式化 {name} 分割區，清空其中的全部內容", "Formats the {name} partition, emptying all it holds")
        }
    }
}

/// Work on whole partitions: each partition and where it stands, what the
/// program said of the last, and once it has ended, how. What the system
/// asked for leaves on its own once it went through every partition, and
/// offers where to go otherwise; what the user chose goes back to its page.
pub fn part_work(cx: &Ctx) -> Frame {
    use crate::app::{ItemEnd, ItemState, PartKind};
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let work = s.part_work.as_ref().expect("the page of partition work shows work");
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let verb = Verb::of(&work.kind);
    // Work on one partition fails; work through several may get partway.
    let (title, several) = match &work.kind {
        PartKind::Disable => (["清除分区", "清除分割區", "Wipe partitions"], true),
        PartKind::Unshare => (["取消数据块共享", "取消資料區塊共用", "Unshare blocks"], true),
        PartKind::Repair => (["修复文件系统", "修復檔案系統", "Repair filesystem"], false),
        PartKind::Remake(_) => (["更换文件系统", "更換檔案系統", "Change filesystem"], false),
    };
    let failed = |state: &ItemState| matches!(state, ItemState::Ended(ItemEnd::Failed(_)));
    let short = matches!(work.outcome, Some(Err(_))) || work.items.iter().any(|(_, state)| failed(state));
    let heading = match (&work.outcome, short, several) {
        (None, _, _) => verb.going(l),
        (Some(_), false, _) => verb.complete(l),
        (Some(_), true, true) => verb.incomplete(l),
        (Some(_), true, false) => verb.failed(l),
    };
    let mut y = widgets::section_first(&mut c, t, f, top, &heading);
    if !work.items.is_empty() {
        let states: Vec<String> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Waiting if work.outcome.is_some() => verb.not_done(l),
                ItemState::Waiting => l.pick(WAITING_TURN).into(),
                ItemState::Working => verb.going(l),
                ItemState::Ended(end) => fmt::item_end(l, end, verb),
            })
            .collect();
        let held: Vec<Option<&str>> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Ended(ItemEnd::NotShared(fs)) if !fs.is_empty() => Some(fs.as_str()),
                _ => None,
            })
            .collect();
        let shown: Vec<Row> = work
            .items
            .iter()
            .zip(&states)
            .zip(&held)
            .map(|(((name, _), state), held)| {
                let row = Row::new(name).trail(Trail::Text(state));
                match held {
                    Some(fs) => row.subtitle(fs),
                    None => row,
                }
            })
            .collect();
        y = widgets::list(&mut c, t, f, y, &shown).bottom;
    }
    let leaving = match s.asked.shutdown_after {
        true => ["完成后将自动关机。", "完成後將自動關機。", "Powers off on its own once done."],
        false => ["完成后将自动重启。", "完成後將自動重新啟動。", "Reboots on its own once done."],
    };
    let mut notes: Vec<String> = Vec::new();
    match &work.outcome {
        None if work.asked => notes.push(l.pick(leaving).into()),
        None => notes.push(verb.hold_on(l)),
        Some(Err(stop)) => {
            let (what, detail) = fmt::part_stop(l, stop, &cx.info.codename);
            notes.push(what);
            notes.extend(detail);
        }
        Some(Ok(())) => {}
    }
    for (name, state) in &work.items {
        if let ItemState::Ended(ItemEnd::Failed(Some(why))) = state {
            let failed = verb.failed_on(l, name);
            notes.push(tr!(l, "{failed}：{why}", "{failed}：{why}", "{failed}: {why}"));
        }
    }
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (work.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    match (&work.outcome, work.asked) {
        (None, _) => {}
        (Some(_), true) if !short => {}
        // The system's work ran without the slot's partitions, which going
        // on in the recovery brings up first, as leaving fastboot does.
        (Some(_), true) => {
            let below = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
            let rows: Vec<Row> = AFTERWARDS.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
            let actions = AFTERWARDS.map(|(_, action)| if action == Action::Dismiss { Action::EnterRecovery } else { action });
            let choices = widgets::list(&mut c, t, f, below, &rows);
            out.extend(hits(&choices.rows, &actions));
            y = choices.bottom;
        }
        (Some(_), false) => {
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
            y = finish.bottom;
        }
    }
    if !work.output.is_empty() {
        let below = widgets::section(&mut c, t, f, y, l.pick(["程序输出", "程式輸出", "Program output"]));
        rows = Some(Rows { edges: widgets::output_edges(&c, t, f, below, &work.output), action: None });
        y = below;
    }
    let back = work.outcome.is_some() && !work.asked;
    Frame { rows, back, ..Frame::new(t, fit(c, t, y), out, l.pick(title), top) }
}

const ACTIONS: Text = ["操作", "操作", "Actions"];

/// The system on the phone now; the newest system the phone maker's
/// server knows, and downloading its full package to install it; and
/// where an update the system installed stands, merging it into the slot it
/// went to: how far a merge under way has come, and a swipe to merge one
/// that waits there for the system to start it.
pub fn update(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    if let Standing::Merging(done) = s.update {
        let share = done as f32;
        let p = percent(share);
        let detail = tr!(l, "已完成 {p}%", "已完成 {p}%", "{p}% done");
        let y = widgets::progress_card(&mut c, t, f, top, l.pick(["正在合并更新", "正在合併更新", "Merging the update"]), &detail, Some(share));
        let note = l.pick([
            "合并完成前，暂不刷写 super 中的分区。返回后将继续合并。",
            "合併完成前，暫不刷寫 super 中的分割區。返回後將繼續合併。",
            "Partitions in super aren't flashed until it merges. Merging goes on after going back.",
        ]);
        let y = widgets::footnote(&mut c, t, f, y, note);
        return Frame::new(t, fit(c, t, y), out, l.pick(UPDATE), top);
    }
    let mut y = top;
    if let Some(v) = &i.os_version {
        y = widgets::section_first(&mut c, t, f, y, l.pick(["本机", "本機", "This phone"]));
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(["当前版本", "目前版本", "Current version"]), v.clone())]);
    }
    if s.update_server {
        let online_update = l.pick(["在线更新", "線上更新", "Online update"]);
        y = if y == top { widgets::section_first(&mut c, t, f, y, online_update) } else { widgets::section(&mut c, t, f, y, online_update) };
        y = online(&mut c, cx, y, &mut out);
    }
    let next = match &s.boot_slots {
        Some(Ok(slots)) => Some(slots.active),
        _ => None,
    };
    let (title, detail) = fmt::update(l, &s.update, next);
    let color = match s.update {
        Standing::None => SLATE,
        Standing::Merged => GREEN,
        Standing::Failed(_) | Standing::Unknown => RED,
        _ => ORANGE,
    };
    let mut row = Row::new(title).icon(color, Icon::Layers).trail(Trail::None);
    if let Some(detail) = &detail {
        row = row.subtitle(detail);
    }
    let merging = l.pick(["更新合并", "更新合併", "Merging"]);
    y = if y == top { widgets::section_first(&mut c, t, f, y, merging) } else { widgets::section(&mut c, t, f, y, merging) };
    y = widgets::list(&mut c, t, f, y, &[row]).bottom;
    // A merge that stopped is asked for again here as one that waits is:
    // what is left of it merges from where it stopped (2026-11-20).
    if matches!(s.update, Standing::Waiting | Standing::Failed(_)) {
        let (label, danger) = confirm_label(Confirm::Merge, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Merge) });
        let note = match s.update {
            Standing::Failed(_) => {
                l.pick(["再合并会从上次停下的地方继续。", "再合併會從上次停下的地方繼續。", "Merging again goes on from where it stopped."])
            }
            _ => l.pick(["合并后无法回到更新前的系统。", "合併後無法回到更新前的系統。", "Once merged, the system from before the update can't come back."]),
        };
        y = widgets::footnote(&mut c, t, f, track.y + track.h, note);
        // An update that will not merge at all is given up here, so that
        // the phone starts from its own partitions instead of coming back
        // to the recovery at every restart (2026-11-20).
        if matches!(s.update, Standing::Failed(_)) {
            let (label, danger) = confirm_label(Confirm::GiveUp, cx.state);
            let drop = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: drop, action: Action::Confirm(Confirm::GiveUp) });
            let note = l.pick([
                "放弃后这次更新作废，开机还是更新前的系统。",
                "放棄後這次更新作廢，開機還是更新前的系統。",
                "Giving it up drops this update; the phone starts the system it had before",
            ]);
            y = widgets::footnote(&mut c, t, f, drop.y + drop.h, note);
        }
    }
    Frame::new(t, fit(c, t, y), out, l.pick(UPDATE), top)
}

/// What downloading or checking for updates needs first, while the phone
/// has joined no network that reaches the internet.
const WLAN_FIRST: Text = ["需先连接 WLAN", "需先連接 Wi-Fi", "Connect to Wi-Fi first"];
const CHECKING: Text = ["正在检查更新", "正在檢查更新", "Checking for updates"];
const CHECK_FOR_UPDATES: Text = ["检查更新", "檢查更新", "Check for updates"];
const DOWNLOADING: Text = ["正在下载", "正在下載", "Downloading"];
const CHECKING_PART: Text = ["正在校验已下载的部分", "正在驗證已下載的部分", "Verifying what was downloaded"];

/// Asking the update server for the newest system, then its full package:
/// downloading it, how far that has come, and installing it once it is
/// whole; then what to note before installing it.
fn online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let release = match &s.check {
        Check::Asking => return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom,
        Check::Answered(Some(r)) => r,
        other => {
            let mut row = Row::new(l.pick(CHECK_FOR_UPDATES));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckUpdate]));
            return match other {
                Check::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                Check::Answered(None) => {
                    let none = l.pick(["暂无本机的系统更新信息。", "目前沒有本機的系統更新資訊。", "The server knows of no system update for this phone."]);
                    widgets::footnote(c, t, f, check.bottom, none)
                }
                _ => check.bottom,
            };
        }
    };
    let newer = i.os_version.as_deref() != Some(release.version.as_str());
    let version = &release.version;
    let title = if newer { tr!(l, "新版本 {version}", "新版本 {version}", "New version {version}") } else { l.pick(["已是最新版本", "已是最新版本", "Up to date"]).to_string() };
    let size = match release.length {
        Some(length) => fmt::file_size(length),
        None => fmt::package_size(&release.size),
    };
    let package = tr!(l, "完整更新包 {size}", "完整更新檔 {size}", "Full OTA package {size}");
    let mut y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Layers).subtitle(&package).trail(Trail::None)]).bottom;
    match (&s.download, s.have) {
        (Some(job), _) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.progress {
                None => (l.pick(GETTING_READY), String::new(), None),
                Some(p) => downloaded(l, p, job.rate),
            };
            y = widgets::progress_card(c, t, f, y + t.m.card_gap, heading, &detail, share);
            let label = l.pick(if job.stopping { ["正在暂停", "正在暫停", "Pausing"] } else { ["暂停", "暫停", "Pause"] });
            let stop = widgets::list(c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopDownload]));
            }
            y = stop.bottom;
        }
        (_, Have::Whole(_)) => {
            let row = Row::new(l.pick(["安装", "安裝", "Install"])).subtitle(l.pick(["已下载到内部存储", "已下載到內部儲存空間", "Downloaded to internal storage"]));
            let install = widgets::list(c, t, f, y + t.m.card_gap, &[row]);
            out.extend(hits(&install.rows, &[Action::InstallDownloaded]));
            y = install.bottom;
        }
        (job, have) => {
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(DownloadEnd::Failed(why)) => y = widgets::footnote(c, t, f, y, &fmt::download_failure(l, why)),
                Some(DownloadEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(["已暂停下载。", "已暫停下載。", "Download paused."])),
                _ => {}
            }
            let title = l.pick(if matches!(have, Have::Part(_)) { ["继续下载", "繼續下載", "Resume download"] } else { ["下载", "下載", "Download"] });
            let had = match have {
                Have::Part(done) => {
                    let done = fmt::file_size(done);
                    Some(tr!(l, "已下载 {done}", "已下載 {done}", "{done} downloaded"))
                }
                _ => None,
            };
            let sub = if !s.data.storage_open() {
                Some(l.pick(DECRYPT_NEEDED))
            } else if !s.online() {
                Some(l.pick(WLAN_FIRST))
            } else {
                had.as_deref()
            };
            let row = Row::new(title);
            let download = widgets::list(c, t, f, y + t.m.card_gap, &[sub.map_or(row, |sub| row.subtitle(sub))]);
            out.extend(hits(&download.rows, &[Action::Download]));
            y = download.bottom;
        }
    }
    let saved = tr!(
        l,
        "完整更新包将保存到\"/sdcard/{DOWNLOADS}\"中，安装后不会自动删除。",
        "完整更新檔將儲存到「/sdcard/{DOWNLOADS}」中，安裝後不會自動刪除。",
        "The full OTA package is saved in “/sdcard/{DOWNLOADS}”, and stays there after it's installed."
    );
    let points = [
        l.pick(["安装系统更新不会删除用户数据，建议在安装前进行备份。", "安裝系統更新不會刪除使用者資料，建議在安裝前進行備份。", "Installing a system update deletes no user data. Backing up first is still wise."]),
        saved.as_str(),
        l.pick(["下载中断后可继续下载，已下载的部分会保留。", "下載中斷後可繼續下載，已下載的部分會保留。", "A download cut short can be resumed, keeping what was downloaded."]),
        l.pick([
            "更新后，系统会在后台进行一系列优化，期间可能出现发热、耗电增加等情况，1 ~ 3 天内将自行恢复。",
            "更新後，系統會在背景進行一系列最佳化，期間可能出現發熱、耗電增加等情況，1 ~ 3 天內將自行恢復。",
            "After the update the system optimizes itself in the background, and the phone may run warm or use more battery for 1 to 3 days.",
        ]),
    ];
    widgets::notes(c, t, f, y + t.m.card_gap, l.pick(NOTES), &points)
}

const NOTES: Text = ["注意事项", "注意事項", "Notes"];

/// What a progress card says of a download, and the share of it done:
/// what an earlier download left being checked, or the rest coming at
/// `rate` bytes a second.
fn downloaded(l: Lang, progress: Progress, rate: u64) -> (&'static str, String, Option<f32>) {
    match progress {
        Progress::Checking { done, total } => (l.pick(CHECKING_PART), format!("{} / {}", fmt::file_size(done), fmt::file_size(total)), Some(share(done, total))),
        Progress::Downloading { done, total } => (
            l.pick(DOWNLOADING),
            format!("{} / {} · {}/s", fmt::file_size(done), fmt::file_size(total), fmt::file_size(rate)),
            Some(share(done, total)),
        ),
    }
}

/// Skyward's own update: the build running now; Skyward's newest release
/// on GitHub, and updating to it; then what to note before updating.
pub fn skyward_update(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["本机", "本機", "This phone"]));
    let y = widgets::info_card(&mut c, t, f, y, &[(l.pick(["当前版本", "目前版本", "Current version"]), RUNNING.to_string())]);
    let y = widgets::section(&mut c, t, f, y, l.pick(["在线更新", "線上更新", "Online update"]));
    let y = skyward_online(&mut c, cx, y, &mut out);
    // What the release says it changed, cut to the few lines that fit here.
    if let Some(newest) = cx.state.skyward_newest()
        && !newest.notes.is_empty()
    {
        let points: Vec<&str> = newest.notes.iter().map(String::as_str).collect();
        let y2 = widgets::notes(&mut c, t, f, y + t.m.card_gap, l.pick(["本次更新", "本次更新", "What changed"]), &points);
        return Frame::new(t, fit(c, t, y2), out, l.pick(SKYWARD_UPDATE), top);
    }
    let points = [
        l.pick([
            "新版本从 GitHub 下载，校验 SHA-256 后写入 recovery 分区，不会改动系统和用户数据。",
            "新版本從 GitHub 下載，驗證 SHA-256 後寫入 recovery 分割區，不會改動系統和使用者資料。",
            "The new version is downloaded from GitHub and goes into the recovery partitions once its SHA-256 checks out. The system and user data are left as they are.",
        ]),
        l.pick([
            "新版本将写入下次启动所用槽位的 recovery 分区；另一个槽位装有 Skyward 时，也会一并更新。",
            "新版本將寫入下次啟動所用槽位的 recovery 分割區；另一個槽位裝有 Skyward 時，也會一併更新。",
            "The new version goes into the recovery partition of the slot that starts next, and into the other slot's where it holds Skyward.",
        ]),
        l.pick(["更新完成后，重启到恢复模式即可使用新版本。", "更新完成後，重新啟動到復原模式即可使用新版本。", "Once updated, reboot to recovery to use the new version."]),
    ];
    let y = widgets::notes(&mut c, t, f, y + t.m.card_gap, l.pick(NOTES), &points);
    Frame::new(t, fit(c, t, y), out, l.pick(SKYWARD_UPDATE), top)
}

/// Asking GitHub for Skyward's newest release, then updating to it: its
/// build for the phone downloaded and written into the recovery partitions,
/// how far that has come, and how it ended.
fn skyward_online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let gap = t.m.card_gap;
    let newest = match &s.skyward_check {
        SkywardCheck::Asking => return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom,
        SkywardCheck::Answered(n) => n,
        other => {
            let mut row = Row::new(l.pick(CHECK_FOR_UPDATES));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckSkyward]));
            return match other {
                SkywardCheck::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                _ => check.bottom,
            };
        }
    };
    let version = &newest.version;
    let (title, detail) = match (newest.newer, &newest.build) {
        (true, Some(build)) => {
            let size = fmt::file_size(build.size);
            (tr!(l, "新版本 {version}", "新版本 {version}", "New version {version}"), tr!(l, "安装包 {size}", "安裝包 {size}", "Package {size}"))
        }
        (true, None) => (
            tr!(l, "新版本 {version}", "新版本 {version}", "New version {version}"),
            l.pick(["没有本机的安装包", "沒有本機的安裝包", "No package for this phone"]).to_string(),
        ),
        (false, _) => (
            l.pick(["已是最新版本", "已是最新版本", "Up to date"]).to_string(),
            tr!(l, "最新发布的版本为 {version}", "最新發佈的版本為 {version}", "The newest release is {version}"),
        ),
    };
    let y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Install).subtitle(&detail).trail(Trail::None)]).bottom;
    if !newest.newer || newest.build.is_none() {
        return y;
    }
    match &s.skyward_job {
        Some(job) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.stage {
                None => (l.pick(GETTING_READY), String::new(), None),
                Some(SkywardStage::Download(p)) => downloaded(l, p, job.rate),
                Some(SkywardStage::Write { done, total }) => {
                    let part = share(done, total);
                    let p = percent(part);
                    (l.pick(["正在写入 recovery 分区", "正在寫入 recovery 分割區", "Writing the recovery partitions"]), tr!(l, "已完成 {p}%", "已完成 {p}%", "{p}% done"), Some(part))
                }
            };
            let y = widgets::progress_card(c, t, f, y + gap, heading, &detail, share);
            if matches!(job.stage, Some(SkywardStage::Write { .. })) {
                return widgets::footnote(c, t, f, y, l.pick(["写入完成前，请不要关机。", "寫入完成前，請不要關機。", "Don't power off until it's written."]));
            }
            let stop = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(if job.stopping { STOPPING } else { STOP })).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopSkywardUpdate]));
            }
            stop.bottom
        }
        Some(SkywardJob { outcome: Some(SkywardEnd::Updated(slots)), .. }) => {
            let names = fmt::slots(l, slots);
            let detail = match (l, slots.len()) {
                (Lang::En, 1) => format!("Skyward {version} is in {names}'s recovery partition."),
                _ => tr!(
                    l,
                    "已将 Skyward {version} 刷入{names} 的 recovery 分区。",
                    "已將 Skyward {version} 刷入{names} 的 recovery 分割區。",
                    "Skyward {version} is in the recovery partitions of {names}."
                ),
            };
            let y = widgets::list(c, t, f, y + gap, &[standing(l.pick(["更新成功", "更新成功", "Updated"]), GREEN, Icon::Install, &detail)]).bottom;
            let restart = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(["重启到恢复模式", "重新啟動到復原模式", "Reboot to recovery"])).icon(ORANGE, Icon::Power)]);
            out.extend(hits(&restart.rows, &[Action::RebootTo(Target::Recovery)]));
            restart.bottom
        }
        job => {
            let mut y = y;
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(SkywardEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(["已停止下载。", "已停止下載。", "Download stopped."])),
                Some(SkywardEnd::Download(why)) => y = widgets::footnote(c, t, f, y, &fmt::skyward_download_failure(l, why)),
                Some(SkywardEnd::NotWritten(why)) => {
                    y = widgets::footnote(c, t, f, y, l.pick(["更新失败，未写入任何内容。", "更新失敗，未寫入任何內容。", "Couldn't update. Nothing was written."]));
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::WriteFailed(slots, why)) => {
                    let names = fmt::slots(l, slots);
                    let warning = match (l, slots.len()) {
                        (Lang::En, 1) => format!(
                            "{}'s recovery partition was only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done.",
                            capitalized(l, &names)
                        ),
                        _ => tr!(
                            l,
                            "{names} 的 recovery 分区仅写入了部分内容，重启到恢复模式可能无法进入 Skyward。请重新更新，完成后再重启。",
                            "{names} 的 recovery 分割區僅寫入了部分內容，重新啟動到復原模式可能無法進入 Skyward。請重新更新，完成後再重新啟動。",
                            "The recovery partitions of {names} were only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done."
                        ),
                    };
                    y = widgets::footnote(c, t, f, y, &warning);
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::Updated(_)) => unreachable!("an update that went in shows how it did"),
                None => {}
            }
            let mut row = Row::new(l.pick(["下载并更新", "下載並更新", "Download and update"]));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let update = widgets::list(c, t, f, y + gap, &[row]);
            out.extend(hits(&update.rows, &[Action::UpdateSkyward]));
            update.bottom
        }
    }
}

/// The WLAN switch; then the network chosen and how joining it goes, and
/// the networks in range, strongest first.
pub fn wlan(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let w = s.wlan.as_ref().expect("the WLAN page shows on a phone whose WLAN is brought up");
    // How many rows the page lays, counted before the canvas is asked for: the
    // phone's scan can list dozens of networks, and a page given three screens
    // whatever it holds leaves the last of them and the row that adds one by
    // hand past the end of the canvas, where no scrolling reaches them
    // (2026-11-25, what the user saw pulling the list to its end).
    let nearby_rows = w.networks.iter().filter(|n| w.joined.as_ref().is_none_or(|j| j.ssid != n.ssid)).count();
    let laid = 3 + usize::from(w.joined.is_some()) + s.wlan_saved.len() + nearby_rows + 1;
    let mut c = long_page_rows(cx, laid);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let on = !matches!(w.power, Power::Off | Power::Failed(_));
    // The WLAN's switch, and the two settings that go with it: opened at
    // start, and the password kept beside the network it was joined with.
    let switch = widgets::list(&mut c, t, f, top, &[
        Row::new(l.pick(WLAN)).trail(Trail::Switch(on)),
        Row::new(l.pick(WLAN_AUTO)).trail(Trail::Switch(s.wlan_auto)),
        Row::new(l.pick(WLAN_KEEP)).trail(Trail::Switch(s.wlan_keep)),
    ]);
    out.extend(hits(&switch.rows, &[Action::Toggle(Switch::Wlan), Action::Toggle(Switch::WlanAuto), Action::Toggle(Switch::WlanKeep)]));
    let mut y = switch.bottom;
    match &w.power {
        Power::Off => {
            let note = l.pick([
                "开启后，可连接 WLAN 下载系统更新和 Skyward 新版本。",
                "開啟後，可連接 Wi-Fi 下載系統更新和 Skyward 新版本。",
                "When on, system updates and new versions of Skyward can download over Wi-Fi.",
            ]);
            y = widgets::footnote(&mut c, t, f, y, note)
        }
        Power::Failed(why) => y = widgets::footnote(&mut c, t, f, y, &tr!(l, "无法开启 WLAN：{why}", "無法開啟 Wi-Fi：{why}", "Couldn't turn on Wi-Fi: {why}")),
        Power::Starting => y = widgets::footnote(&mut c, t, f, y, l.pick(["正在开启 WLAN", "正在開啟 Wi-Fi", "Turning on Wi-Fi"])),
        Power::On => {
            let locked = |s: Security| !matches!(s, Security::Open | Security::Owe);
            // The network chosen stands out in the accent, as the stock
            // page shows the one it is on.
            if let Some(j) = &w.joined {
                let found = w.networks.iter().find(|n| n.ssid == j.ssid);
                let (signal, bands) = match (&j.stage, found) {
                    (Joining::Connected(link), found) => (Some(link.signal), found.map_or(Bands::of(link.frequency), |n| n.bands)),
                    (_, Some(n)) => (Some(n.signal), n.bands),
                    (_, None) => (None, Bands::default()),
                };
                let tag = fmt::band_tag(bands);
                let mut row = Row::new(&j.name).fan(signal.map_or(0, fmt::signal_level)).subtitle(fmt::joining(l, &j.stage));
                row = row.trail(Trail::Network { locked: locked(j.security) });
                if let Some(tag) = &tag {
                    row = row.tag(tag);
                }
                let joined = widgets::list_toned(&mut c, t, f, y + t.m.card_gap, &[row], &Tone::lit(t));
                out.extend(hits(&joined.rows, &[Action::OpenJoined]));
                out.push(Hit { rect: widgets::round_button(t, joined.rows[0]), action: Action::OpenJoined });
                y = joined.bottom;
                if let Joining::Connected(link) = &j.stage
                    && let Some(note) = link.reach.as_ref().and_then(|r| fmt::reach(l, r))
                {
                    y = widgets::footnote(&mut c, t, f, y, &note);
                }
            }
            // 记住的那些网：到了有其中一张的地方，点一下就用记住的密码连它，
            // 不必等扫描把它列出来（2026-11-24 用户要的「在哪里就连哪里的
            // WiFi」）。名字在前、锁在后，已连的那张不再重复。
            // 已连的那张也列在这里（它是名单里的第一张），副标题写「已连接」：
            // 只记着一张的时候，这一节也不至于整节消失（2026-11-24）。
            let kept: Vec<(usize, &crate::app::SavedWlan)> = s.wlan_saved.iter().enumerate().collect();
            // 只有真的连上了才写「已连接」：正在连、找不到、密码错都不算。
            let on = w
                .joined
                .as_ref()
                .filter(|j| matches!(j.stage, Joining::Connected(_)))
                .map(|j| j.ssid.clone());
            if !kept.is_empty() {
                let head = l.pick(["已保存的网络", "已保存的網路", "Saved networks"]);
                y = widgets::section(&mut c, t, f, y + t.m.card_gap, head);
                let rows: Vec<Row> = kept
                    .iter()
                    .map(|(_, n)| {
                        let mut row = Row::new(&n.name).trail(Trail::Network { locked: locked(n.security) });
                        if on.as_deref() == Some(n.ssid.as_slice()) {
                            row.sub = Some(l.pick(["已连接", "已連線", "Connected"]));
                        }
                        row
                    })
                    .collect();
                let list = widgets::list(&mut c, t, f, y, &rows);
                out.extend(hits(&list.rows, &kept.iter().map(|(k, _)| Action::JoinKept(*k)).collect::<Vec<_>>()));
                // 行右端那枚圆钮进它的详情：在那里能删掉这张（2026-11-24 用户
                // 说已保存的点进去只会连，删不掉）。
                for (rect, (k, _)) in list.rows.iter().zip(&kept) {
                    out.push(Hit { rect: widgets::round_button(t, *rect), action: Action::KeptDetails(*k) });
                }
                y = widgets::footnote(
                    &mut c,
                    t,
                    f,
                    list.bottom,
                    l.pick([
                        "点一下用保存的密码直接连；密码可以在上面「记住密码」关掉。",
                        "點一下用儲存的密碼直接連；密碼可以在上面「記住密碼」關掉。",
                        "Tap one to join with the password kept; the switch above forgets them all.",
                    ]),
                );
            }
            let others: Vec<(usize, &Network)> =
                w.networks.iter().enumerate().filter(|(_, n)| w.joined.as_ref().is_none_or(|j| j.ssid != n.ssid)).collect();
            let nearby = l.pick(["选取附近的 WLAN", "選擇附近的 Wi-Fi 網路", "Choose a nearby network"]);
            let (below, rescan) = widgets::section_action(&mut c, t, f, y, nearby, l.pick(["刷新", "重新整理", "Refresh"]));
            out.push(Hit { rect: rescan, action: Action::Rescan });
            y = below;
            if others.is_empty() {
                let looking = l.pick(["正在搜索附近的 WLAN", "正在搜尋附近的 Wi-Fi 網路", "Looking for nearby networks"]);
                y = widgets::footnote(&mut c, t, f, y - t.m.dp(10.0), looking) + t.m.card_gap;
            }
            let tags: Vec<Option<String>> = others.iter().map(|(_, n)| fmt::band_tag(n.bands)).collect();
            let mut rows: Vec<Row> = others
                .iter()
                .zip(&tags)
                .map(|((_, n), tag)| {
                    let row = Row::new(&n.name).fan(fmt::signal_level(n.signal)).trail(Trail::Network { locked: locked(n.security) });
                    match tag {
                        Some(tag) => row.tag(tag),
                        None => row,
                    }
                })
                .collect();
            // The list ends with adding a network by hand, as the stock one
            // does.
            rows.push(Row::new(l.pick(ADD_NETWORK)).tint(t.pal.accent).trail(Trail::None));
            let list = widgets::list(&mut c, t, f, y, &rows);
            let mut picks: Vec<Action> = others.iter().map(|(k, _)| Action::JoinNetwork(*k)).collect();
            picks.push(Action::AddNetwork);
            out.extend(hits(&list.rows, &picks));
            // The round buttons come after the rows they sit on, and a touch
            // finds the last control under it.
            for (rect, (k, _)) in list.rows.iter().zip(&others) {
                out.push(Hit { rect: widgets::round_button(t, *rect), action: Action::NetworkDetails(*k) });
            }
            y = list.bottom;
        }
    }
    Frame::new(t, fit(c, t, y), out, l.pick(WLAN), top)
}

const ADD_NETWORK: Text = ["添加网络", "新增網路", "Add network"];
const SHOW_PASSWORD: Text = ["显示密码", "顯示密碼", "Show password"];
const CONNECT_KEY: Text = ["连接", "連線", "Connect"];
const SECURITY: Text = ["安全性", "安全性", "Security"];

/// The password of the network chosen: its name, what is typed, a switch
/// to show it, and the keyboard.
pub fn wlan_password(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let network = s.joining.as_ref().expect("the password page is for a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
    let (y, caret) = widgets::text_field(&mut c, t, f, y, &shown);
    let show = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
    out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    keyboard_hits(&mut c, t, f, s, l.pick(CONNECT_KEY), false, &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(["输入密码", "輸入密碼", "Enter password"]), top) }
}

/// A network added by hand: its name, how it is protected, its password
/// where it takes one with the switch that shows it, and the keyboard,
/// which types into the field touched. The protections open over the page
/// from their row, which alone then answers a touch.
pub fn wlan_add(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let a = s.adding.as_ref().expect("the page adds a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = m.content_top;
    let field = |y: f32| Rect::new(m.margin, y, m.w as f32 - m.margin * 2.0, m.dp(56.0));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["网络名称", "網路名稱", "Network name"]));
    let typing = with_preview(s, &a.name);
    let (below, mut caret) = widgets::text_field(&mut c, t, f, y, &typing);
    out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Name) });
    // Held down while its list is open, as the stock row stays.
    let tone = if a.choosing { Tone::page(&t.pressed()) } else { Tone::page(t) };
    let protection = [Row::new(l.pick(SECURITY)).trail(Trail::Choice(fmt::security(l, a.security)))];
    let security = widgets::list_toned(&mut c, t, f, below + m.card_gap, &protection, &tone);
    out.extend(hits(&security.rows, &[Action::ChooseSecurity]));
    if a.security.password() {
        let y = widgets::section(&mut c, t, f, security.bottom, l.pick(["密码", "密碼", "Password"]));
        let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
        let (below, typed) = widgets::text_field(&mut c, t, f, y, &shown);
        out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Password) });
        if a.field == Field::Password {
            caret = typed;
        }
        let show = widgets::list(&mut c, t, f, below + m.card_gap, &[Row::new(l.pick(SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
        out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    }
    // 只有「名称」那一格收汉字；密码格不按拼音打，但按了「中/英」照样能切中文键盘。
    keyboard_hits(&mut c, t, f, s, l.pick(CONNECT_KEY), a.field == Field::Name, &mut out);
    if !a.choosing {
        return Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(ADD_NETWORK), top) };
    }
    // A touch anywhere off the list closes it.
    out.clear();
    out.push(Hit { rect: Rect::new(0.0, 0.0, m.w as f32, m.h as f32), action: Action::CloseChoices });
    let names = ADDABLE.map(|p| fmt::security(l, p));
    let chosen = ADDABLE.iter().position(|&p| p == a.security);
    let held = ADDABLE.iter().position(|&p| s.held == Some(Action::PickSecurity(p)));
    let list = widgets::choices(&mut c, t, f, security.rows[0], &names, chosen, held);
    out.extend(hits(&list, &ADDABLE.map(Action::PickSecurity)));
    Frame::new(t, c, out, l.pick(ADD_NETWORK), top)
}

/// A network's details, in the stock page's grid: for the one chosen, how
/// joining it goes and what it gave, and forgetting it; for any other, how
/// strong and how protected it is.
pub fn wlan_network(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let network = s.details.as_ref().expect("the details page is of a network");
    let joined = s.wlan.as_ref().and_then(|w| w.joined.as_ref()).filter(|j| j.ssid == network.ssid);
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let detail = |icon, title, value: String| Detail { icon, title, value };
    let security = detail(Outline::Shield, l.pick(SECURITY), fmt::security(l, network.security).to_string());
    let signal = l.pick(["信号强度", "訊號強度", "Signal strength"]);
    let title = l.pick(["网络详情", "網路詳細資料", "Network details"]);
    let Some(j) = joined else {
        let items = [
            detail(Outline::Wifi, l.pick(["频段", "頻段", "Bands"]), fmt::bands(l, network.bands)),
            detail(Outline::Signal, signal, fmt::signal_known(l, network.signal)),
            security,
        ];
        let y = widgets::detail_grid(&mut c, t, f, y, &items);
        // 没连着的这一张（多半只是名单里记着的）：删除也要在这里给出来，否则
        // 已保存的网络根本删不掉（2026-11-24 用户报的）。
        let forget = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(["删除网络", "刪除網路", "Forget network"])).tint(t.pal.danger).trail(Trail::None)]);
        out.extend(hits(&forget.rows, &[Action::LeaveNetwork]));
        return Frame::new(t, fit(c, t, forget.bottom), out, title, top);
    };
    let mut items = vec![detail(Outline::Info, l.pick(["状态信息", "狀態資訊", "Status"]), fmt::joining(l, &j.stage).to_string())];
    match &j.stage {
        Joining::Connected(link) => {
            if let Some(n) = link.generation {
                items.push(detail(Outline::Wifi, l.pick(["技术标准", "技術標準", "Wi-Fi standard"]), fmt::generation(l, n)));
            }
            if let Some(speed) = link.speed {
                items.push(detail(Outline::Speed, l.pick(["连接速度", "連線速度", "Link speed"]), format!("{speed} Mbps")));
            }
            items.push(detail(Outline::Signal, signal, fmt::signal(l, link.signal).to_string()));
            items.push(security);
            items.push(detail(Outline::Address, l.pick(["IP 地址", "IP 位址", "IP address"]), link.address.to_string()));
            items.push(detail(Outline::Mask, l.pick(["子网掩码", "子網路遮罩", "Subnet mask"]), link.mask.to_string()));
            if let Some(router) = link.router {
                items.push(detail(Outline::Router, l.pick(["路由器", "路由器", "Router"]), router.to_string()));
            }
        }
        _ => {
            items.push(detail(Outline::Signal, signal, fmt::signal_known(l, network.signal)));
            items.push(security);
        }
    }
    let y = widgets::detail_grid(&mut c, t, f, y, &items);
    let forget = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(["删除网络", "刪除網路", "Forget network"])).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&forget.rows, &[Action::LeaveNetwork]));
    Frame::new(t, fit(c, t, forget.bottom), out, title, top)
}

/// The file manager, every entry of the folder shown, folders first.
/// The files on the OTG drive, all of them: folders and files alike.
///
/// An entry picked in the file manager is put down here from the bar held at
/// the bottom, so a file on the phone is copied onto the drive.
pub fn files_otg(cx: &Ctx) -> Frame {
    let l = cx.state.lang;
    let top = l.pick(["OTG", "OTG", "OTG"]);
    let dock = cx.state.clip.as_ref().map(|clip| {
        let go = l.pick(match clip.op {
            FileOp::Copy => ["复制到此处", "複製到這裡", "Copy here"],
            FileOp::Move => ["移动到此处", "移動到這裡", "Move here"],
        });
        let (canvas, rects) = widgets::dock_bar(cx.t, cx.f, files::page(cx.t), &[l.pick(CANCEL), go]);
        Dock { canvas, hits: hits(&rects, &[Action::Unpick, Action::Paste]) }
    });
    let frame = listing(
        cx,
        &cx.state.otg,
        top,
        None,
        l.pick(["U 盘为空", "USB 隨身碟是空的", "The drive is empty"]),
        Action::OpenOtgEntry,
        top,
        None,
        None,
        &[(l.pick(["内部存储", "內部儲存空間", "Internal storage"]), Action::Open(Screen::Files))],
        &[],
    );
    Frame { dock, ..frame }
}

pub fn files(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // 排序的面板是这一页上的另一屏：返回键先关它，不离开文件管理。
    if s.sort_panel {
        return sort_page(cx);
    }
    // An entry picked to copy or move goes to the folder shown, from the
    // bar held at the bottom.
    let picking = s.picking.then(|| {
        let use_here = l.pick(["用这个文件夹", "用這個資料夾", "Serve this folder"]);
        let (canvas, rects) = widgets::dock_bar(t, f, files::page(t), &[use_here]);
        Dock { canvas, hits: hits(&rects, &[Action::ServeUseHere]) }
    });
    let dock = picking.or_else(|| s.clip.as_ref().map(|clip| {
        let go = l.pick(match clip.op {
            FileOp::Copy => ["复制到此处", "複製到這裡", "Copy here"],
            FileOp::Move => ["移动到此处", "移動到這裡", "Move here"],
        });
        let (canvas, rects) = widgets::dock_bar(t, f, files::page(t), &[l.pick(CANCEL), go]);
        Dock { canvas, hits: hits(&rects, &[Action::Unpick, Action::Paste]) }
    }));
    // Choosing several entries: the bar at the foot carries what can be done
    // with them, and the page's own button starts and stops the choosing.
    let choosing = s.choosing.then(|| {
        let all = l.pick(["全选", "全選", "All"]);
        let none = l.pick(["全不选", "全不選", "None"]);
        let copy = l.pick(["复制", "複製", "Copy"]);
        let cut = l.pick(["移动", "移動", "Move"]);
        let delete = l.pick(["删除", "刪除", "Delete"]);
        let (canvas, rects) = widgets::dock_bar_danger(t, f, files::page(t), &[all, copy, cut, delete], Some(3));
        let _ = none;
        Dock { canvas, hits: hits(&rects, &[Action::ChooseAll, Action::PasteChosen(FileOp::Copy), Action::PasteChosen(FileOp::Move), Action::AskDeleteChosen]) }
    });
    // 排序收进面板：面板是这一页上的整页（底部的 dock 在真机上命中不上，
    // 改成整页之后每一行都由页面自己的命中框回答，2026-11-24）。
    let dock = choosing.or(dock);
    // 右上角那个终端圆钮去掉：顶栏那四枚里有一枚放大镜，终端留在这里并不比
    // 它常用，而顶栏原来空着（2026-11-24）。
    let button = None;
    let root = l.pick(["内部存储", "內部儲存空間", "Internal storage"]);
    // 已经在 /sdcard 里就不用再画这一枚；U 盘改挂到顶栏那四枚里。
    let card = l.pick(["sdcard", "sdcard", "sdcard"]);
    let on_card = s.files.path.trim_end_matches('/') == "/sdcard";
    let places: Vec<(&str, Action)> = match on_card {
        true => Vec::new(),
        false => vec![(card, Action::FilesSdCard)],
    };
    let entry_action: fn(usize) -> Action = if s.choosing { Action::ChooseEntryAt } else { Action::OpenEntry };
    // 四枚圆钮：排序、多选、U 盘、搜索，排在 sdcard 那一行的右端。
    let chips: [(Icon, Action, bool); 4] = [
        (Icon::Sort, Action::SortPanel(true), s.sort_panel),
        (Icon::Choose, Action::ChooseMany(!s.choosing), s.choosing),
        (Icon::UsbDrive, Action::FilesOtg, false),
        (Icon::Search, Action::SearchFiles, false),
    ];
    let frame = listing(
        cx,
        &s.files,
        root,
        None,
        l.pick(["暂无文件", "沒有檔案", "No files"]),
        entry_action,
        l.pick(FILES),
        None,
        None,
        &places,
        &chips,
    );
    Frame { dock, button, ..frame }
}

/// 排序的那一屏：按什么排、升序还是降序，一点就排好并回到文件列表。
fn sort_page(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = page(cx);
    let top = m.content_top;
    let keys: [(&str, SortKey); 4] = [
        (l.pick(["名称", "名稱", "Name"]), SortKey::Name),
        (l.pick(["大小", "大小", "Size"]), SortKey::Size),
        (l.pick(["修改时间", "修改時間", "Modified"]), SortKey::Modified),
        (l.pick(["类型", "類型", "Type"]), SortKey::Type),
    ];
    let sides: [(&str, bool); 2] = [
        (l.pick(["升序 ↑", "升序 ↑", "Up ↑"]), false),
        (l.pick(["降序 ↓", "降序 ↓", "Down ↓"]), true),
    ];
    let mut rows: Vec<Row> = Vec::new();
    for (label, key) in keys {
        rows.push(Row::new(label).trail(Trail::Check(s.sort == key)));
    }
    for (label, desc) in sides {
        rows.push(Row::new(label).trail(Trail::Check(s.sort_desc == desc)));
    }
    let block = widgets::list(&mut c, t, f, top, &rows);
    let mut out = Vec::new();
    for (k, rect) in block.rows.iter().enumerate() {
        let action = match keys.get(k) {
            Some((_, key)) => Action::SortTo(*key),
            None => Action::SortDesc(sides[k - keys.len()].1),
        };
        out.push(Hit { rect: *rect, action });
    }
    // 「文件夹置顶 / 混合排列」：照旧一项可切，和另外两件事放在同一屏。
    let first = l.pick(match s.folders_first {
        true => ["文件夹置顶", "資料夾置頂", "Folders first"],
        false => ["混合排列", "混合排列", "Mixed"],
    });
    let extra = widgets::list(&mut c, t, f, block.bottom + m.card_gap, &[Row::new(first).trail(Trail::Check(s.folders_first))]);
    out.extend(hits(&extra.rows, &[Action::ToggleFoldersFirst]));
    // 底下一枚「完成」：不改什么也能回去。
    let y = extra.bottom + m.dp(20.0);
    let px = m.dp(17.0);
    let h_row = m.dp(44.0);
    let w = c.text_width(&f.value, l.pick(DONE), px) + m.dp(56.0);
    let x = (m.w as f32 - w) / 2.0;
    widgets::pill_sized(&mut c, f, x, y, h_row, px, m.dp(20.0), l.pick(DONE), t.pal.search, t.pal.text);
    out.push(Hit { rect: Rect::new(x, y, w, h_row), action: Action::SortPanel(false) });
    let bottom = y + h_row;
    Frame::new(t, fit(c, t, bottom + m.dp(24.0)), out, l.pick(["排序", "排序", "Sort"]), top)
}

const CANCEL: Text = ["取消", "取消", "Cancel"];

/// A place's folders and the files in them that can be installed, to
/// choose one. A USB drive's page waits for one to be plugged in, and the
/// drive's files show as soon as it is.
pub fn choose(cx: &Ctx, place: Place) -> Frame {
    let (s, l) = (cx.state, cx.state.lang);
    let unreadable;
    let absent = match (place, &s.usb_drive) {
        (Place::UsbDrive, None) => Some((
            l.pick(["未连接 U 盘", "未連接 USB 隨身碟", "No USB drive"]),
            l.pick(["通过 OTG 连接 U 盘后，将在此显示其中的文件。", "透過 OTG 連接 USB 隨身碟後，將在此顯示其中的檔案。", "Connect a USB drive through OTG to see its files here."]),
        )),
        (Place::UsbDrive, Some(UsbDrive::Unreadable(why))) => {
            unreadable = tr!(l, "请使用 FAT32 或 exFAT 格式的 U 盘。{why}", "請使用 FAT32 或 exFAT 格式的 USB 隨身碟。{why}", "Use a USB drive formatted as FAT32 or exFAT. {why}");
            Some((l.pick(USB_DRIVE_UNREADABLE), unreadable.as_str()))
        }
        _ => None,
    };
    let empty = l.pick(["暂无可安装的文件", "沒有可安裝的檔案", "Nothing here to install"]);
    // The page that chooses a file has no order of its own: it shows the same
    // three controls, which turn the order of the file manager it came from.
    listing(cx, s.folder(Screen::Choose(place)), place_name(l, place), absent, empty, Action::ChooseEntry, l.pick(["选择文件", "選擇檔案", "Choose a file"]), None, None, &[], &[])
}

/// A folder laid out as the stock file manager lays one out: the path as
/// capsules from `top`, held under the bar once scrolled there, then the
/// entries in one list on the plain page, each row drawn as it scrolls into
/// view and touched for `action`. In place of the list, `absent` says why
/// nothing of the folder can be shown, and `empty` that it holds nothing.
fn listing(
    cx: &Ctx,
    folder: &Folder,
    top: &str,
    absent: Option<(&str, &str)>,
    empty: &str,
    action: fn(usize) -> Action,
    title: &'static str,
    order: Option<[&str; 3]>,
    extra: Option<(&str, Action)>,
    // The other places the files can be shown from, beside the trail.
    places_other: &[(&str, Action)],
    // 顶栏那四枚圆钮（排序、多选、U 盘、搜索）：画在路径条右端，返回它们的命中框。
    chips: &[(Icon, Action, bool)],
) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let bg = files::page(t);
    let mut c = sheet(cx, t.m.h, bg);
    let rows = match (absent, &folder.entries) {
        (Some((note, why)), _) => {
            files::note(&mut c, t, f, note, Some(why));
            None
        }
        (None, Err(why)) => {
            files::note(&mut c, t, f, cx.state.lang.pick(["无法读取该文件夹", "無法讀取這個資料夾", "This folder can't be read"]), Some(why));
            None
        }
        (None, Ok(entries)) if entries.is_empty() => {
            files::note(&mut c, t, f, empty, None);
            None
        }
        (None, Ok(entries)) => Some(files::rows(t, f, entries, action)),
    };
    let pinned = Some(files::strip_with(t, f, top, &folder.path, order, extra, places_other, chips));
    Frame { bg, rows, pinned, ..Frame::new(t, c, Vec::new(), title, t.m.dp(files::LIST_TOP_DP)) }
}

/// Row `i` of the page's long list over the whole of `c`, sized for it,
/// drawn as the row comes into view.
pub fn row(screen: Screen, cx: &Ctx, i: usize, c: &mut Canvas) {
    match screen {
        // 应用管理的一行：名字在上，包名在下，图标统一一个。
        Screen::DataApps => {
            if let Some(Ok(list)) = &cx.state.apps
                && let Some(a) = list.get(i) {
                    // 名字只占一行：超了就截断加省略号，行高才稳。
                    let m = &cx.t.m;
                    let text_x = m.inner + m.icon + m.icon_gap;
                    let room = m.w as f32 - m.margin * 2.0 - text_x - m.inner;
                    // 二分找能放下的前缀：一个字符一个字符地试，长包名会一次
                    // 量上百遍，翻页就卡那一下（2026-11-18）。
                    let bounds: Vec<usize> = a.label.char_indices().map(|(i, _)| i).collect();
                    let fits = |end: usize| c.text_width(&cx.f.heading, &a.label[..end], m.row_title_px) <= room;
                    let mut lo = 0usize;
                    let mut hi = bounds.len() + 1;
                    while lo + 1 < hi {
                        let mid = (lo + hi) / 2;
                        let end = bounds.get(mid).copied().unwrap_or(a.label.len());
                        if fits(end) {
                            lo = mid;
                        } else {
                            hi = mid;
                        }
                    }
                    let cut = bounds.get(lo).copied().unwrap_or(a.label.len());
                    let label = a.label[..cut].to_string();
                    let mut row = Row::new(&label).icon(PURPLE, Icon::Layers);
                    row.sub = Some(&a.package);
                    widgets::draw_one(c, cx.t, cx.f, &row);
                }
        }
        Screen::Files | Screen::FilesOtg | Screen::Choose(_) => {
            let folder = cx.state.folder(screen);
            let entries = folder.entries.as_ref().expect("a folder listed draws its rows");
            let chosen = cx.state.choosing && cx.state.chosen.contains(&entries[i].name);
            files::row_marked(c, cx.t, cx.f, cx.state.lang, &entries[i], folder.listed, cx.state.zone(), chosen);
        }
        Screen::Installing => widgets::output_row(c, cx.t, cx.f, &cx.state.script_output, i),
        Screen::PartWork => {
            let work = cx.state.part_work.as_ref().expect("the page of partition work shows work");
            widgets::output_row(c, cx.t, cx.f, &work.output, i)
        }
        other => unreachable!("{other:?} draws no list row by row"),
    }
}

/// One entry of a folder, a file or a folder: what it is, and what can be
/// done with it. Deleting asks first, with a swipe.
pub fn file_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the file page shows an entry");
    let folder = matches!(e.kind, FileKind::Folder { .. });
    let mut c = long_page(cx);
    let m = &t.m;
    let (w, h) = (m.w as f32, m.h as f32);
    let side = m.margin;
    let top = m.content_top;
    let (card, name_ink, note_ink, pill_bg, pill_ink) = if t.dark {
        (
            Rgba::rgb(28, 28, 30),
            Rgba::rgb(242, 242, 242),
            Rgba::rgb(128, 128, 128),
            Rgba::rgb(38, 38, 38),
            Rgba::rgb(214, 216, 222),
        )
    } else {
        (
            Rgba::rgb(246, 247, 250),
            Rgba::rgb(0, 0, 0),
            Rgba::rgb(102, 102, 102),
            Rgba::rgb(238, 240, 245),
            Rgba::rgb(40, 44, 52),
        )
    };

    // 这一个文件能做些什么。能装能改的排在最前面，删除永远在最后一格、红字。
    let mut pills: Vec<(String, Action, bool)> = Vec::new();
    if installs(e) {
        let image = Path::new(&e.name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("img"));
        pills.push((l.pick(if image { ["刷入", "刷入", "Flash"] } else { ["安装", "安裝", "Install"] }).to_string(), Action::InstallEntry, false));
    }
    if e.kind == FileKind::File {
        pills.push((l.pick(["编辑", "編輯", "Edit"]).to_string(), Action::EditEntry, false));
    }
    // 四排：复制/移动、复制到 U 盘/移动到 U 盘、重命名/文件权限、删除独占一行
    //（2026-11-17）。移动就是剪切，同一边挑目录；到另一边是一步直达。
    pills.push((l.pick(["复制", "複製", "Copy"]).to_string(), Action::PickEntry(FileOp::Copy), false));
    pills.push((l.pick(["移动", "移動", "Move"]).to_string(), Action::PickEntry(FileOp::Move), false));
    match s.file_on_drive {
        true => {
            pills.push((l.pick(["复制到手机", "複製到手機", "Copy to phone"]).to_string(), Action::ToPhone(FileOp::Copy), false));
            pills.push((l.pick(["移动到手机", "移動到手機", "Move to phone"]).to_string(), Action::ToPhone(FileOp::Move), false));
        }
        false => {
            pills.push((l.pick(["复制到 U 盘", "複製到 USB", "Copy to drive"]).to_string(), Action::ToDrive(FileOp::Copy), false));
            pills.push((l.pick(["移动到 U 盘", "移動到 USB", "Move to drive"]).to_string(), Action::ToDrive(FileOp::Move), false));
        }
    }
    // 一个 zip 还能当模块装：KSU / Magisk 的模块就是这个形状。
    if e.kind == FileKind::File && e.name.to_lowercase().ends_with(".zip") {
        pills.push((l.pick(["作为模块安装", "當作模組安裝", "Install as module"]).to_string(), Action::InstallAsModule, false));
    }
    pills.push((l.pick(RENAME).to_string(), Action::RenameEntry, false));
    if !matches!(e.kind, FileKind::Link { .. }) {
        pills.push((l.pick(["文件权限", "檔案權限", "Permissions"]).to_string(), Action::OpenPermissions, false));
    }
    pills.push((l.pick(["删除", "刪除", "Delete"]).to_string(), Action::AskDelete, true));

    // 等宽网格：三列（选项多的）或两列，格子一样宽、左右都对齐。删除单独
    // 一行、占满整行——它和别的不是一类事，挤在角落里反而像没排完。
    let (danger, others) = match pills.split_last() {
        Some((last, rest)) if last.2 => (Some(last.clone()), rest.to_vec()),
        _ => (None, pills.clone()),
    };
    // 列数按总格数挑，保证最多两行——三行排下来像没排完（2026-11-17）。
    // 删除也算一格，所以是「总格数的一半向上取整」。
    // 第一排四个，剩下的排第二排并**把整排均分**：第二排三个就三个各占
    // 三分之一，不留空档（2026-11-17）。
    // 每排两个、均匀铺满；删除单独占最后一整行（2026-11-17）。
    let all: Vec<(String, Action, bool)> = others.clone();
    let mut rows: Vec<(usize, usize)> = all
        .len()
        .checked_add(1)
        .map(|_| (0..all.len()).step_by(2).map(|k| (k, (k + 2).min(all.len()))).collect::<Vec<_>>())
        .unwrap_or_default();
    if danger.is_some() {
        rows.push((all.len(), all.len()));
    }
    let row_count = rows.len();
    let gap = m.dp(widgets::PILL_GAP_DP);
    let ph = m.dp(widgets::PILL_DP);
    let room = w - side * 2.0;
    let px = m.dp(widgets::PILL_TEXT_DP);
    let grid_px = m.dp(13.0);
    let grid_pad = m.dp(7.0);
    let inner = m.dp(18.0);
    let card_h = inner * 2.0 + m.dp(42.0);
    let title_px = m.dp(13.0);
    let inner_h = card_h;
    let sheet_h = m.dp(30.0) + row_count as f32 * ph + (row_count.saturating_sub(1)) as f32 * gap + m.dp(18.0);
    // 整块（信息卡 + 操作格）在页面里居中：全压在底部会让上半屏空得像没加载完。
    let block_h = inner_h + m.dp(12.0) + sheet_h;
    let block_top = ((h - block_h) / 2.0).max(top);
    let sheet_top = block_top + inner_h + m.dp(12.0);
    let card_top = block_top;

    // 上面这张卡只说一件事：正在操作哪一个。名字、大小或项数、改于何时。
    // 卡片按内容高，不抻到底：空着一大片会把视线拽走。
    // 整块贴着底部：卡片紧挨在操作格上面，像从下面升起来的一段
    //（空着的那一截留在页面标题之下，不把视线拉成两段）。
    c.fill_round_rect(side, card_top, room, card_h, m.card_radius, card);
    let cx = side + inner;
    let avatar = m.dp(42.0);
    let ay = card_top + inner;
    match e.kind {
        FileKind::Folder { .. } => {
            icons::glyph(&mut c, Icon::Folder, cx + avatar / 2.0, ay + avatar / 2.0 - m.dp(4.0), avatar * 0.8, Rgba::rgb(246, 196, 68));
        }
        _ => {
            let half = avatar * 0.78;
            c.fill_round_rect(cx + (avatar - half) / 2.0, ay + (avatar - half) / 2.0, half, half, m.dp(6.0), Rgba::rgb(120, 168, 255));
            let ext = Path::new(&e.name).extension().map(|x| x.to_string_lossy().to_uppercase()).unwrap_or_default();
            let ext: String = ext.chars().take(4).collect();
            if !ext.is_empty() {
                let pxl = m.dp(10.0);
                let tw = c.text_width(&f.value, &ext, pxl);
                c.text(&f.value, &ext, cx + avatar / 2.0 - tw / 2.0, ay + avatar / 2.0 + pxl * 0.36, pxl, Rgba::rgb(255, 255, 255), None);
            }
        }
    }
    let text_x = cx + avatar + m.dp(16.0);
    let text_room = side + room - inner - text_x;
    let title = cut_to(&c, &f.value, &e.name, m.dp(17.0), text_room);
    c.text(&f.value, &title, text_x, ay + m.dp(18.0), m.dp(17.0), name_ink, None);
    let holds = match &e.kind {
        FileKind::Folder { items: Some(n) } => match (l, *n) {
            (Lang::En, 1) => "1 item".to_string(),
            (_, n) => tr!(l, "{n} 项", "{n} 項", "{n} items"),
        },
        FileKind::Folder { items: None } => l.pick(UNREADABLE).to_string(),
        FileKind::Link { .. } => l.pick(["链接", "連結", "A link"]).to_string(),
        _ => fmt::file_size(e.size),
    };
    let when = fmt::datetime(l, e.modified, s.zone());
    c.text(&f.value, &format!("{holds} · {when}"), text_x, ay + m.dp(38.0), m.dp(13.0), note_ink, None);

    // 下面：一句标题，再一层等宽的药丸。
    c.text(&f.value, l.pick(["文件的操作", "檔案的操作", "What can be done"]), side, sheet_top + m.dp(20.0), title_px, note_ink, None);
    let mut rects: Vec<Rect> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    for (r, (from, to)) in rows.iter().enumerate() {
        let n = to - from;
        let y = sheet_top + m.dp(30.0) + r as f32 * (ph + gap);
        // 删除那一排：一格占满整行。
        if n == 0 {
            if let Some((label, action, _)) = &danger {
                c.fill_round_rect(side, y, room, ph, ph / 2.0, pill_bg);
                c.text_center(&f.value, label, side + room / 2.0, y + ph / 2.0, px, t.pal.danger);
                rects.push(Rect::new(side, y, room, ph));
                acts.push(*action);
            }
            continue;
        }
        let cell = (room - gap * (n as f32 - 1.0)) / n as f32;
        for (i, k) in (*from..*to).enumerate() {
            let (label, action, bad) = &all[k];
            let x = side + i as f32 * (cell + gap);
            c.fill_round_rect(x, y, cell, ph, ph / 2.0, pill_bg);
            let shown = cut_to(&c, &f.value, label, grid_px, cell - grid_pad * 2.0);
            c.text_center(&f.value, &shown, x + cell / 2.0, y + ph / 2.0, grid_px, if *bad { t.pal.danger } else { pill_ink });
            rects.push(Rect::new(x, y, cell, ph));
            acts.push(*action);
        }
    }
    let page_title = l.pick(if folder { ["文件夹信息", "資料夾資訊", "Folder info"] } else { ["文件信息", "檔案資訊", "File info"] });
    let bottom = sheet_top + sheet_h + m.dp(8.0);
    Frame::new(t, fit(c, t, bottom), hits(&rects, &acts), page_title, top)
}

/// `text` cut to `w` pixels, its middle taken out.
fn cut_to(c: &Canvas, face: &crate::font::Face, text: &str, px: f32, w: f32) -> String {
    let width = |t: &str| c.text_width(face, t, px);
    if width(text) <= w {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let room = w - width("…");
    let mut right = chars.len();
    while right > 0 && width(&chars[right - 1..].iter().collect::<String>()) <= room / 2.0 {
        right -= 1;
    }
    let left_room = room - width(&chars[right..].iter().collect::<String>());
    let mut left = 0;
    while left < right && width(&chars[..=left].iter().collect::<String>()) <= left_room {
        left += 1;
    }
    format!("{}…{}", chars[..left].iter().collect::<String>(), chars[right..].iter().collect::<String>())
}

const NAME: Text = ["名称", "名稱", "Name"];
const PERMISSIONS: Text = ["权限", "權限", "Permissions"];
const CHANGE_PERMISSIONS: Text = ["修改权限", "修改權限", "Change permissions"];
const RENAME: Text = ["重命名", "重新命名", "Rename"];

/// The permission bits of the entry the file page shows, a switch each that
/// takes effect at once: reading, writing and running for its owner, its
/// group and everyone else, then the set-ID and sticky bits.
pub fn permissions(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the permissions page shows an entry");
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mode = e.mode & 0o7777;
    let bits = fmt::mode(e.mode);
    let facts = [(l.pick(NAME), e.name.clone()), (l.pick(PERMISSIONS), tr!(l, "{bits}，{mode:04o}", "{bits}，{mode:04o}", "{bits}, {mode:04o}"))];
    let mut y = widgets::info_card(&mut c, t, f, top, &facts);
    let mut out = Vec::new();
    let classes = [(["所有者", "擁有者", "Owner"], 6), (["用户组", "群組", "Group"], 3), (["其他用户", "其他使用者", "Others"], 0)];
    let (read, write, run) = (l.pick(["读取", "讀取", "Read"]), l.pick(["写入", "寫入", "Write"]), l.pick(["执行", "執行", "Execute"]));
    let groups = classes.map(|(title, shift)| (l.pick(title), [(read, 4 << shift), (write, 2 << shift), (run, 1 << shift)]));
    let special = (l.pick(["特殊权限", "特殊權限", "Special"]), [("SUID", 0o4000), ("SGID", 0o2000), (l.pick(["粘滞位", "黏滯位", "Sticky"]), 0o1000)]);
    for (title, bits) in groups.into_iter().chain([special]) {
        y = widgets::section(&mut c, t, f, y, title);
        let rows = bits.map(|(name, bit)| Row::new(name).trail(Trail::Switch(mode & bit != 0)));
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(hits(&list.rows, &bits.map(|(_, bit)| Action::ModeBit(bit))));
        y = list.bottom;
    }
    Frame::new(t, fit(c, t, y), out, l.pick(CHANGE_PERMISSIONS), top)
}

/// A new name for the entry the file page shows, typed on the keyboard.
pub fn rename(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let typing = with_preview(s, &s.new_name);
    let (y, caret) = widgets::text_field(&mut c, t, f, top, &typing);
    widgets::footnote(&mut c, t, f, y, l.pick(["名称不能包含\"/\"。点击「完成」保存。", "名稱不能包含「/」。點選「完成」儲存。", "A name can't hold “/”. Tap Done to save it."]));
    keyboard_hits(&mut c, t, f, s, l.pick(DONE), true, &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(RENAME), top) }
}

/// The folder an entry goes to already holds its name: what to do.
pub fn clash(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let (clip, into, _) = s.clash.as_ref().expect("the clash page shows a clash");
    let name = Path::new(&clip.path).file_name().map_or_else(|| clip.path.clone(), |n| n.to_string_lossy().into_owned());
    let verb = l.pick(match clip.op {
        FileOp::Copy => ["复制", "複製", "copy"],
        FileOp::Move => ["移动", "移動", "move"],
    });
    let mut c = page(cx);
    let top = t.m.content_top;
    let there = tr!(l, "\"{into}\"中已有同名的\"{name}\"。", "「{into}」中已有同名的「{name}」。", "“{into}” already holds an entry named “{name}”.");
    let y = widgets::paragraph(&mut c, t, f, top, &there);
    let skip = tr!(l, "保留已有的同名项，不{verb}该项", "保留已有的同名項目，不{verb}這個項目", "Keeps the entry there, and doesn't {verb} this one");
    let choices = widgets::list(&mut c, t, f, y + t.m.dp(12.0), &[
        Row::new(l.pick(["替换", "取代", "Replace"])).subtitle(l.pick(["覆盖已有的同名项", "覆寫已有的同名項目", "Writes over the entry there"])),
        Row::new(l.pick(["跳过", "略過", "Skip"])).subtitle(&skip),
        Row::new(l.pick(["保留两者", "保留兩者", "Keep both"])).subtitle(l.pick(["自动为该项重新命名", "自動為這個項目重新命名", "Gives this one a new name"])),
    ]);
    let out = hits(&choices.rows, &[
        Action::Resolve(ClashChoice::Replace),
        Action::Resolve(ClashChoice::Skip),
        Action::Resolve(ClashChoice::KeepBoth),
    ]);
    Frame::new(t, c, out, l.pick(["名称已存在", "名稱已存在", "Name taken"]), top)
}

/// How work on files is told of: the Chinese verb its sentences build on,
/// in simplified and in traditional characters, then English's own words
/// for what is done and for doing it; the page's titles while it goes on
/// and once it ended short; and what stopping it may have left.
struct FileWords {
    verb: [&'static str; 2],
    done: &'static str,
    base: &'static str,
    titles: (Text, Text),
    stopped: Text,
}

fn file_words(task: &FileTask) -> FileWords {
    match task {
        FileTask::Copy { .. } => FileWords {
            verb: ["复制", "複製"],
            done: "copied",
            base: "copy",
            titles: (["正在复制", "正在複製", "Copying"], ["复制未完成", "複製未完成", "Copy incomplete"]),
            stopped: ["已停止，部分内容可能已复制到目标文件夹。", "已停止，部分內容可能已複製到目標資料夾。", "Stopped. Part of it may have been copied into the folder."],
        },
        FileTask::Move { .. } => FileWords {
            verb: ["移动", "移動"],
            done: "moved",
            base: "move",
            titles: (["正在移动", "正在移動", "Moving"], ["移动未完成", "移動未完成", "Move incomplete"]),
            stopped: ["已停止，部分内容可能已移动到目标文件夹。", "已停止，部分內容可能已移動到目標資料夾。", "Stopped. Part of it may have been moved into the folder."],
        },
        FileTask::Delete { .. } => FileWords {
            verb: ["删除", "刪除"],
            done: "deleted",
            base: "delete",
            titles: (["正在删除", "正在刪除", "Deleting"], ["删除未完成", "刪除未完成", "Delete incomplete"]),
            stopped: ["已停止，部分内容可能已删除。", "已停止，部分內容可能已刪除。", "Stopped. Part of it may have been deleted."],
        },
    }
}

/// Copying, moving or deleting: how far it has come and a way to stop it,
/// then, if it did not finish, why.
pub fn file_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.job.as_ref().expect("the job page shows work on files");
    let path = match &job.task {
        FileTask::Copy { path, .. } | FileTask::Move { path, .. } | FileTask::Delete { path } => path,
    };
    let words = file_words(&job.task);
    let ([hans, hant], done_en, base) = (words.verb, words.done, words.base);
    let name = Path::new(path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
    let share = |(done, total): (u64, u64)| if total == 0 { 0.0 } else { done as f32 / total as f32 };
    // The bar stays while the work measures, so that the button under it
    // does not move. Deleting, and copying entries that hold no bytes, is
    // counted in entries.
    let (detail, done) = match (job.progress, &job.task) {
        (None, _) => (l.pick(GETTING_READY).to_string(), 0.0),
        (Some((bytes, _)), FileTask::Copy { .. } | FileTask::Move { .. }) if bytes.1 > 0 => {
            let (did, total) = (fmt::file_size(bytes.0), fmt::file_size(bytes.1));
            (tr!(l, "已{hans} {did}，共 {total}", "已{hant} {did}，共 {total}", "{did} of {total} {done_en}"), share(bytes))
        }
        (Some((_, entries)), _) => {
            let (did, total) = entries;
            (tr!(l, "已{hans} {did} 项，共 {total} 项", "已{hant} {did} 項，共 {total} 項", "{did} of {total} items {done_en}"), share(entries))
        }
    };
    let mut c = page(cx);
    let top = t.m.content_top;
    let y = widgets::progress_card(&mut c, t, f, top, &name, &detail, Some(done));
    let mut out = Vec::new();
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { STOPPING } else { STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopJob]));
            }
        }
        Some(end) => {
            let why = match end {
                JobEnd::Stopped => l.pick(words.stopped).to_string(),
                JobEnd::Failed(why) => tr!(l, "{hans}失败：{why}", "{hant}失敗：{why}", "Couldn't {base} it: {why}"),
                JobEnd::Done => unreachable!("work done leaves its page"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &why);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    let title = l.pick(if job.outcome.is_none() { words.titles.0 } else { words.titles.1 });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// A shell on a terminal: the screen it writes, the keys a phone keyboard
/// lacks, and the keyboard, every key of which goes to the shell.
pub fn terminal(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    keyboard_hits(&mut c, t, f, s, s.lang.pick(["回车", "Enter", "Enter"]), false, &mut out);
    let l = term::layout(t, f, top);
    term::draw(&mut c, t, f, &l, cx.term);
    let held = cx.term.is_some_and(|term| term.ctrl);
    let keys = term::keys(&mut c, t, f, &l, &term::TERM_KEYS, held);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    Frame { scrollback: Some((l.card, l.pitch)), ..Frame::new(t, c, out, s.lang.pick(TERMINAL), top) }
}

/// A text file being edited: where it is, its lines wrapped on a card, the
/// keys a phone keyboard lacks for moving the cursor, and the keyboard.
/// Once it is changed, the round button saves it.
pub fn editor(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let ed = s.editor.as_ref().expect("the editor page shows a file");
    let mut c = page(cx);
    let mut out = Vec::new();
    keyboard_hits(&mut c, t, f, s, s.lang.pick(["换行", "換行", "Return"]), false, &mut out);
    editor::caption(&mut c, t, f, &ed.path);
    let l = editor::layout(t, f);
    let caret = editor::draw(&mut c, t, f, &l, ed);
    let keys = term::keys(&mut c, t, f, &l, &editor::EDITOR_KEYS, false);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    let button = ed.changed.then_some((widgets::Glyph::Done, Action::SaveText));
    Frame { scrollback: Some((l.card, l.pitch)), caret, button, ..Frame::new(t, c, out, s.lang.pick(["编辑", "編輯", "Edit"]), t.m.content_top) }
}

/// The languages the pages speak, each named in itself, to speak another.
pub fn language(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let rows: Vec<Row> = Lang::ALL.iter().map(|&k| Row::new(k.name()).trail(Trail::Check(k == l))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let out = hits(&list.rows, &Lang::ALL.map(Action::Language));
    Frame::new(t, c, out, l.pick(LANGUAGE), top)
}

/// The time zones, in the words their pages put on them.
const FOLLOW_SYSTEM: Text = ["跟随系统", "跟隨系統", "Use the system's time zone"];
const CHOOSE_ZONE: Text = ["选择时区", "選擇時區", "Choose a time zone"];
const ZONE_NOTE: Text = ["状态栏和各页面中的时间按所选时区显示。", "狀態列和各頁面中的時間按所選時區顯示。", "The status bar and the pages tell times in the zone chosen."];

/// The time zone the status bar's clock and the times on the pages keep.
/// On a phone whose installed system's zone is known, a switch follows it
/// and tells which it is; a phone with no system, or one that has set no
/// zone, shows none. Under it a zone is chosen from a part of the world,
/// whose row names the zone kept while that is one of its own.
pub fn time_zone(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = match &s.system_zone {
        Some(zone) => {
            let name = listed(&zone.name).map_or(zone.name.as_str(), |at| l.pick(TIME_ZONES[at].name));
            let system = format!("{} · {name}", fmt::utc_offset(s.now, &zone.rules));
            let follow = Row::new(l.pick(FOLLOW_SYSTEM)).subtitle(&system).trail(Trail::Switch(s.follow_system_zone));
            let follow = widgets::list(&mut c, t, f, top, &[follow]);
            out.extend(hits(&follow.rows, &[Action::Toggle(Switch::SystemTimeZone)]));
            widgets::section(&mut c, t, f, follow.bottom, l.pick(CHOOSE_ZONE))
        }
        None => top,
    };
    let kept = s.zone_listed();
    let rows = Region::ALL.map(|region| {
        let row = Row::new(l.pick(region.name()));
        match kept.filter(|&at| TIME_ZONES[at].region == Some(region)) {
            Some(at) => row.trail(Trail::Value(l.pick(TIME_ZONES[at].name))),
            None => row,
        }
    });
    let regions = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&regions.rows, &Region::ALL.map(|region| Action::Open(Screen::TimeZones(region)))));
    let utc = listed(UTC).expect("UTC is a zone listed");
    let offset = fmt::utc_offset(s.now, fmt::rules(utc));
    let universal = Row::new(l.pick(TIME_ZONES[utc].name)).subtitle(&offset).trail(Trail::Check(kept == Some(utc)));
    let universal = widgets::list(&mut c, t, f, regions.bottom + t.m.card_gap, &[universal]);
    out.extend(hits(&universal.rows, &[Action::TimeZone(utc)]));
    let bottom = widgets::footnote(&mut c, t, f, universal.bottom, l.pick(ZONE_NOTE));
    Frame::new(t, fit(c, t, bottom), out, l.pick(TIME_ZONE), top)
}

/// The time zones of one part of the world, from the one furthest behind
/// UTC now to the one furthest ahead, each with how far that is and the
/// cities it is known by.
pub fn time_zones(cx: &Ctx, region: Region) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut zones: Vec<usize> = (0..TIME_ZONES.len()).filter(|&at| TIME_ZONES[at].region == Some(region)).collect();
    zones.sort_by_key(|&at| fmt::utc_offset_seconds(s.now, fmt::rules(at)));
    let lines: Vec<String> = zones.iter().map(|&at| format!("{} · {}", fmt::utc_offset(s.now, fmt::rules(at)), l.pick(TIME_ZONES[at].cities))).collect();
    let kept = s.zone_listed();
    let rows: Vec<Row> = zones.iter().zip(&lines).map(|(&at, line)| Row::new(l.pick(TIME_ZONES[at].name)).subtitle(line).trail(Trail::Check(kept == Some(at)))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}

/// Saving the logs, as a bug report asks for them.
pub fn logs(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["日志内容", "記錄內容", "Logs"]));
    let parts = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["Recovery 日志", "Recovery 記錄", "Recovery log"]))
            .subtitle(l.pick(["界面、安装和挂载的记录", "介面、安裝和掛載的記錄", "What the pages, installs and mounts did"]))
            .trail(Trail::Check(s.logs[0])),
        Row::new(l.pick(["内核日志", "核心記錄", "Kernel log"]))
            .subtitle(l.pick(["本次开机以来的内核消息", "本次開機以來的核心訊息", "The kernel's messages since this start"]))
            .trail(Trail::Check(s.logs[1])),
    ]);
    out.extend(hits(&parts.rows, &[Action::LogPart(0), Action::LogPart(1)]));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["保存到", "儲存到", "Save to"]));
    let storage = l.pick(match cx.state.data.storage_open() {
        true => ["Skyward/logs 文件夹", "Skyward/logs 資料夾", "The Skyward/logs folder"],
        false => DECRYPT_NEEDED,
    });
    let places = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["临时文件夹", "暫存資料夾", "Temporary folder"]))
            .subtitle(l.pick(["/skyward/run/logs，可通过 adb pull 导出到电脑", "/skyward/run/logs，可透過 adb pull 匯出到電腦", "/skyward/run/logs, which adb pull copies to a computer"]))
            .trail(Trail::Check(s.logs_to == 0)),
        Row::new(place_name(l, Place::Storage)).subtitle(storage).trail(Trail::Check(s.logs_to == 1)),
    ]);
    out.extend(hits(&places.rows, &[Action::LogsTo(0), Action::LogsTo(1)]));
    let note = l.pick([
        "每次导出保存在以导出时间命名的文件夹中。重启后将清空临时文件夹。",
        "每次匯出儲存在以匯出時間命名的資料夾中。重新啟動後將清空暫存資料夾。",
        "Each export goes in a folder named by when it was made. A reboot empties the temporary folder.",
    ]);
    let y = widgets::footnote(&mut c, t, f, places.bottom, note);
    let export = widgets::button(&mut c, t, f, y + t.m.dp(24.0), l.pick(["导出", "匯出", "Export"]));
    out.push(Hit { rect: export, action: Action::ExportLogs });
    Frame::new(t, c, out, l.pick(LOGS), top)
}

/// Skyward's licence, in Chinese, ahead of the words the licence gives for
/// a program's notice; English reads those words alone.
const LICENSE_NOTICE: [&str; 2] = [
    "Skyward 是自由软件：你可以依据自由软件基金会发布的 GNU 通用公共许可证第 3 版，或（由你选择）任何更新的版本，重新发布和修改它。\n\n\
发布 Skyward 是希望它有用，但不附带任何担保，也不担保其适销性或适用于特定用途。详见 GNU 通用公共许可证。",
    "Skyward 是自由軟體：你可以依據自由軟體基金會發布的 GNU 通用公眾授權條款第 3 版，或（由你選擇）任何更新的版本，重新發布和修改它。\n\n\
發布 Skyward 是希望它有用，但不附帶任何擔保，也不擔保其適銷性或適用於特定用途。詳見 GNU 通用公眾授權條款。",
];

const LICENSE_ORIGINAL: &str = "Skyward\nCopyright (C) 2026 Kirk Lin\n\n\
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\n\
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.\n\n\
You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.";

const LICENSE_TERMS: Text = ["许可证", "授權條款", "License"];

/// Skyward's own licence.
pub fn license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let terms = l.pick([
        "GNU 通用公共许可证第 3 版或更新的版本（GPL-3.0-or-later）",
        "GNU 通用公眾授權條款第 3 版或更新的版本（GPL-3.0-or-later）",
        "GNU General Public License, version 3 or later (GPL-3.0-or-later)",
    ]);
    let y = widgets::info_card(&mut c, t, f, top, &[(l.pick(LICENSE_TERMS), terms.to_string())]);
    let bottom = match l {
        Lang::Hans | Lang::Hant => {
            let notice = if l == Lang::Hans { LICENSE_NOTICE[0] } else { LICENSE_NOTICE[1] };
            let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), notice);
            let y = widgets::section(&mut c, t, f, y, "原文");
            widgets::paragraph(&mut c, t, f, y - t.m.dp(16.0), LICENSE_ORIGINAL)
        }
        Lang::En => widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), LICENSE_ORIGINAL),
    };
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(LICENSE), top)
}

/// The monospaced typeface's licence, which names the copyright holders on
/// its first lines; the image carries the same file beside the font.
const MONO_LICENSE: &str = include_str!("../../../resources/fonts/KirkMono-OFL.txt");

/// The typefaces, as they name themselves, and the terms each is used under.
pub fn font_license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let (typeface, version, copyright) = (l.pick(["字体", "字型", "Typeface"]), l.pick(["版本", "版本", "Version"]), l.pick(["版权", "版權", "Copyright"]));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["界面文字", "介面文字", "Interface text"]));
    let y = widgets::info_card(&mut c, t, f, y, &[
        (typeface, f.info.family.clone()),
        (version, f.info.version.clone()),
        (copyright, f.info.copyright.clone().expect("MiSans names its copyright holder")),
    ]);
    let misans = l.pick([
        "界面文字使用小米 MiSans 字体，依据《MiSans 字体知识产权许可协议》免费使用。字体文件未经修改，仅随 Skyward 提供，不单独分发。\n\n协议全文可在 MiSans 官方网站查看。",
        "介面文字使用小米 MiSans 字型，依據《MiSans 字體知識產權許可協議》免費使用。字型檔案未經修改，僅隨 Skyward 提供，不單獨散布。\n\n協議全文可在 MiSans 官方網站查看。",
        "The interface text is set in Xiaomi's MiSans, used free of charge under the MiSans Font Intellectual Property License Agreement. The font files are unchanged, and come with Skyward alone, never on their own.\n\nThe agreement in full is on the MiSans website.",
    ]);
    let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), misans);
    let y = widgets::section(&mut c, t, f, y, l.pick(["终端文字", "終端機文字", "Terminal text"]));
    let holders: Vec<&str> = MONO_LICENSE.lines().take_while(|l| l.starts_with("Copyright")).collect();
    let y = widgets::info_card(&mut c, t, f, y, &[
        (typeface, f.mono_info.family.clone()),
        (version, f.mono_info.version.clone()),
        (copyright, holders.join("\n")),
        (l.pick(LICENSE_TERMS), l.pick(["SIL 开源字体许可证 1.1 版（OFL-1.1）", "SIL 開放字型授權條款 1.1 版（OFL-1.1）", "SIL Open Font License, version 1.1 (OFL-1.1)"]).to_string()),
    ]);
    let mono = l.pick([
        "终端文字使用 Kirk Mono 等宽字体。该许可证允许自由使用、修改和再发布字体，但不允许单独出售字体。\n\n许可证全文位于 /skyward/KirkMono-OFL.txt，也可在 openfontlicense.org 查看。",
        "終端機文字使用 Kirk Mono 等寬字型。這份授權條款允許自由使用、修改和再散布字型，但不允許單獨出售字型。\n\n授權條款全文位於 /skyward/KirkMono-OFL.txt，也可在 openfontlicense.org 查看。",
        "The terminal text is set in the monospaced Kirk Mono. Its license lets the font be used, changed and passed on freely, but not sold on its own.\n\nThe license in full is at /skyward/KirkMono-OFL.txt, and on openfontlicense.org.",
    ]);
    let bottom = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), mono);
    // 名字里的表情符号：两份 Noto 字体，和上面两种一样要在页面上说明。
    let y = widgets::section(&mut c, t, f, bottom, l.pick(["表情符号", "表情符號", "Emoji"]));
    let emoji = l.pick([
        "联系人名称里的表情符号由随包附带的两份 Noto 字体绘制：普通表情是矢量的 Noto Color Emoji，国旗是位图的 Noto Color Emoji Flags。两份都按 SIL 开源字体许可证 1.1 版（OFL-1.1）使用，字体文件未经修改；读不到随包这两份时，用已安装系统里的那一份。",
        "聯絡人名稱裡的表情符號由隨包附帶的兩份 Noto 字型繪製：一般表情是向量的 Noto Color Emoji，國旗是點陣的 Noto Color Emoji Flags。兩份都依 SIL 開源字型授權條款 1.1 版（OFL-1.1）使用，字型檔案未經修改；讀不到隨包這兩份時，用已安裝系統裡的那一份。",
        "Emoji in a name are drawn from the two Noto typefaces that come with Skyward: Noto Color Emoji, in outlines, and Noto Color Emoji Flags, in bitmaps for the flags. Both are used under the SIL Open Font License, version 1.1, unchanged; where the two are not beside it, the installed system's own copies are used.",
    ]);
    let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), emoji);
    let licence = l.pick([
        "许可证全文位于 /skyward/NotoColorEmoji-OFL.txt，也可在 openfontlicense.org 查看。",
        "授權條款全文位於 /skyward/NotoColorEmoji-OFL.txt，也可在 openfontlicense.org 查看。",
        "The license in full is at /skyward/NotoColorEmoji-OFL.txt, and on openfontlicense.org.",
    ]);
    let bottom = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), licence);
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(FONTS), top)
}

const BOOT_SLOT: Text = ["启动槽位", "啟動槽位", "Boot slot"];

/// Which slot the phone starts from next, as the boot control holds it,
/// and what it holds of each; picking the other and swiping switches.
/// While an update is in flight the slot stays as the update set it.
pub fn slot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(BOOT_SLOT);
    let Some(current) = i.slot else {
        widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["本机没有 A/B 槽位", "本機沒有 A/B 槽位", "This phone has no A/B slots"])).trail(Trail::None)]);
        return Frame::new(t, c, out, title, top);
    };
    let slots = match &s.boot_slots {
        Some(Ok(slots)) => *slots,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["无法读取启动槽位", "無法讀取啟動槽位", "Couldn't read the boot slots"])).trail(Trail::None)]);
            widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, c, out, title, top);
        }
        None => return Frame::new(t, c, out, title, top),
    };
    let chosen = s.next_slot.unwrap_or(slots.active);
    // 启动控制在 recovery 里没起来：只知道当前运行在哪个槽位，另一格的状态
    // 读不到，也别给切换（切了也没人记账）。2026-11-23 用户要「槽位能读出来」。
    if s.slot_from_running {
        let y = widgets::section_first(&mut c, t, f, top, l.pick(["下次启动", "下次啟動", "Next boot"]));
        let titles = [Slot::A, Slot::B].map(|slot| {
            let slot = slot.letter();
            tr!(l, "槽位 {slot}", "槽位 {slot}", "Slot {slot}")
        });
        let subs = [Slot::A, Slot::B].map(|slot| {
            l.pick(if slot == current {
                ["当前运行的系统", "目前執行的系統", "The system running now"]
            } else {
                ["另一个槽位，recovery 里读不到它的状态", "另一個槽位，recovery 裡讀不到它的狀態", "The other slot; recovery can't read its state"]
            })
        });
        let rows: Vec<Row> =
            [Slot::A, Slot::B].iter().enumerate().map(|(k, &slot)| Row::new(&titles[k]).subtitle(subs[k]).trail(Trail::Check(slot == chosen))).collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        let note = l.pick([
            "recovery 里没有启动控制服务，只能按当前运行的系统说下次启动；要在两个槽位之间切换，请在系统的「系统更新」里做。",
            "recovery 裡沒有啟動控制服務，只能按目前執行的系統說下次啟動；要在兩個槽位之間切換，請在系統的「系統更新」裡做。",
            "Recovery has no boot control service, so the slot running now is the one shown for next boot. Switching slots goes through the system's own updater.",
        ]);
        widgets::footnote(&mut c, t, f, list.bottom, note);
        return Frame::new(t, c, out, title, top);
    }
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["下次启动", "下次啟動", "Next boot"]));
    let titles = [Slot::A, Slot::B].map(|slot| {
        let slot = slot.letter();
        tr!(l, "槽位 {slot}", "槽位 {slot}", "Slot {slot}")
    });
    let subs = [Slot::A, Slot::B].map(|slot| {
        let k = slot.number() as usize;
        let which = l.pick(if slot == current { ["当前运行的系统", "目前執行的系統", "The system running now"] } else { ["另一个槽位", "另一個槽位", "The other slot"] });
        match (slots.bootable[k], slots.successful[k]) {
            (false, _) => tr!(l, "{which}，已标记为无法启动", "{which}，已標記為無法啟動", "{which}, marked unbootable"),
            (true, false) => tr!(l, "{which}，尚未完整启动过", "{which}，尚未完整啟動過", "{which}, not yet fully started"),
            (true, true) => which.to_string(),
        }
    });
    let rows: Vec<Row> =
        [Slot::A, Slot::B].iter().enumerate().map(|(k, &slot)| Row::new(&titles[k]).subtitle(&subs[k]).trail(Trail::Check(slot == chosen))).collect();
    let list = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&list.rows, &[Slot::A, Slot::B].map(Action::PickSlot)));
    if let Some(note) = fmt::snapshot_slot_note(l, &i.snapshot) {
        widgets::footnote(&mut c, t, f, list.bottom, note);
        return Frame::new(t, c, out, title, top);
    }
    let note = l.pick([
        "另一个槽位中没有完整的系统时，切换后将无法开机。",
        "另一個槽位中沒有完整的系統時，切換後將無法開機。",
        "If the other slot holds no whole system, the phone won't start after switching.",
    ]);
    let y = widgets::footnote(&mut c, t, f, list.bottom, note);
    let (label, danger) = confirm_label(Confirm::Slot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Slot) });
    Frame::new(t, c, out, title, top)
}

const MAIN_SYSTEM: Text = ["主系统", "主系統", "Main system"];
const SECOND_SYSTEM: Text = ["第二系统", "第二系統", "Second system"];

/// What the home page and the systems page say once the second system's
/// start stopped.
const SECOND_FAILED: Text = ["第二系统未能启动", "第二系統未能啟動", "The second system couldn't start"];

/// The two systems as big cards one over the other, laid out as the boot
/// slot page lays out its slots: the main system, in the phone's own
/// partitions, and the second, whose files lie in the data partition, or
/// the place to install one, the one that starts next checked. Picking the
/// other and swiping writes its boot images into the slot that starts
/// next, and restarts into it. Under them, the second system, and removing
/// it.
pub fn systems(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(DUAL_BOOT);
    // While the boot partitions are written, or the second system removed,
    // the page only says so: of a switch the system asked for, perhaps
    // before the systems are read.
    if let Some(work) = s.systems_work {
        let (heading, detail) = match work {
            SystemsWork::Switching(to) => {
                let name = match (&s.systems, to) {
                    (Some(Ok(systems)), Who::Installed) => Some(systems.installed.name.as_str()),
                    (Some(Ok(systems)), Who::Other) => systems.other.as_ref().map(|o| o.name.as_str()),
                    _ => None,
                };
                let heading = match name {
                    Some(name) => tr!(l, "正在切换到 {name}", "正在切換到 {name}", "Switching to {name}"),
                    None => l.pick(["正在切换系统", "正在切換系統", "Switching systems"]).to_string(),
                };
                (heading, l.pick(["正在写入启动镜像", "正在寫入啟動映像檔", "Writing its boot images"]))
            }
            SystemsWork::Removing => (
                l.pick(["正在删除第二系统", "正在刪除第二系統", "Removing the second system"]).to_string(),
                l.pick(["正在归还 data 分区中的空间", "正在歸還 data 分割區中的空間", "Giving its room back to the data partition"]),
            ),
        };
        let y = widgets::progress_card(&mut c, t, f, top, &heading, detail, None);
        let y = widgets::footnote(&mut c, t, f, y, l.pick(["完成前，请不要关机。", "完成前，請不要關機。", "Don't power off until it's done."]));
        return Frame::new(t, fit(c, t, y), out, title, top);
    }
    let systems = match &s.systems {
        Some(Ok(systems)) => systems,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["无法读取双系统", "無法讀取雙系統", "Couldn't read the systems"])).trail(Trail::None)]);
            let y = widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, fit(c, t, y), out, title, top);
        }
        None => return Frame::new(t, fit(c, t, top), out, title, top),
    };
    let starts = starting(systems);
    let chosen = s.system_pick.unwrap_or(starts);
    let second_role = l.pick(SECOND_SYSTEM);
    // Boot partitions that hold a second system's image nothing is kept of
    // start neither system the page knows.
    let main = SystemCard {
        role: l.pick(MAIN_SYSTEM),
        name: &systems.installed.name,
        detail: systems.installed.version.as_deref().unwrap_or_default(),
        mark: SystemMark::Check(chosen == Who::Installed && systems.held != Held::Unrecorded),
    };
    let second = match &systems.other {
        Some(other) => SystemCard { role: second_role, name: &other.name, detail: &other.version, mark: SystemMark::Check(chosen == Who::Other) },
        None => SystemCard {
            role: second_role,
            name: l.pick(["安装系统", "安裝系統", "Install a system"]),
            detail: l.pick(["选择第二系统的安装包", "選擇第二系統的安裝檔", "Choose the second system's package"]),
            mark: SystemMark::Opens,
        },
    };
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["下次启动", "下次啟動", "Next boot"]));
    let shown = widgets::system_cards(&mut c, t, f, y, &[main, second]);
    let second_opens = match systems.other {
        Some(_) => Action::PickSystem(Who::Other),
        None => Action::Open(Screen::Install),
    };
    out.extend(hits(&shown.rows, &[Action::PickSystem(Who::Installed), second_opens]));
    let mut y = shown.bottom + t.m.dp(12.0);
    if let Some(why) = &s.system_notice {
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(SECOND_FAILED), why.clone())]);
    }
    if chosen != starts {
        match (chosen, fmt::snapshot_system_note(l, &i.snapshot)) {
            (Who::Other, Some(note)) => y = widgets::footnote(&mut c, t, f, y, note),
            _ => {
                let (label, danger) = confirm_label(Confirm::SwitchSystem, s);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(12.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::SwitchSystem) });
                y = track.y + track.h;
            }
        }
    }
    let what = l.pick([
        "点按卡片选择系统。切换系统只改写下次启动槽位的 boot 和 init_boot 分区，主系统的其他分区保持不变。第二系统的文件保存在 data 分区中。",
        "點按卡片選擇系統。切換系統只改寫下次啟動槽位的 boot 和 init_boot 分割區，主系統的其他分割區保持不變。第二系統的檔案儲存在 data 分割區中。",
        "Tap a card to pick a system. Switching rewrites only the boot and init_boot partitions of the slot that starts next, and leaves the main system's other partitions as they are. The second system's files are kept in the data partition.",
    ]);
    y = widgets::footnote(&mut c, t, f, y, what);
    match systems.held {
        Held::Recorded => {}
        Held::Rewritten => {
            let note = l.pick([
                "Skyward 写入第二系统的启动镜像之后，启动分区被改写过。选择主系统并滑动确认，会用保存的主系统启动镜像覆盖现在的内容；在此之前，安装、清除和备份都不会进行。",
                "Skyward 寫入第二系統的啟動映像檔之後，啟動分割區被改寫過。選擇主系統並滑動確認，會用儲存的主系統啟動映像檔覆蓋現在的內容；在此之前，安裝、清除和備份都不會進行。",
                "Something else has written the boot partitions since Skyward put the second system's boot images there. Picking the main system and sliding to confirm writes the main system's saved boot images over what is there now; until then, nothing is installed, wiped or backed up.",
            ]);
            y = widgets::footnote(&mut c, t, f, y, note);
        }
        Held::Unrecorded => {
            let note = l.pick([
                "启动分区里是第二系统的启动镜像，Skyward 保存的记录已经不存在，无法写回主系统的启动镜像。请重新刷入主系统的 boot 和 init_boot 镜像。",
                "啟動分割區裡是第二系統的啟動映像檔，Skyward 儲存的記錄已經不存在，無法寫回主系統的啟動映像檔。請重新刷入主系統的 boot 和 init_boot 映像檔。",
                "The boot partitions hold a second system's boot images, and what Skyward kept of them is gone, so it can't put the main system's back. Flash the main system's boot and init_boot images again.",
            ]);
            y = widgets::footnote(&mut c, t, f, y, note);
        }
    }
    if systems.other.is_some() && systems.changed {
        let note = l.pick([
            "安装第二系统后，主系统的 vendor_boot 或 dtbo 分区已改变，第二系统可能无法正常启动。",
            "安裝第二系統後，主系統的 vendor_boot 或 dtbo 分割區已改變，第二系統可能無法正常啟動。",
            "The main system's vendor_boot or dtbo partition has changed since the second system was installed, so the second system may not start properly.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let Some(other) = &systems.other {
        y = widgets::section(&mut c, t, f, y, second_role);
        let size = fmt::bytes(other.size);
        let facts = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["占用空间", "佔用空間", "Space used"])).trail(Trail::Text(&size))]);
        let mounted = matches!(s.data, Data::Open | Data::Locked { .. });
        let remove = Row::new(l.pick(["删除第二系统", "刪除第二系統", "Remove the second system"])).tint(t.pal.danger);
        let (remove, act) = match mounted {
            true => (remove.trail(Trail::None), Action::AskDelete),
            false => (remove.subtitle(l.pick(DECRYPT_NEEDED)), Action::Open(Screen::Decrypt)),
        };
        let removing = widgets::list(&mut c, t, f, facts.bottom + t.m.card_gap, &[remove]);
        out.extend(hits(&removing.rows, &[act]));
        y = removing.bottom;
        if s.asking_delete && mounted {
            let ask = match starts {
                Who::Other => l.pick([
                    "将先切换回主系统，再删除第二系统及其全部数据，删除后无法恢复。",
                    "將先切換回主系統，再刪除第二系統及其全部資料，刪除後無法復原。",
                    "The phone goes back to the main system first, then the second system and all its data are removed for good.",
                ]),
                Who::Installed => l.pick([
                    "删除第二系统及其全部数据？删除后无法恢复。",
                    "刪除第二系統及其全部資料？刪除後無法復原。",
                    "Remove the second system and all its data? It can't be brought back.",
                ]),
            };
            let y2 = widgets::footnote(&mut c, t, f, y, ask);
            let (label, danger) = confirm_label(Confirm::RemoveSystem, s);
            let track = widgets::confirm(&mut c, t, f, y2 + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RemoveSystem) });
            y = track.y + track.h;
        }
    }
    Frame::new(t, fit(c, t, y), out, title, top)
}

/// The phone's own fastbootd serving the computer: whether it runs, how far
/// the computer has the port, and where to go from here, chosen with the
/// keys and lit where they are.
pub fn fastboot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let running = l.pick(if s.fastboot.running { ["正在运行", "正在執行", "Running"] } else { ["未运行", "未執行", "Not running"] });
    let link = l.pick(match s.fastboot.link {
        UsbLink::Connected => ["已连接电脑", "已連接電腦", "Computer connected"],
        UsbLink::Connecting => ["正在连接", "正在連線", "Connecting"],
        UsbLink::Unplugged => ["未连接电脑", "未連接電腦", "No computer"],
    });
    let slot = i.slot.map(|s| s.letter());
    let mut facts = vec![
        Row::new("fastbootd").trail(Trail::Text(running)),
        Row::new("USB").trail(Trail::Text(link)),
        Row::new(l.pick(["设备代号", "裝置代號", "Codename"])).trail(Trail::Text(&i.codename)),
    ];
    if let Some(letter) = slot {
        facts.push(Row::new(l.pick(CURRENT_SLOT)).trail(Trail::Text(letter)));
    }
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["状态", "狀態", "Status"]));
    let state = widgets::list(&mut c, t, f, y, &facts);
    let note = l.pick([
        "你可以在电脑上使用 fastboot 命令刷写分区，包括 super 中的逻辑分区。",
        "你可以在電腦上使用 fastboot 指令刷寫分割區，包括 super 中的邏輯分割區。",
        "fastboot commands on the computer can flash partitions, the logical ones in super among them.",
    ]);
    let y = widgets::footnote(&mut c, t, f, state.bottom, note);
    let y = widgets::section(&mut c, t, f, y, l.pick(["退出 fastboot", "結束 fastboot", "Leave fastboot"]));
    let rows: Vec<Row> = FASTBOOT_CHOICES.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
    let choices = widgets::list(&mut c, t, f, y, &rows);
    let actions: Vec<Action> = FASTBOOT_CHOICES.iter().map(|&(_, action)| action).collect();
    let out = hits(&choices.rows, &actions);
    widgets::footnote(&mut c, t, f, choices.bottom, l.pick(["按音量键选择，按电源键确认。", "按音量鍵選擇，按電源鍵確認。", "Choose with the volume keys, and press the power key to confirm."]));
    Frame { back: false, ..Frame::new(t, c, out, "Fastboot", top) }
}

/// Where to go once what the system asked for has stopped or cannot be
/// done, in the order the pages list them.
const AFTERWARDS: [(Text, Action); 3] = [
    (["重启到系统", "重新啟動到系統", "Reboot to system"], Action::RebootTo(Target::System)),
    (["进入恢复模式", "進入復原模式", "Enter recovery"], Action::Dismiss),
    (["关机", "關機", "Power off"], Action::RebootTo(Target::PowerOff)),
];

/// What work the system asked for says while it goes on of how the phone
/// leaves once it is done.
const POWERS_OFF_WHEN_DONE: Text = ["完成后将自动关机。", "完成後將自動關機。", "Powers off on its own once done."];
const REBOOTS_WHEN_DONE: Text = ["完成后将自动重启。", "完成後將自動重新啟動。", "Reboots on its own once done."];

/// The list of [`AFTERWARDS`] at `y`, and where it ends. Work the user
/// chose in the recovery goes back to its home page, rather than into it.
fn afterwards(c: &mut Canvas, t: &Theme, f: &Fonts, l: Lang, y: f32, asked: bool, out: &mut Vec<Hit>) -> f32 {
    let rows: Vec<Row> = AFTERWARDS
        .iter()
        .map(|&(title, action)| Row::new(l.pick(if action == Action::Dismiss && !asked { ["返回主页", "返回主頁", "Back to home"] } else { title })).trail(Trail::None))
        .collect();
    let choices = widgets::list(c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &AFTERWARDS.map(|(_, action)| action)));
    choices.bottom
}

/// Erasing: each part and how far it has come, and once erasing has
/// stopped, why. What the system asked for leaves on its own once it is
/// done, and offers where to go only if it stops; what the user chose
/// offers where to go once it ends either way.
pub fn erase(cx: &Ctx) -> Frame {
    use std::cmp::Ordering;
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = &s.erase;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let failed = matches!(e.outcome, Some(Err(_)));
    // Only a part that went wrong was under way; whatever else stopped
    // erasing stopped it before any part began.
    let begun = !failed || matches!(e.outcome, Some(Err(EraseStop::Failed(..))));
    let wiped = ["已清除", "已清除", "Wiped"];
    let heading = l.pick(match &e.outcome {
        None => WIPING,
        Some(Ok(())) => wiped,
        Some(Err(_)) => WIPE_INCOMPLETE,
    });
    let mut y = widgets::section_first(&mut c, t, f, top, heading);
    if !e.parts.is_empty() {
        let states: Vec<String> = (0..e.parts.len())
            .map(|k| match (k.cmp(&e.done), failed) {
                (Ordering::Less, _) => l.pick(wiped).into(),
                (Ordering::Equal, false) => match e.progress {
                    Some((done, total)) if total > 0 => format!("{} {}%", l.pick(WIPING), percent(share(done, total))),
                    _ => l.pick(WIPING).into(),
                },
                (Ordering::Equal, true) if begun => l.pick(["未完成", "未完成", "Unfinished"]).into(),
                (Ordering::Greater, false) => l.pick(WAITING_TURN).into(),
                (Ordering::Equal | Ordering::Greater, true) => l.pick(["未清除", "未清除", "Not wiped"]).into(),
            })
            .collect();
        let rows: Vec<Row> = e.parts.iter().zip(&states).map(|(&part, state)| Row::new(fmt::part(l, part)).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y, &rows).bottom;
    }
    let notes: Vec<String> = match (&e.outcome, e.asked, e.reset) {
        (Some(Err(stop)), _, _) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            std::iter::once(what).chain(detail).collect()
        }
        (_, true, true) => {
            let leaving = match s.asked.shutdown_after {
                true => ["本机的全部数据将被清除，完成后将自动关机。", "本機的全部資料將被清除，完成後將自動關機。", "All of this phone's data is being wiped, and it powers off on its own once done."],
                false => ["本机的全部数据将被清除，完成后将自动重启。", "本機的全部資料將被清除，完成後將自動重新啟動。", "All of this phone's data is being wiped, and it reboots on its own once done."],
            };
            vec![l.pick(leaving).into()]
        }
        (_, true, false) => vec![l.pick(if s.asked.shutdown_after { POWERS_OFF_WHEN_DONE } else { REBOOTS_WHEN_DONE }).into()],
        (None, false, _) => vec![l.pick(KEEP_POWER).into()],
        (Some(Ok(())), false, true) => {
            vec![l.pick(["下次开机时将重新创建 data 分区，启动时间较长。", "下次開機時將重新建立 data 分割區，啟動時間較長。", "The data partition is made anew at the next start, which then takes longer."]).into()]
        }
        (Some(Ok(())), false, false) => Vec::new(),
    };
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (e.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    if failed || (!e.asked && e.outcome.is_some()) {
        let below = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
        y = afterwards(&mut c, t, f, l, below, e.asked, &mut out);
    }
    let title = l.pick(match (e.reset, e.asked) {
        (true, _) => FACTORY_RESET,
        (false, true) => ["清除缓存", "清除快取", "Wipe cache"],
        (false, false) => ["清除数据", "清除資料", "Wipe data"],
    });
    Frame { back: false, ..Frame::new(t, fit(c, t, y), out, title, top) }
}

const FACTORY_RESET: Text = ["恢复出厂设置", "恢復原廠設定", "Factory reset"];

/// The system could not start Android and asks whether to erase all user
/// data: start it again, look around the recovery first, or erase.
pub fn cant_start(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let what = l.pick([
        "无法启动系统，本机数据可能已损坏。如果每次启动都停留在此页面，可恢复出厂设置，这将清除本机的全部数据。",
        "無法啟動系統，本機資料可能已損壞。如果每次啟動都停留在此頁面，可恢復原廠設定，這將清除本機的全部資料。",
        "The system couldn't start, and this phone's data may be damaged. If every start ends on this page, a factory reset may help, though it wipes all of this phone's data.",
    ]);
    let mut y = widgets::paragraph(&mut c, t, f, top, what);
    if let Some(reason) = &s.asked.reason {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    let rows = [
        Row::new(l.pick(["重试", "重試", "Try again"])).subtitle(l.pick(["重新启动系统", "重新啟動系統", "Starts the system again"])).trail(Trail::None),
        Row::new(l.pick(["进入恢复模式", "進入復原模式", "Enter recovery"])).subtitle(l.pick(["不清除任何内容", "不清除任何內容", "Wipes nothing"])).trail(Trail::None),
    ];
    let choices = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &[Action::RebootTo(Target::System), Action::Dismiss]));
    let y = widgets::footnote(&mut c, t, f, choices.bottom, l.pick(["恢复出厂设置后，数据无法恢复。", "恢復原廠設定後，資料無法復原。", "Data a factory reset wipes can't be brought back."]));
    let (label, danger) = confirm_label(Confirm::FactoryReset, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::FactoryReset) });
    Frame { back: false, ..Frame::new(t, c, out, l.pick(["无法启动系统", "無法啟動系統", "Can't start the system"]), top) }
}

/// Something the system asked of the recovery that it does not do: what
/// was asked, and where to go instead.
pub fn unhandled(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut items = vec![(l.pick(["请求", "要求", "Request"]), s.asked.unhandled.clone().unwrap_or_default())];
    if let Some(reason) = &s.asked.reason {
        items.push((l.pick(REASON), reason.clone()));
    }
    let y = widgets::info_card(&mut c, t, f, top, &items);
    let y = widgets::footnote(&mut c, t, f, y, l.pick(["Skyward 暂不支持该请求，本机未做任何改动。", "Skyward 尚不支援這項要求，本機未做任何變更。", "Skyward doesn't support this request, and changed nothing on this phone."]));
    let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    afterwards(&mut c, t, f, l, y, true, &mut out);
    Frame { back: false, ..Frame::new(t, c, out, l.pick(["无法处理该请求", "無法處理這項要求", "Can't handle the request"]), top) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::BackupRefusal;

    fn job(restore: bool, checks: bool) -> BackupJob {
        let plan = vec![("boot".into(), 200), ("system".into(), 800)];
        BackupJob { restore, place: Place::Storage, checks, plan, at: None, outcome: None, stopping: false }
    }

    fn at(item: usize, done: u64, of: u64, checking: bool) -> BackupAt {
        BackupAt { item, done, of, checking }
    }

    #[test]
    fn a_script_shows_its_last_line_until_skyward_goes_back() {
        let said = ["- Flashing boot".to_string(), "  ".to_string()];
        let l = Lang::Hans;
        assert_eq!(script_stage(l, None, &[]), ("正在运行安装脚本", None));
        assert_eq!(script_stage(Lang::En, None, &[]), ("Running the install script", None));
        assert_eq!(script_stage(l, Some(&Stage::Script { thousandths: 250 }), &said), ("- Flashing boot", Some(0.25)), "past a blank line");
        assert_eq!(script_stage(l, Some(&Stage::Reflashing { done: 1, total: 4 }), &said), ("正在重新刷入 Skyward", Some(0.25)));
        assert_eq!(
            update_stage(Lang::En, Some(&Stage::Reflashing { done: 3, total: 4 }), None),
            ("Flashing Skyward again".to_string(), Some(0.75))
        );
        assert_eq!(
            update_stage(Lang::Hant, Some(&Stage::Writing { done: 1, total: 4 }), None).0,
            "第 1 步，共 3 步：寫入新系統 25%"
        );
    }

    #[test]
    fn a_backup_checks_its_parts_last_and_a_restore_first() {
        // Halfway through writing system, 600 of the 1000 bytes: the first
        // of two passes, then the second.
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, false)), (0.3, 600));
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, true)), (0.8, 600));
        assert_eq!(job_progress(&job(false, false), &at(1, 400, 800, false)), (0.6, 600), "without a check, writing is all");
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, true)), (0.05, 100));
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, false)), (0.55, 100));
    }

    #[test]
    fn each_part_says_where_it_stands() {
        let states = |j: &BackupJob| (0..2).map(|k| part_state(Lang::Hans, j, k)).collect::<Vec<_>>();
        let mut b = job(false, true);
        assert_eq!(states(&b), ["等待中", "等待中"]);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!(states(&b), ["已备份", "正在备份 50%"]);
        b.at = Some(at(0, 50, 200, true));
        assert_eq!(states(&b), ["正在校验 25%", "已备份"]);
        b.outcome = Some(BackupEnd::Failed("the backup does not hold what was stored".into()));
        assert_eq!(states(&b), ["未完成", "未校验"]);
        b.outcome = Some(BackupEnd::Saved("20261021-141639".into()));
        assert_eq!(states(&b), ["已校验", "已校验"]);
        let mut r = job(true, true);
        r.at = Some(at(1, 0, 800, true));
        assert_eq!(states(&r), ["已校验", "正在校验 0%"]);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!(states(&r), ["正在恢复 50%", "已校验"]);
        r.outcome = Some(BackupEnd::Failed("write /dev/block/by-name/boot_b".into()));
        assert_eq!(states(&r), ["未完成", "未恢复"]);
        r.at = None;
        r.outcome = Some(BackupEnd::Refused(BackupRefusal::DataReset));
        assert_eq!(states(&r), ["未开始", "未开始"]);
    }

    #[test]
    fn each_part_says_where_it_stands_in_the_language() {
        let mut b = job(false, true);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!((0..2).map(|k| part_state(Lang::En, &b, k)).collect::<Vec<_>>(), ["Backed up", "Backing up 50%"]);
        let mut r = job(true, true);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!((0..2).map(|k| part_state(Lang::Hant, &r, k)).collect::<Vec<_>>(), ["正在還原 50%", "已驗證"]);
    }

    #[test]
    fn a_phrase_that_opens_a_line_starts_with_a_capital_in_english() {
        assert_eq!(capitalized(Lang::En, "empties the data partition"), "Empties the data partition");
        assert_eq!(capitalized(Lang::Hans, "清空 data 分区"), "清空 data 分区");
    }
}

#[cfg(test)]
mod preview {
    use super::*;

    #[test]
    fn the_field_shows_the_pinyin_that_is_being_typed() {
        let mut s = State::default();
        s.pinyin.on = true;
        // 26 键：按下去就是字母，框里显示的就是攒着的那一串。
        s.pinyin.composing = "woxihuanni".to_string();
        assert_eq!(with_preview(&s, ""), "woxihuanni", "26 键的字母原样显示");
        assert_eq!(with_preview(&s, "备份"), "备份woxihuanni", "已落下的字后面接正在打的拼音");
        s.pinyin.on = false;
        assert_eq!(with_preview(&s, "备份"), "备份", "英文模式框里只有字");
        s.pinyin.on = true;
        s.pinyin.composing.clear();
        assert_eq!(with_preview(&s, "备份"), "备份", "没在打拼音就什么都不加");
    }
}
