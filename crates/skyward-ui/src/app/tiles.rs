//! 控制中心摆的磁贴：有哪几种、各叫什么、按什么顺序摆。
//!
//! 摆法存在 `/metadata/skyward/tiles` 里，一行一个磁贴的名字；没存过时
//! 按 [`DEFAULT`] 摆。名字是给文件用的，改一个字就对不上旧文件。

use super::*;

/// 控制中心可以摆的一种磁贴。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    // 头十二个是 v124 面板上原来那十二枚，也是没存过时的默认摆法：升级
    // 上来看到的面板跟以前一模一样，后加的十二枚在「添加磁贴」里等着。
    Wlan,
    Screenshot,
    Reboot,
    Record,
    Language,
    Files,
    Terminal,
    Mount,
    Update,
    Backup,
    Settings,
    Shutdown,
    // 2026-11-22 追加：开关型的三枚与原来只在页面里点得到的东西。
    Vibrate,
    FileTransfer,
    Usb,
    Modules,
    Partitions,
    Restore,
    Filesystems,
    Logs,
    DataApps,
    Systems,
    Tasks,
    Fetch,
    ScreenOff,
    Avb,
    Serve,
    /// 分区镜像：导出、写回、删除。
    ImageBackup,
    /// Root 管理：KernelSU / Magisk 与在线写入。
    Root,
    /// 2026-11-28 追加：高级页去掉之后，原来只在那一页里点到的东西各领一枚。
    Decrypt,
    SkywardUpdate,
    Relabel,
    HomeOrder,
    /// 布局备份：把首页与磁贴的摆法写成一个文件，也能读回来。
    Layout,
}

/// 没存过时摆的：v124 那十二枚，一行四个、三排。
pub const DEFAULT: [Tile; 12] = [
    Tile::Wlan,
    Tile::Screenshot,
    Tile::Reboot,
    Tile::Record,
    Tile::Language,
    Tile::Files,
    Tile::Terminal,
    Tile::Mount,
    Tile::Update,
    Tile::Backup,
    Tile::Settings,
    Tile::Shutdown,
];

/// 全部磁贴，按默认摆的顺序排：前十二个是 [`DEFAULT`]，后面是后加的。
pub const TILES: [Tile; 34] = [
    Tile::Wlan,
    Tile::Screenshot,
    Tile::Reboot,
    Tile::Record,
    Tile::Language,
    Tile::Files,
    Tile::Terminal,
    Tile::Mount,
    Tile::Update,
    Tile::Backup,
    Tile::Settings,
    Tile::Shutdown,
    Tile::Vibrate,
    Tile::FileTransfer,
    Tile::Usb,
    Tile::Modules,
    Tile::Partitions,
    Tile::Restore,
    Tile::Filesystems,
    Tile::Logs,
    Tile::DataApps,
    Tile::Systems,
    Tile::Tasks,
    Tile::Fetch,
    Tile::ScreenOff,
    Tile::Avb,
    Tile::Serve,
    Tile::ImageBackup,
    Tile::Root,
    Tile::Decrypt,
    Tile::SkywardUpdate,
    Tile::Relabel,
    Tile::HomeOrder,
    Tile::Layout,
];

