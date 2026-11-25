use anyhow::{anyhow, Context, Result};
use evdev::raw_stream::RawDevice;
use evdev::{AbsoluteAxisCode, EventSummary, InputEvent, KeyCode, PropType, SynchronizationCode};
use skyward_config::Write;
use skyward_sys::sleep::Wakeful;
use skyward_ui::app::{Key, Touch};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs;
use std::io::{self, ErrorKind};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub enum Event {
    /// A touch and when the panel reported it, on [`monotonic_ms`]'s clock.
    Touch(Touch, u64),
    /// A button going down, or coming back up.
    Key(Key, bool),
}

/// Milliseconds on the monotonic clock, which the touch panel stamps its
/// events with.
pub fn monotonic_ms() -> u64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    assert_eq!(rc, 0, "read the monotonic clock");
    ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000
}

fn set_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// 触摸开关只在三处写：面板接管时、主页上来时、每次亮屏。以前是「前十几秒
/// 每两秒重写一遍」——写一次驱动就把 THP 重配一遍（日志里每写一次就有一行
/// `store_enable_touch_raw`），正按着的手指会断，而它并不能把延迟抹掉
/// （2026-11-23 用户第二次反馈之后去掉）。
const POKE_GAP: Duration = Duration::from_millis(300);

const KEYS: [KeyCode; 4] = [KeyCode::KEY_POWER, KeyCode::KEY_VOLUMEUP, KeyCode::KEY_VOLUMEDOWN, KeyCode::KEY_BACK];

fn key_of(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::KEY_POWER => Some(Key::Power),
        KeyCode::KEY_VOLUMEUP => Some(Key::VolumeUp),
        KeyCode::KEY_VOLUMEDOWN => Some(Key::VolumeDown),
        KeyCode::KEY_BACK => Some(Key::Back),
        _ => None,
    }
}

/// A device read failed because it was unplugged, or because the driver
/// under it went away: the panel is let go of and looked for again either
/// way. `ENODEV` is the plain case; a controller that was reset under an open
/// node answers `EIO` or `ENXIO` instead, and one the kernel shut down
/// answers `ESHUTDOWN` -- reading only `ENODEV` left the pages deaf until
/// the recovery was restarted (2026-11-25, after `adb sideload`).
fn gone(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(libc::ENODEV | libc::ENXIO | libc::EIO | libc::ESHUTDOWN | libc::EBADF))
}

/// No report from the panel for this long while the pages think a finger is
/// down: the panel is read back, and a finger it no longer sees is let go of.
/// A panel at rest sends nothing, so this only runs while something is held.
const STALE_REPORT: Duration = Duration::from_millis(2500);

/// After a panel's read failed, wait this long before opening it again: a
/// node that answers an error every time must not be reopened every turn.
const PANEL_RETRY: Duration = Duration::from_secs(2);