impl Tile {
    /// 存进文件的名字：认得出是哪一个就行。
    pub fn id(self) -> &'static str {
        match self {
            Tile::Wlan => "wlan",
            Tile::Screenshot => "screenshot",
            Tile::Reboot => "reboot",
            Tile::Record => "record",
            Tile::Language => "language",
            Tile::Files => "files",
            Tile::Terminal => "terminal",
            Tile::Mount => "mount",
            Tile::Update => "update",
            Tile::Backup => "backup",
            Tile::Settings => "settings",
            Tile::Shutdown => "shutdown",
            Tile::Vibrate => "vibrate",
            Tile::FileTransfer => "mtp",
            Tile::Usb => "usb",
            Tile::Restore => "restore",
            Tile::Modules => "modules",
            Tile::Partitions => "partitions",
            Tile::Filesystems => "filesystems",
            Tile::Logs => "logs",
            Tile::DataApps => "dataapps",
            Tile::Systems => "systems",
            Tile::Tasks => "tasks",
            Tile::Fetch => "fetch",
            Tile::ScreenOff => "screenoff",
            Tile::Avb => "avb",
            Tile::Serve => "serve",
            Tile::ImageBackup => "image-backup",
            Tile::Root => "root",
            Tile::Decrypt => "decrypt",
            Tile::SkywardUpdate => "skywardupdate",
            Tile::Relabel => "relabel",
            Tile::HomeOrder => "homeorder",
            Tile::Layout => "layout",
        }
    }

    /// 文件里这个名字说的是哪一个；不是磁贴就是 `None`。
    pub fn from_id(id: &str) -> Option<Tile> {
        TILES.into_iter().find(|t| t.id() == id.trim())
    }

    /// 磁贴下面那行字。WLAN 的开关状态与语言的名字在画的时候另说。
    pub fn label(self, l: Lang) -> &'static str {
        match self {
            Tile::Wlan => "WLAN",
            Tile::Screenshot => l.pick(["截图", "截圖", "Screenshot"]),
            Tile::Reboot => l.pick(["重启", "重新啟動", "Reboot"]),
            Tile::Record => l.pick(["录屏", "錄影", "Record"]),
            Tile::Language => l.pick(["语言", "語言", "Language"]),
            Tile::Files => l.pick(["文件", "檔案", "Files"]),
            Tile::Terminal => l.pick(["终端", "終端機", "Terminal"]),
            Tile::Mount => l.pick(["挂载", "掛載", "Mount"]),
            Tile::Update => l.pick(["更新", "更新", "Update"]),
            Tile::Backup => l.pick(["备份", "備份", "Backup"]),
            Tile::Settings => l.pick(["设置", "設定", "Options"]),
            Tile::Shutdown => l.pick(["关机", "關機", "Power off"]),
            Tile::Vibrate => l.pick(["震动", "震動", "Haptics"]),
            Tile::FileTransfer => "MTP",
            Tile::Usb => l.pick(["U 盘", "USB 隨身碟", "USB drive"]),
            Tile::Restore => l.pick(["恢复", "還原", "Restore"]),
            Tile::Modules => l.pick(["模块", "模組", "Modules"]),
            Tile::Partitions => l.pick(["分区", "分割區", "Partitions"]),
            Tile::Filesystems => l.pick(["文件系统", "檔案系統", "Filesystems"]),
            Tile::Logs => l.pick(["日志", "日誌", "Logs"]),
            Tile::DataApps => l.pick(["应用", "應用", "Apps"]),
            Tile::Systems => l.pick(["第二系统", "第二系統", "Dual boot"]),
            Tile::Tasks => l.pick(["任务", "任務", "Tasks"]),
            Tile::Fetch => l.pick(["下载", "下載", "Download"]),
            Tile::ScreenOff => l.pick(["息屏", "螢幕逾時", "Screen off"]),
            Tile::Avb => l.pick(["AVB", "AVB", "AVB"]),
            Tile::Root => l.pick(["Root", "Root", "Root"]),
            Tile::Serve => l.pick(["无线传输", "無線傳輸", "Wi-Fi share"]),
            Tile::ImageBackup => l.pick(["分区镜像", "分割區映像檔", "Images"]),
            Tile::Decrypt => l.pick(["解密数据", "解密資料", "Decrypt"]),
            Tile::SkywardUpdate => l.pick(["版本更新", "版本更新", "Version update"]),
            Tile::Relabel => l.pick(["文件标签", "檔案標籤", "Relabel"]),
            Tile::HomeOrder => l.pick(["首页排序", "首頁排序", "Home order"]),
            Tile::Layout => l.pick(["布局备份", "佈局備份", "Layout backup"]),
        }
    }

    /// 它在默认顺序里的位置：「可添加」那段按这个排，删了又加也回到原处。
    pub fn rank(self) -> usize {
        TILES.iter().position(|&t| t == self).unwrap_or(TILES.len())
    }

    /// 还没摆上去的磁贴，按默认顺序。
    pub fn left(on: &[Tile]) -> Vec<Tile> {
        let mut left: Vec<Tile> = TILES.into_iter().filter(|t| !on.contains(t)).collect();
        left.sort_by_key(|t| t.rank());
        left
    }

    /// 磁贴的图标；`None` 是这一枚自己手绘（截图那台相机、录屏那个红点）。
    pub fn icon(self) -> Option<crate::icons::Icon> {
        use crate::icons::Icon;
        Some(match self {
            Tile::Wlan => Icon::Wlan,
            Tile::Screenshot | Tile::Record => return None,
            Tile::Reboot => Icon::Restore,
            Tile::Language => Icon::Globe,
            Tile::Files => Icon::Folder,
            Tile::Terminal => Icon::Terminal,
            Tile::Mount => Icon::Mount,
            Tile::Update => Icon::Layers,
            Tile::Backup => Icon::Backup,
            Tile::Settings => Icon::Gear,
            Tile::Shutdown => Icon::Power,
            Tile::Vibrate => Icon::Phone,
            Tile::FileTransfer => Icon::Usb,
            Tile::Usb => Icon::UsbDrive,
            Tile::Restore => Icon::Restore,
            Tile::Modules => Icon::Layers,
            Tile::Partitions => Icon::Mount,
            Tile::Filesystems => Icon::Layers,
            Tile::Logs => Icon::Log,
            Tile::DataApps => Icon::Folder,
            Tile::Systems => Icon::Systems,
            Tile::Tasks => Icon::Clock,
            Tile::Fetch => Icon::Install,
            Tile::ScreenOff => Icon::Clock,
            Tile::Avb => Icon::Shield,
            Tile::Root => Icon::Shield,
            Tile::Serve => Icon::Globe,
            Tile::ImageBackup => Icon::Backup,
            Tile::Decrypt => Icon::Lock,
            Tile::SkywardUpdate => Icon::Install,
            Tile::Relabel => Icon::Layers,
            Tile::HomeOrder => Icon::Sort,
            Tile::Layout => Icon::Backup,
        })
    }

    /// 这一枚一按打开哪一页；开关、截图、录屏、语言自己另有做法，是 `None`。
    pub fn screen(self) -> Option<Screen> {
        match self {
            Tile::Wlan
            | Tile::Screenshot
            | Tile::Record
            | Tile::Language
            | Tile::Vibrate
            | Tile::FileTransfer => None,
            Tile::Reboot => Some(Screen::Reboot),
            Tile::Files => Some(Screen::Files),
            Tile::Terminal => Some(Screen::Terminal),
            Tile::Mount => Some(Screen::Mount),
            Tile::Update => Some(Screen::Update),
            Tile::Backup => Some(Screen::Backup),
            Tile::Usb => Some(Screen::FilesOtg),
            Tile::Restore => Some(Screen::Restore),
            Tile::Settings => Some(Screen::Settings),
            Tile::Shutdown => Some(Screen::Shutdown),
            Tile::Modules => Some(Screen::Modules),
            Tile::Partitions => Some(Screen::Partitions),
            Tile::Filesystems => Some(Screen::Filesystems),
            Tile::Logs => Some(Screen::Logs),
            Tile::DataApps => Some(Screen::DataApps),
            Tile::Systems => Some(Screen::Systems),
            Tile::Tasks => Some(Screen::Tasks),
            Tile::Fetch => Some(Screen::Fetch),
            Tile::ScreenOff => Some(Screen::ScreenOff),
            Tile::Avb => Some(Screen::Avb),
            Tile::Root => Some(Screen::Root),
            Tile::Serve => Some(Screen::Serve),
            Tile::ImageBackup => Some(Screen::ImageBackup),
            Tile::Decrypt => Some(Screen::Decrypt),
            Tile::SkywardUpdate => Some(Screen::SkywardUpdate),
            Tile::Relabel => Some(Screen::Relabel),
            Tile::HomeOrder => Some(Screen::HomeOrder),
            Tile::Layout => Some(Screen::Layout),
        }
    }

    /// 磁贴开着没有：开着的那枚染上主色。开关型的说实话，其余的一按就跳页，
    /// 画成关着的样子。
    pub fn lit(self, s: &State) -> bool {
        match self {
            Tile::Wlan => s.wlan.as_ref().is_some_and(|w| w.power == Power::On),
            Tile::Record => s.record.on,
            Tile::Vibrate => s.vibrate,
            Tile::FileTransfer => s.file_transfer,
            _ => false,
        }
    }
}