/// Slot 0's value of a multitouch axis. The kernel keeps multitouch values
/// per slot, apart from the axis state the ABS queries return, and hands
/// them out through EVIOCGMTSLOTS: the axis code, then one value per slot.
fn slot0_value(dev: &RawDevice, axis: AbsoluteAxisCode, slots: usize) -> io::Result<i32> {
    let mut buf = vec![0i32; 1 + slots];
    buf[0] = i32::from(axis.0);
    // _IOC(_IOC_READ, 'E', 0x0a, size)
    let request = (2u32 << 30) | (((buf.len() * 4) as u32) << 16) | (u32::from(b'E') << 8) | 0x0a;
    if unsafe { libc::ioctl(dev.as_raw_fd(), request as _, buf.as_mut_ptr()) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(buf[1])
}

/// The direct-touch panel. Only the first finger is followed (multitouch
/// protocol B, slot 0).
struct Panel {
    dev: RawDevice,
    path: PathBuf,
    x_range: (i32, i32),
    y_range: (i32, i32),
    slots: usize,
    slot: i32,
    down: bool,
    was_down: bool,
    raw: (i32, i32),
    moved: bool,
    /// The kernel dropped events that were not read in time; what arrives
    /// up to the next report is incomplete.
    dropped: bool,
    /// The first finger that lands is written to kmsg once, with the time the
    /// panel stamped on it: that is the moment a touch could first be read
    /// after the display came up, which is the window a user calls "the screen
    /// is lit but nothing answers".
    first_told: bool,
    /// 这一根手指的按下发出去没有。内核丢事件（SYN_DROPPED）或者手指正压在
    /// 面板上时接管，按下那一条就可能没送到，只剩一串移动 —— 页面从此不认这
    /// 一根手指，看着就是「点了没反应」。没送过就先补一个按下（2026-11-23）。
    sent_down: bool,
    /// 面板最后一次报告的时刻，`monotonic_ms` 的钟。按住的时候它不动，
    /// 就是这一根手指的下文丢了（2026-11-25）。
    last_report: u64,
    /// 上一次因内核丢事件而重读状态的时刻，报错也只说一次。
    dropped_at: Option<u64>,
}

impl Panel {
    fn new(path: PathBuf, dev: RawDevice) -> Result<Self> {
        let mut x_range = None;
        let mut y_range = None;
        let mut slots = 1;
        for (axis, info) in dev.get_absinfo().context("read touch axis ranges")? {
            match axis {
                AbsoluteAxisCode::ABS_MT_POSITION_X => x_range = Some((info.minimum(), info.maximum())),
                AbsoluteAxisCode::ABS_MT_POSITION_Y => y_range = Some((info.minimum(), info.maximum())),
                AbsoluteAxisCode::ABS_MT_SLOT => slots = info.maximum() as usize + 1,
                _ => {}
            }
        }
        let x_range = x_range.ok_or_else(|| anyhow!("touch panel reports no X range"))?;
        let y_range = y_range.ok_or_else(|| anyhow!("touch panel reports no Y range"))?;
        // Stamp events with the clock the UI keeps time by: how fast a
        // finger moved when it lifted is measured from these stamps.
        // EVIOCSCLOCKID: _IOW('E', 0xa0, int)
        let request = (1u32 << 30) | (4u32 << 16) | (u32::from(b'E') << 8) | 0xa0;
        let clock: libc::c_int = libc::CLOCK_MONOTONIC;
        if unsafe { libc::ioctl(dev.as_raw_fd(), request as _, &clock) } < 0 {
            return Err(io::Error::last_os_error()).context("stamp touch events with the monotonic clock");
        }
        set_nonblocking(dev.as_raw_fd()).context("make touch non-blocking")?;
        let mut panel = Panel {
            dev,
            path,
            x_range,
            y_range,
            slots,
            slot: 0,
            down: false,
            was_down: false,
            raw: (0, 0),
            moved: false,
            dropped: false,
            first_told: false,
            sent_down: false,
            last_report: monotonic_ms(),
            dropped_at: None,
        };
        // The kernel drops a position equal to the one a slot already holds,
        // so a touch landing where the last one lifted carries no position;
        // start from what the panel holds now.
        panel.resync().context("read the touch state")?;
        let _ = skyward_sys::kmsg(&format!("touch {:?}: x {:?} y {:?}, {slots} slots, slot 0 at {:?}", panel.dev.name().unwrap_or("?"), x_range, y_range, panel.raw));
        Ok(panel)
    }

    /// Read the current slot and slot 0's finger back from the panel.
    fn resync(&mut self) -> io::Result<()> {
        self.slot = self.dev.get_abs_state()?[usize::from(AbsoluteAxisCode::ABS_MT_SLOT.0)].value;
        self.down = slot0_value(&self.dev, AbsoluteAxisCode::ABS_MT_TRACKING_ID, self.slots)? >= 0;
        self.raw = (
            slot0_value(&self.dev, AbsoluteAxisCode::ABS_MT_POSITION_X, self.slots)?,
            slot0_value(&self.dev, AbsoluteAxisCode::ABS_MT_POSITION_Y, self.slots)?,
        );
        self.moved = true;
        // 面板说的这一根发没发出去过，只有 read() 知道；读回来的状态只用来
        // 决定下一个 SYN_REPORT 该不该补一个按下。
        self.sent_down = false;
        self.was_down = self.down;
        Ok(())
    }

    fn read(&mut self, w: f32, h: f32, out: &mut Vec<Event>) -> io::Result<()> {
        let events: Vec<InputEvent> = match self.dev.fetch_events() {
            Ok(events) => events.collect(),
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        };
        for e in events {
            match e.destructure() {
                // Skip to the next report and read the state back there.
                EventSummary::Synchronization(_, SynchronizationCode::SYN_DROPPED, _) => {
                    self.dropped = true;
                    // 丢事件在这里说一声：真机上「屏幕亮着、点哪都不动」的
                    // 那几次，日志里连一句丢事件都没有，事后分不清是面板没
                    // 报还是自己把它丢了（2026-11-25）。两秒最多一句。
                    let now = monotonic_ms();
                    if self.dropped_at.is_none_or(|at| now.saturating_sub(at) >= 2000) {
                        self.dropped_at = Some(now);
                        let _ = skyward_sys::kmsg(&format!(
                            "the panel's events were dropped by the kernel at {now} ms; its state is read back"
                        ));
                    }
                }
                EventSummary::Synchronization(_, SynchronizationCode::SYN_REPORT, _) => {
                    self.last_report = monotonic_ms();
                    if std::mem::take(&mut self.dropped) {
                        // 状态读不回来不结束这一条流：把手指状态清干净，
                        // 下一次按下必定重新发一条按下。以前这里把错误往上传，
                        // 主循环每轮都在读输入处失败，页面照画、触摸全哑，
                        // 只能重启恢复（2026-11-25）。
                        if let Err(e) = self.resync() {
                            let _ = skyward_sys::kmsg(&format!(
                                "the panel's state is not read back ({e}); a finger held is let go of"
                            ));
                            self.down = false;
                            self.was_down = false;
                            self.sent_down = false;
                            self.moved = false;
                        }
                    }
                    let time = e.as_ref().time;
                    let at = time.tv_sec as u64 * 1000 + time.tv_usec as u64 / 1000;
                    let span = |r: (i32, i32)| (r.1 - r.0).max(1) as f32;
                    let x = (self.raw.0 - self.x_range.0) as f32 / span(self.x_range) * w;
                    let y = (self.raw.1 - self.y_range.0) as f32 / span(self.y_range) * h;
                    // 按下只发一次：面板本来就按着（接管时手指在上面）或者内核
                    // 丢了按下那一条，都不该只剩一串移动 —— 页面收不到按下，
                    // 这一根手指就整根作废。落地与读到的时刻也记下来：差得多是
                    // 主循环读得晚，只差几毫秒是面板自己没报（2026-11-23）。
                    let lands = matches!((self.was_down, self.down), (false, true)) || !self.sent_down;
                    if self.down && lands {
                        if !self.first_told {
                            self.first_told = true;
                            let now = monotonic_ms();
                            let _ = skyward_sys::kmsg(&format!(
                                "the first touch came in at {at} ms and was read {} ms later, {:?}",
                                now.saturating_sub(at),
                                self.dev.name().unwrap_or("?")
                            ));
                        }
                        self.sent_down = true;
                        out.push(Event::Touch(Touch::Down(x, y), at));
                    } else {
                        match (self.was_down, self.down) {
                            (true, true) if self.moved => out.push(Event::Touch(Touch::Move(x, y), at)),
                            (true, false) => {
                                self.sent_down = false;
                                out.push(Event::Touch(Touch::Up, at))
                            }
                            _ => {}
                        }
                    }
                    self.was_down = self.down;
                    self.moved = false;
                }
                _ if self.dropped => {}
                EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_SLOT, v) => self.slot = v,
                EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_TRACKING_ID, v) if self.slot == 0 => self.down = v >= 0,
                EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_X, v) if self.slot == 0 => {
                    self.raw.0 = v;
                    self.moved = true;
                }
                EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_Y, v) if self.slot == 0 => {
                    self.raw.1 = v;
                    self.moved = true;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 按住不放的时候，面板的下文要是丢了（抬手那一条没送来，或驱动在
    /// 底下被复位），页面会一直以为手指还在：之后再按只有移动、没有按下，
    /// 看着就是整块屏幕点不动，而时钟照走。隔一段时间没有报告就把它读回来，
    /// 面板也不再报的手指，替它抬一次手（2026-11-25）。
    fn catch_up(&mut self, out: &mut Vec<Event>) {
        let now = monotonic_ms();
        // 没有手指按着的时候面板本来就安静，不打扰它。
        if !self.down && !self.sent_down {
            self.last_report = now;
            return;
        }
        if now.saturating_sub(self.last_report) < STALE_REPORT.as_millis() as u64 {
            return;
        }
        self.last_report = now;
        self.moved = false;
        let resynced = self.resync();
        if resynced.is_ok() && self.down {
            // 面板说还按着：让页面重新认这一根（下一个报告会补一条按下）。
            self.sent_down = false;
            return;
        }
        if resynced.is_err() {
            let _ = skyward_sys::kmsg("the panel's state is still not read back; its finger is let go of");
        }
        if self.sent_down {
            self.sent_down = false;
            out.push(Event::Touch(Touch::Up, now));
            let _ = skyward_sys::kmsg(&format!(
                "a finger the panel no longer reports is let go of at {now} ms: the page is told, so it does not stay pressed"
            ));
        }
        self.down = false;
        self.was_down = false;
    }
}

/// The touchscreen and the hardware buttons. /dev/input is watched, so a
/// device that shows up later, such as a touch panel whose driver module
/// loads after the UI starts, is picked up when it appears.
pub struct Inputs {
    watch: OwnedFd,
    /// The watch and every device taken in: a touch or a key press keeps
    /// the phone awake until it is read.
    wakeful: Wakeful,
    panel: Option<Panel>,
    /// A multitouch device not marked direct, kept in case no direct one comes.
    spare_panel: Option<(PathBuf, RawDevice)>,
    buttons: Vec<(PathBuf, RawDevice)>,
    seen: HashSet<PathBuf>,
    /// What the touchscreen is told each time its panel is taken in.
    touch_writes: Vec<Write>,
    /// 上一次写离现在多久：接管、主页上来、亮屏这三处都会写，挨太近就只写一次。
    writes_at: Option<Instant>,
    /// 面板读错之后先等一会儿再把它找回来：节点还在、读还是错的时候，
    /// 别每一轮都重开一遍（2026-11-25）。
    panel_wait_until: Option<Instant>,
    w: f32,
    h: f32,
}

impl Inputs {
    pub fn open(w: u32, h: u32, touch_writes: Vec<Write>) -> Result<Self> {
        let fd = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
        if fd < 0 {
            return Err(io::Error::last_os_error()).context("create inotify instance");
        }
        let watch = unsafe { OwnedFd::from_raw_fd(fd) };
        let dir = CString::new("/dev/input")?;
        // IN_ATTRIB too: ueventd sets a node's owner and mode right after
        // creating it, and the node may only open once that is done.
        if unsafe { libc::inotify_add_watch(fd, dir.as_ptr(), libc::IN_CREATE | libc::IN_ATTRIB) } < 0 {
            return Err(io::Error::last_os_error()).context("watch /dev/input");
        }
        let wakeful = Wakeful::new()?;
        wakeful.add(fd)?;
        let mut inputs = Inputs {
            watch,
            wakeful,
            panel: None,
            spare_panel: None,
            buttons: Vec::new(),
            seen: HashSet::new(),
            touch_writes,
            writes_at: None,
            panel_wait_until: None,
            w: w as f32,
            h: h as f32,
        };
        inputs.scan()?;
        if inputs.panel.is_none() {
            let _ = skyward_sys::kmsg("no touch panel yet; waiting for one to appear under /dev/input");
        }
        Ok(inputs)
    }

    /// Take in the devices under /dev/input not seen before: every direct
    /// multitouch device is a candidate for the panel, the best of them is
    /// taken in, and every device with a power, volume or back key is a
    /// button. A panel taken in is told what the device description writes
    /// to the touchscreen.
    fn scan(&mut self) -> Result<()> {
        // 面板刚读过一次是错的：退一会儿再枚举。节点还在、读还是错的时候，
        // 每轮都重开一遍只是空转（2026-11-25）。
        if self.panel.is_none() && self.panel_wait_until.is_some_and(|at| at.elapsed() < PANEL_RETRY) {
            return Ok(());
        }
        let mut candidates: Vec<Panel> = Vec::new();
        for (path, dev) in evdev::raw_stream::enumerate() {
            if !self.seen.insert(path.clone()) {
                continue;
            }
            let direct = dev.properties().contains(PropType::DIRECT);
            let mt = dev
                .supported_absolute_axes()
                .is_some_and(|a| a.contains(AbsoluteAxisCode::ABS_MT_POSITION_X) && a.contains(AbsoluteAxisCode::ABS_MT_POSITION_Y));
            let keys = dev.supported_keys().is_some_and(|k| KEYS.iter().any(|c| k.contains(*c)));
            let _ = skyward_sys::kmsg(&format!("input {}: {:?} direct={direct} multitouch={mt} keys={keys}", path.display(), dev.name().unwrap_or("?")));
            // 触摸设备未必被标成 direct：有些机型的控制器不报这一位，
            // 只按 direct+mt 认会把面板漏掉。先把 mt 的都记下，direct 的优先；
            // 一直没有 direct 的，就把最后那个 mt 当面板用（下面 take_panel）。
            // 这台机器有两个面板（主屏 1200×2608、副屏 976×596），副屏的节点
            // 先出现：direct 的候选要打分，见一个就用会抓到副屏，主屏点不动
            // （2026-11-13）。
            if mt {
                if direct {
                    let name = dev.name().unwrap_or("?").to_owned();
                    match Panel::new(path, dev) {
                        Ok(panel) => candidates.push(panel),
                        Err(e) => {
                            let _ = skyward_sys::kmsg(&format!("touch {name}: not taken in: {e:#}"));
                        }
                    }
                } else {
                    let _ = skyward_sys::kmsg(&format!(
                        "{} reports multitouch but is not marked direct; kept in case no direct panel comes",
                        path.display()
                    ));
                    self.spare_panel = Some((path, dev));
                }
            } else if keys {
                set_nonblocking(dev.as_raw_fd()).context("make buttons non-blocking")?;
                self.wakeful.add(dev.as_raw_fd())?;
                self.buttons.push((path, dev));
            }
        }
        if !candidates.is_empty() {
            self.choose(candidates)?;
        }
        let _ = skyward_sys::kmsg(&format!(
            "{} input devices under /dev/input, a touch panel {} among them",
            self.seen.len(),
            if self.panel.is_some() { "is" } else { "is not" }
        ));
        Ok(())
    }

    /// Take the best of the panels that answered: the one the display's
    /// size fits exactly (the axis tops are the resolution less one) beats
    /// any other, and the rest are compared by area, as a coarse fallback.
    /// A panel already taken in gives way only to a better one, so one that
    /// registers late is taken in when it is the right one.
    fn choose(&mut self, candidates: Vec<Panel>) -> Result<()> {
        let score = |p: &Panel| -> i64 {
            let exact = ((p.x_range.1 as i64 + 1 == self.w as i64) && (p.y_range.1 as i64 + 1 == self.h as i64)) as i64;
            (exact << 60) + (p.x_range.1 - p.x_range.0) as i64 * (p.y_range.1 - p.y_range.0) as i64
        };
        for candidate in candidates {
            let name = candidate.dev.name().unwrap_or("?").to_owned();
            let take = match &self.panel {
                None => true,
                Some(p) => score(&candidate) > score(p),
            };
            if !take {
                let _ = skyward_sys::kmsg(&format!("touchscreen: {name} loses to the panel already taken in"));
                continue;
            }
            match self.panel.replace(candidate) {
                None => {
                    let _ = skyward_sys::kmsg(&format!("touchscreen: {name} taken in"));
                }
                Some(old) => {
                    let _ = skyward_sys::kmsg(&format!("touchscreen: {name} replaces {:?}", old.dev.name().unwrap_or("?")));
                }
            }
            if let Some(panel) = &self.panel {
                self.wakeful.add(panel.dev.as_raw_fd())?;
            }
            self.panel_wait_until = None;
            self.tell(&self.touch_writes)?;
            self.writes_at = Some(Instant::now());
        }
        Ok(())
    }

    /// Take the device at `path` as the panel, telling the touchscreen what the
    /// device description writes to it.
    fn take_panel(&mut self, path: std::path::PathBuf, dev: RawDevice) -> Result<()> {
        self.wakeful.add(dev.as_raw_fd())?;
        self.panel = Some(Panel::new(path, dev)?);
        self.panel_wait_until = None;
        self.tell(&self.touch_writes)?;
        self.writes_at = Some(Instant::now());
        Ok(())
    }

    /// 主页上来那一下再写一次：这是用户手指落下来的时刻（2026-11-23）。
    pub fn poke(&mut self) -> Result<()> {
        if self.touch_writes.is_empty() || self.panel.is_none() {
            return Ok(());
        }
        if self.writes_at.is_some_and(|at| at.elapsed() < POKE_GAP) {
            return Ok(());
        }
        self.writes_at = Some(Instant::now());
        let value = self.touch_writes.first().map(|w| w.value.clone()).unwrap_or_default();
        skyward_sys::kmsg(&format!("the page came up at {} ms; the touch switch is written, value {value}", monotonic_ms()))?;
        self.tell(&self.touch_writes)
    }

    /// 亮屏的那一下也写一次：息屏久了 THP 会自己沉下去，第一下滑动就白做。
    /// 写的是配置里那个值，与手机自己的系统给它的一样（2026-11-23）。
    pub fn on_wake(&mut self) -> Result<()> {
        if self.touch_writes.is_empty() || self.panel.is_none() {
            return Ok(());
        }
        if self.writes_at.is_some_and(|at| at.elapsed() < POKE_GAP) {
            return Ok(());
        }
        self.writes_at = Some(Instant::now());
        let value = self.touch_writes.first().map(|w| w.value.clone()).unwrap_or_default();
        skyward_sys::kmsg(&format!("the panel lit at {} ms; the touch switch is written, value {value}", monotonic_ms()))?;
        self.tell(&self.touch_writes)
    }

    /// Tell the touchscreen what the device description writes to it. A
    /// write that does not go is said so but does not drop the panel: the
    /// driver may keep the switch on another phone.
    fn tell(&self, writes: &[Write]) -> Result<()> {
        for w in writes {
            match fs::write(&w.path, &w.value) {
                Ok(()) => {
                    skyward_sys::kmsg(&format!("wrote {} to {} for the touchscreen", w.value, w.path))?;
                }
                Err(e) => {
                    skyward_sys::kmsg(&format!("{} is not written for the touchscreen: {e}", w.path))?;
                }
            }
        }
        Ok(())
    }

    /// Take the spare multitouch device as the panel when no direct one came.
    /// Called once the wait for a panel has run out.
    pub fn take_spare_panel(&mut self) -> Result<bool> {
        if self.panel.is_some() {
            return Ok(false);
        }
        let Some((path, dev)) = self.spare_panel.take() else { return Ok(false) };
        skyward_sys::kmsg(&format!("{} is taken as the panel though it is not marked direct", path.display()))?;
        self.take_panel(path, dev)?;
        Ok(true)
    }

    /// Empty the watch's queue; true when /dev/input changed.
    fn changed(&self) -> Result<bool> {
        let mut buf = [0u8; 4096];
        let mut any = false;
        loop {
            let n = unsafe { libc::read(self.watch.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len()) };
            if n >= 0 {
                any = true;
                continue;
            }
            let err = io::Error::last_os_error();
            match err.kind() {
                ErrorKind::WouldBlock => return Ok(any),
                ErrorKind::Interrupted => continue,
                _ => return Err(err).context("read /dev/input changes"),
            }
        }
    }

    /// Whether a touch panel has come up.
    pub fn has_panel(&self) -> bool {
        self.panel.is_some()
    }

    /// Readable while a device has something queued, or /dev/input changed.
    pub fn fd(&self) -> RawFd {
        self.wakeful.fd()
    }

    /// Everything the devices have queued, as touches and key presses.
    ///
    /// 这里不把错误交给主循环：一轮报错会让「读输入」整个跳过，页面照画、
    /// 时钟照走、一个触摸也处理不到 —— 真机上 `adb sideload` 之后那次
    /// 「点哪都没反应、屏幕还在走」就是这样（2026-11-25）。设备读不动就把它
    /// 放掉，下一轮 /dev/input 的变动或主循环仍会重新找它。
    pub fn read(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        match self.changed() {
            Ok(true) => {
                if let Err(e) = self.scan() {
                    let _ = skyward_sys::kmsg(&format!("/dev/input is not read: {e:#}"));
                }
            }
            Ok(false) => {}
            Err(e) => {
                let _ = skyward_sys::kmsg(&format!("/dev/input changes are not read: {e:#}"));
            }
        }
        // 面板被放掉之后节点未必再变动一次：退避过了就自己再去找它，
        // 否则一次读错等于这一屏从此没有触摸（2026-11-25）。
        if self.panel.is_none() && self.panel_wait_until.is_some_and(|at| at.elapsed() >= PANEL_RETRY) {
            self.panel_wait_until = Some(Instant::now());
            if let Err(e) = self.scan() {
                let _ = skyward_sys::kmsg(&format!("/dev/input is not read: {e:#}"));
            }
        }
        let mut drop_panel = None;
        if let Some(panel) = &mut self.panel {
            match panel.read(self.w, self.h, &mut out) {
                Ok(()) => panel.catch_up(&mut out),
                Err(e) => {
                    let said = if gone(&e) {
                        format!("input {}: removed", panel.path.display())
                    } else {
                        format!("input {}: the panel is not read: {e}", panel.path.display())
                    };
                    let _ = skyward_sys::kmsg(&said);
                    // 面板被放掉之前，替按着的那根手指抬一次手：页面才不会
                    // 停在「按住」上，等它回来还能接着点。
                    if panel.sent_down {
                        out.push(Event::Touch(Touch::Up, monotonic_ms()));
                    }
                    drop_panel = Some(panel.path.clone());
                }
            }
        }
        if let Some(path) = drop_panel {
            let _ = skyward_sys::kmsg(&format!(
                "the panel {} is let go of; it is looked for again shortly",
                path.display()
            ));
            self.seen.remove(&path);
            self.panel = None;
            self.panel_wait_until = Some(Instant::now());
        }
        let mut removed = Vec::new();
        for (i, (path, b)) in self.buttons.iter_mut().enumerate() {
            match b.fetch_events() {
                Ok(events) => {
                    // The kernel's own repeats of a key held are left out;
                    // the UI times its own.
                    for e in events {
                        if let EventSummary::Key(_, code, value @ (0 | 1)) = e.destructure()
                            && let Some(k) = key_of(code)
                        {
                            out.push(Event::Key(k, value == 1));
                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                // 读不动的按键设备一律放掉：一个坏了不该让整块屏幕哑掉。
                Err(e) => {
                    let said = format!(
                        "input {}: {}{e}",
                        path.display(),
                        if gone(&e) { "removed: " } else { "the keys are not read: " }
                    );
                    let _ = skyward_sys::kmsg(&said);
                    removed.push(i);
                }
            }
        }
        for i in removed.into_iter().rev() {
            let (path, _) = self.buttons.remove(i);
            self.seen.remove(&path);
        }
        out
    }
}