/// 把文件里读到的名字读成一串磁贴：认不出的丢掉，重复的只留头一个。
pub fn from_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Vec<Tile> {
    let mut out: Vec<Tile> = Vec::new();
    for id in ids {
        if let Some(tile) = Tile::from_id(id)
            && !out.contains(&tile)
        {
            out.push(tile);
        }
    }
    out
}

/// 把一串磁贴写成文件的样子：一行一个名字。
pub fn to_text(tiles: &[Tile]) -> String {
    let mut out = String::new();
    for tile in tiles {
        out.push_str(tile.id());
        out.push('\n');
    }
    out
}

impl State {
    /// 还没摆上去的磁贴，按默认顺序。
    pub fn tiles_left(&self) -> Vec<Tile> {
        Tile::left(&self.tiles)
    }

    /// 把磁贴 `tile` 从控制中心拿下来；本来就没摆就不动。
    pub fn tile_remove(&mut self, tile: Tile) -> bool {
        match self.tiles.iter().position(|&t| t == tile) {
            Some(at) => {
                self.tiles.remove(at);
                true
            }
            None => false,
        }
    }

    /// 把磁贴 `tile` 摆到指定的一格；`at` 超出范围就落在最后。
    pub fn tile_insert(&mut self, tile: Tile, at: usize) -> bool {
        if self.tiles.contains(&tile) {
            return false;
        }
        let at = at.min(self.tiles.len());
        self.tiles.insert(at, tile);
        true
    }

    /// 把磁贴 `tile` 摆到最后一格；已经摆着就不动。
    pub fn tile_add(&mut self, tile: Tile) -> bool {
        self.tile_insert(tile, self.tiles.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 把面板整个画一遍：`shade_hits` 是画的时候填的，只画页面不填它。
    fn paint(app: &mut App) {
        let f = crate::app::testing::fonts();
        let i = crate::app::testing::info();
        let (w, h) = (crate::app::testing::W, crate::app::testing::H);
        let mut c = crate::canvas::Canvas::new(w, h);
        app.compose(&mut c, &f, &i, "12:00", 0);
    }

    #[test]
    fn a_tile_keeps_its_name_through_the_file() {
        for tile in TILES {
            assert_eq!(Tile::from_id(tile.id()), Some(tile), "{tile:?} 的名字要读得回来");
        }
        assert_eq!(Tile::from_id("no such tile"), None);
        assert_eq!(Tile::from_id("  wlan \n"), Some(Tile::Wlan), "两边空着也认");
    }

    #[test]
    fn a_file_read_back_drops_what_is_not_a_tile_and_keeps_one_of_each() {
        let read = from_ids(["wlan", "nope", "files", "wlan", "", "record"]);
        assert_eq!(read, vec![Tile::Wlan, Tile::Files, Tile::Record]);
        assert_eq!(from_ids(std::iter::empty()), Vec::new());
    }

    #[test]
    fn the_text_goes_round_the_file() {
        let tiles = vec![Tile::Record, Tile::Wlan, Tile::Settings];
        let text = to_text(&tiles);
        assert_eq!(text, "record\nwlan\nsettings\n");
        assert_eq!(from_ids(text.lines()), tiles);
    }

    #[test]
    fn a_tile_taken_off_goes_back_to_its_own_place_among_those_left() {
        let mut s = State::default();
        assert!(s.tile_remove(Tile::Files));
        assert!(!s.tiles.contains(&Tile::Files));
        assert!(!s.tile_remove(Tile::Files), "本来就没摆");
        let left = s.tiles_left();
        assert!(left.contains(&Tile::Files), "拿下来的回到「可添加」里");
        assert!(s.tile_add(Tile::Files));
        assert_eq!(s.tiles.last(), Some(&Tile::Files), "加上去的落在最后一格");
        assert!(!s.tile_add(Tile::Files), "已经摆着就不再摆一个");
    }

    #[test]
    fn a_tile_dropped_in_the_middle_lands_there() {
        let mut s = State::default();
        assert!(s.tile_insert(Tile::Tasks, 0));
        assert_eq!(s.tiles.first(), Some(&Tile::Tasks));
        assert!(s.tile_insert(Tile::Avb, 3));
        assert_eq!(s.tiles.get(3), Some(&Tile::Avb));
        assert!(!s.tile_insert(Tile::Avb, 0), "已经摆着就不动");
    }

    #[test]
    fn the_first_twelve_stand_by_default_and_the_rest_wait_below() {
        let s = State::default();
        assert_eq!(s.tiles, DEFAULT.to_vec(), "默认还是 v124 那十二枚");
        let left = s.tiles_left();
        assert_eq!(left.len(), TILES.len() - DEFAULT.len(), "后加的都在「可添加」里");
        assert_eq!(left.first(), Some(&Tile::Vibrate), "「可添加」按默认顺序排");
        assert_eq!(left.last(), Some(&Tile::Layout), "最后加的一枚是布局备份");
        assert_eq!(TILES.len(), 34, "老的三十枚去掉高级工具，补上五枚新的");
        assert!(Tile::from_id("advanced").is_none(), "高级工具那一枚不再有了");
    }

    /// 每一枚能开页的磁贴，按下去都打开它自己那一页（2026-11-22）。
    #[test]
    fn every_tile_that_opens_a_page_opens_its_own() {
        for tile in TILES {
            if let Some(screen) = tile.screen() {
                let f = crate::app::testing::fonts();
                let mut app = App::new(
                    Screen::Home,
                    crate::app::testing::W,
                    crate::app::testing::H,
                    crate::app::testing::PPD,
                );
                app.state.tiles = vec![tile];
                app.shade = 1.0;
                paint(&mut app);
                let (x, y) = app
                    .shade_hits
                    .iter()
                    .find(|(_, h)| *h == ShadeHit::Tile(tile))
                    .expect("接手指")
                    .0
                    .centre();
                app.touch(Touch::Down(x, y), &f, 1000);
                app.touch(Touch::Up, &f, 1000);
                if screen == Screen::FilesOtg {
                    // U 盘那一页没插盘就退回来：只要面板收起来了就算按到了。
                    assert!(app.shade == 0.0, "{tile:?} 按下该把面板收起来");
                    continue;
                }
                assert_eq!(app.screen(), screen, "{tile:?} 该打开 {screen:?}");
            }
        }
    }

    /// 点时间行右边那支笔就进编辑态；磁贴不再靠长按（2026-11-22）。
    #[test]
    fn the_pencil_at_the_top_right_opens_the_editor() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Record, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        paint(&mut app);
        assert!(!app.shade_editing, "拉下来先是平常那个样子");
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesEdit).expect("笔接手指").0.centre();
        let fx = {
            app.touch(Touch::Down(x, y), &f, 1000);
            app.touch(Touch::Up, &f, 1000)
        };
        assert!(app.shade_editing, "点一下就进编辑态");
        assert_eq!(fx, vec![Effect::Haptic], "进的时候马达点一下");
        assert!(app.shade_drag.is_none(), "不是按出来的，不拎着任何一枚");
    }

    /// 磁贴按一下就是它自己那个动作，不会因为按久了变成编辑（2026-11-22）。
    #[test]
    fn holding_a_tile_does_not_open_the_editor_any_more() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Record, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Record)).expect("第一枚接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.tick(1000 + LONG_PRESS_MS * 3);
        assert!(!app.shade_editing, "按再久也不进编辑态");
        assert!(app.shade_drag.is_none(), "也没有被拎起来的磁贴");
    }

    /// 编辑态下按住一枚磁贴拖走，松手落进手指指着的那一格（2026-11-22）。
    #[test]
    fn a_tile_dragged_in_the_editor_lands_where_the_finger_lets_it_go() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Record, Tile::Files, Tile::Settings, Tile::Wlan];
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Record)).expect("接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        assert_eq!(app.shade_drag.map(|d| d.tile), Some(Tile::Record), "编辑时按住就拎起来");
        let grid = TileGrid::of(&app.theme(), app.shade_top());
        let (tx, ty) = grid.centre(2);
        app.touch(Touch::Move(tx, ty), &f, 1100);
        let fx = app.touch(Touch::Up, &f, 1200);
        assert_eq!(app.state.tiles, vec![Tile::Files, Tile::Settings, Tile::Record, Tile::Wlan], "落进第三格");
        assert!(app.shade_drag.is_none(), "松手就不拎着了");
        assert!(
            fx.iter().any(|e| matches!(e, Effect::KeepTiles(t) if t == &app.state.tiles)),
            "落完要存下来：{fx:?}"
        );
    }

    /// 编辑时点圆钮右上角的小叉，那枚就没了；点下面的候选磁贴，那枚就上来
    /// （2026-11-22）。
    #[test]
    fn the_editor_takes_tiles_off_and_puts_them_back_by_tapping() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Record, Tile::Files];
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileRemove(Tile::Files)).expect("叉接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.state.tiles, vec![Tile::Record], "点叉就没了");
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileAdd(Tile::Terminal)).expect("候选接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1100);
        app.touch(Touch::Up, &f, 1100);
        assert_eq!(app.state.tiles, vec![Tile::Record, Tile::Terminal], "点候选就摆上来");
    }

    /// 后加的那十二枚也在候选区里，点一下就能摆上来（2026-11-22）。
    #[test]
    fn the_new_tiles_stand_in_the_candidate_area_too() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = DEFAULT.to_vec();
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        for tile in [Tile::Modules, Tile::Partitions, Tile::Tasks, Tile::Serve] {
            let found = app.shade_hits.iter().any(|(_, h)| *h == ShadeHit::TileAdd(tile));
            assert!(found, "{tile:?} 该在候选区里");
        }
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileAdd(Tile::Modules)).expect("候选接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.state.tiles.last(), Some(&Tile::Modules), "点一下就摆到最后一格");
    }

    /// 编辑时点「完成」收起来；点圆钮本身不执行磁贴，免得编辑时误触
    /// （2026-11-22）。
    #[test]
    fn done_ends_the_editing_and_a_tap_does_not_fire_a_tile_while_editing() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Files)).expect("接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        let fx = app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.screen(), Screen::Home, "编辑时点圆钮不跳页：{fx:?}");
        assert!(app.shade > 0.0, "面板也没被收掉");
        assert!(app.shade_editing, "还在编辑");
        paint(&mut app);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("完成接手指").0.centre();
        app.touch(Touch::Down(x, y), &f, 1100);
        app.touch(Touch::Up, &f, 1100);
        assert!(!app.shade_editing, "点完成就收起来");
    }

    /// 面板收起来，编辑也一起完：下次拉下来还是平常那个样子（2026-11-22）。
    #[test]
    fn closing_the_sheet_ends_the_editing() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        app.touch(Touch::Down(700.0, H as f32 - 4.0), &f, 1000);
        assert_eq!(app.shade, 0.0, "点面板外面就收起来");
        assert!(!app.shade_editing, "编辑也跟着完");
    }

    /// 编辑过、把面板拖回去收起来之后，下一次还得拉得下来：`shade_editing`
    /// 留在那儿，`Touch::Move` 直接 return，面板再也拉不开（2026-11-22 修）。
    #[test]
    fn the_sheet_still_pulls_down_after_it_was_edited_and_put_away() {
        use crate::app::testing::*;
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.touch(Touch::Down(700.0, 8.0), &f, 1000);
        app.touch(Touch::Move(700.0, 900.0), &f, 1050);
        app.touch(Touch::Up, &f, 1100);
        app.tick(3000);
        assert_eq!(app.shade, 1.0, "拉开到底了");
        app.shade_editing = true;
        app.touch(Touch::Down(700.0, 60.0), &f, 4000);
        app.touch(Touch::Move(700.0, 120.0), &f, 4050);
        app.touch(Touch::Move(700.0, 20.0), &f, 4100);
        app.touch(Touch::Move(700.0, -400.0), &f, 4150);
        app.touch(Touch::Up, &f, 4200);
        app.tick(6000);
        assert_eq!(app.shade, 0.0, "收起来了");
        assert!(!app.shade_editing, "编辑也跟着收掉");
        app.touch(Touch::Down(700.0, 8.0), &f, 8000);
        app.touch(Touch::Move(700.0, 900.0), &f, 8050);
        app.touch(Touch::Up, &f, 8100);
        app.tick(10000);
        assert_eq!(app.shade, 1.0, "还拉得下来");
    }

    /// 平常态不该有编辑相关的东西：它们只在编辑态才接手指（2026-11-22）。
    #[test]
    fn the_sheet_says_nothing_about_editing_while_it_is_not() {
        use crate::app::testing::*;
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        paint(&mut app);
        let editing_hits: Vec<&ShadeHit> = app
            .shade_hits
            .iter()
            .map(|(_, h)| h)
            .filter(|h| matches!(h, ShadeHit::TilesDone | ShadeHit::TileAdd(_) | ShadeHit::TileRemove(_)))
            .collect();
        assert!(editing_hits.is_empty(), "平常态不该有编辑的东西：{editing_hits:?}");
    }

    /// 「完成」的圆心要跟最后一列圆钮的圆心在同一条竖线上，不是跟命中格的
    /// 中心：一格比圆钮宽、圆钮在格里居中（2026-11-22）。
    #[test]
    fn the_done_circle_stands_over_the_last_column_of_tiles() {
        use crate::app::testing::*;
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Wlan, Tile::Screenshot, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.shade_editing = true;
        paint(&mut app);
        let grid = TileGrid::of(&app.theme(), app.shade_top());
        let (last_cx, _) = grid.centre(3);
        let done = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("完成接手指").0;
        let (done_cx, _) = done.centre();
        assert!((done_cx - last_cx).abs() < 0.5, "完成圆心 {done_cx} 该在最后一列圆钮的竖线上 {last_cx}");
    }

    /// 面板收起来时录屏胶囊还在右上角 46dp 那个老位置（2026-11-22）。
    #[test]
    fn the_recording_pill_stays_where_it_was_with_the_sheet_away() {
        use crate::app::testing::*;
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.record.on = true;
        app.state.record.elapsed = "1:23".into();
        let mut c = crate::canvas::Canvas::new(W, H);
        app.compose(&mut c, &f, &i, "12:00", 0);
        let pill = app.record_pill.expect("录屏时有胶囊");
        let m = app.theme().m;
        assert!((pill.body.y - m.dp(46.0)).abs() < 0.5, "面板收着时仍在 {}，是 {}", m.dp(46.0), pill.body.y);
    }

    /// 面板拉开时整条胶囊挪到面板顶上：不压时间、不压日期、不压那支笔
    /// （2026-11-22）。
    #[test]
    fn the_recording_pill_moves_out_of_the_way_under_the_sheet() {
        use crate::app::testing::*;
        let (f, i) = (fonts(), info());
        for editing in [false, true] {
            let mut app = App::new(Screen::Home, W, H, PPD);
            app.shade = 1.0;
            app.shade_editing = editing;
            app.state.record.on = true;
            app.state.record.elapsed = "1:23".into();
            let mut c = crate::canvas::Canvas::new(W, H);
            app.compose(&mut c, &f, &i, "12:00", 0);
            let pill = app.record_pill.expect("录屏时有胶囊");
            let m = app.theme().m;
            let top = app.shade_top();
            assert!(pill.stop.w > 0.0, "编辑 {editing}：结束键还在");
            assert!(pill.body.w > m.dp(80.0), "编辑 {editing}：不是缩小的圆点");
            assert!(
                (pill.body.y - top - m.dp(4.0)).abs() < 0.5,
                "编辑 {editing}：胶囊该贴在面板顶上，实际 y {}",
                pill.body.y
            );
            let clock_top = top + m.dp(86.0) - m.dp(42.0) * 0.78;
            assert!(pill.body.y + pill.body.h <= clock_top, "编辑 {editing}：不压时间");
            let button = app
                .shade_hits
                .iter()
                .find(|(_, h)| matches!(h, ShadeHit::TilesEdit | ShadeHit::TilesDone))
                .expect("右上角有按钮")
                .0;
            let (p, b) = (pill.body, button);
            let overlap = p.contains(b.x, b.y) || p.contains(b.x + b.w, b.y + b.h) || b.contains(p.x, p.y) || b.contains(p.x + p.w, p.y + p.h);
            assert!(!overlap, "编辑 {editing}：胶囊 {p:?} 不该压住按钮 {b:?}");
        }
    }
}
