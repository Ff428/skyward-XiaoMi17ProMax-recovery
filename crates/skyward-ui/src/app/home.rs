//! 首页的一栏栏：有哪些、按什么顺序摆、什么时候不摆（2026-11-28，取自
//! skyward-pudding v0.5.13）。
//!
//! 分类（想建几个建几个）各有名字与自己的条目；没进任何分类的条目落在
//! 「其他」那一栏里（[`State::home_rest`]）。两边合起来必须正好覆盖每一条
//! 一次，[`normalize`] 负责把它修成这样。

use crate::app::Screen;
use crate::icons::Icon;
use crate::lang::Lang;

/// 出厂时单独摆在首页最上面、不进任何一栏的：四条最常用的（2026-11-28 用户
/// 给的截图口径：无线网络、文件管理、模块管理、安装刷机）。
pub const DEFAULT_TOP: [Entry; 4] = [Entry::Wlan, Entry::Files, Entry::Modules, Entry::Install];

/// 内置三栏里的条目，按截图上标出来的条数：刷机备份 3、文件模块 5、系统设置 11。
const FLASH: [Entry; 3] = [Entry::FileTransfer, Entry::Backup, Entry::Restore];
const FILEWORK: [Entry; 5] = [
    Entry::Relabel,
    Entry::Serve,
    Entry::Usb,
    Entry::Filesystems,
    Entry::Fetch,
];
const SETUP: [Entry; 11] = [
    Entry::Systems,
    Entry::Root,
    Entry::DataApps,
    Entry::Terminal,
    Entry::Record,
    Entry::Charge,
    Entry::Mount,
    Entry::Partitions,
    Entry::Tasks,
    Entry::Avb,
    Entry::Decrypt,
];

/// 出厂时落在「其他」里的那 8 条，按用户手机上那一份的先后。
pub const FACTORY_REST: [Entry; 8] = [
    Entry::Settings,
    Entry::HomeOrder,
    Entry::Logs,
    Entry::Wipe,
    Entry::Update,
    Entry::SkywardUpdate,
    Entry::Reboot,
    Entry::Shutdown,
];

/// 出厂的三栏。名字、图标、折叠状态都能改，删掉也随用户。三栏出厂都折着
/// （截图上就是折的；「首页分类展开」默认关，起来本来就全折）。
pub fn default_sections(l: Lang) -> Vec<Section> {
    vec![
        Section {
            name: l.pick(["刷机备份", "刷機備份", "Flash & backup"]).to_string(),
            entries: FLASH.to_vec(),
            icon: Icon::Install,
            collapsed: true,
        },
        Section {
            name: l.pick(["文件模块", "檔案模組", "Files"]).to_string(),
            entries: FILEWORK.to_vec(),
            icon: Icon::Folder,
            collapsed: true,
        },
        Section {
            name: l.pick(["系统设置", "系統設定", "System"]).to_string(),
            entries: SETUP.to_vec(),
            icon: Icon::Gear,
            collapsed: true,
        },
    ]
}

/// 出厂的摆法：顶置四条、三栏、剩下 8 条在「其他」，总计三十一条一条不落。
pub fn factory(l: Lang) -> (Vec<Entry>, Vec<Section>, Vec<Entry>) {
    (DEFAULT_TOP.to_vec(), default_sections(l), FACTORY_REST.to_vec())
}

/// 0.8.21 那版的出厂四栏（认「没人动过」的旧摆法用）。
fn old_four(l: Lang) -> Vec<Section> {
    vec![
        Section {
            name: l.pick(["刷机备份", "刷機備份", "Flash & backup"]).to_string(),
            entries: vec![Entry::Install, Entry::Backup, Entry::Restore],
            icon: Icon::Install,
            collapsed: false,
        },
        Section {
            name: l.pick(["文件模块", "檔案模組", "Files"]).to_string(),
            entries: vec![
                Entry::FileTransfer,
                Entry::Files,
                Entry::Modules,
                Entry::DataApps,
                Entry::Usb,
                Entry::Relabel,
                Entry::Serve,
                Entry::Fetch,
            ],
            icon: Icon::Folder,
            collapsed: true,
        },
        Section {
            name: l.pick(["其他分类", "其他分類", "Tools"]).to_string(),
            entries: vec![
                Entry::Decrypt,
                Entry::Terminal,
                Entry::Partitions,
                Entry::Avb,
                Entry::Root,
                Entry::Update,
            ],
            icon: Icon::Layers,
            collapsed: true,
        },
        Section {
            name: l.pick(["系统设置", "系統設定", "System"]).to_string(),
            entries: vec![
                Entry::Systems,
                Entry::Settings,
                Entry::Reboot,
                Entry::Wipe,
                Entry::SkywardUpdate,
                Entry::HomeOrder,
                Entry::Logs,
                Entry::Charge,
            ],
            icon: Icon::Gear,
            collapsed: true,
        },
    ]
}

/// 栏名 + 条目，用来比形状；折叠状态不算（用户可能展开过）。
fn shape(sections: &[Section]) -> Vec<(String, Vec<Entry>)> {
    sections.iter().map(|s| (s.name.clone(), s.entries.clone())).collect()
}

/// 两串条目比内容不比先后：用户在「其他」里拖过顺序，不算动过形状。
fn same_set(a: &[Entry], b: &[Entry]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_by_key(|entry| entry.id());
    b.sort_by_key(|entry| entry.id());
    a == b
}

/// 把历史上「没人动过」的出厂摆法换成这一版的三栏。认两种旧形状：
/// 0.8.1 之前的一栏「刷机与备份」，以及 0.8.21 的四栏。用户自己分过组、
/// 改过名的，一律不动。
pub fn upgrade(state: &mut super::State) {
    let l = state.lang;
    let (top, sections, rest) = factory(l);

    // 一栏那版：顶上什么都没有，唯一那栏还是出厂名与出厂条目。
    let old_names = [
        l.pick(["刷机与备份", "刷機與備份", "Flash & backup"]).to_string(),
        l.pick(["刷机备份", "刷機備份", "Flash backup"]).to_string(),
    ];
    let mut old_rest = default_rest();
    old_rest.retain(|entry| !FLASH.contains(entry));
    let single = state.home_top.is_empty()
        && state.home_sections.len() == 1
        && old_names.contains(&state.home_sections[0].name)
        && state.home_sections[0].entries == vec![Entry::Install, Entry::Backup, Entry::Restore]
        && same_set(&state.home_rest, &old_rest);

    // 0.8.21 那版的四栏：顶置只有无线网络，四栏与「其他」都对得上。
    let four = state.home_top == vec![Entry::Wlan]
        && shape(&state.home_sections) == shape(&old_four(l))
        && same_set(&state.home_rest, &[Entry::Tasks, Entry::Record, Entry::Mount, Entry::Filesystems, Entry::Shutdown]);

    if !(single || four) {
        return;
    }
    state.home_top = top;
    state.home_rest_at = Some(sections.len());
    state.home_sections = sections;
    state.home_rest = rest;
}

/// 首页可以摆的一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Wlan,
    Files,
    Modules,
    DataApps,
    Install,
    Backup,
    Restore,
    Systems,
    Settings,
    Reboot,
    Wipe,
    SkywardUpdate,
    /// 打开首页排序页；它自己也是一条，可以摆在首页或任何分类里。
    HomeOrder,
    /// 文件传输：点进去是它自己的页，开关在那页上。
    FileTransfer,
    // 高级页去掉之后搬过来的：原来只在那一页里点到的东西，现在都是首页条目。
    Decrypt,
    Terminal,
    Partitions,
    Serve,
    Fetch,
    Avb,
    Root,
    Update,
    Logs,
    Charge,
    Usb,
    Relabel,
    // 本仓库高级页上还有的那几条，一起放在「其他」里，免得没有入口。
    Tasks,
    Record,
    Mount,
    Filesystems,
    Shutdown,
}

/// 从高级页搬过来的那些，按原来那一页的先后。
pub const ADVANCED: [Entry; 17] = [
    Entry::Tasks,
    Entry::Partitions,
    Entry::Record,
    Entry::Charge,
    Entry::Decrypt,
    Entry::Mount,
    Entry::Terminal,
    Entry::Filesystems,
    Entry::Serve,
    Entry::Fetch,
    Entry::Avb,
    Entry::Root,
    Entry::Update,
    Entry::Logs,
    Entry::Usb,
    Entry::Relabel,
    Entry::Shutdown,
];

/// 默认顺序：与「去掉分组之前」从上到下一致。
pub const DEFAULT_ORDER: [Entry; 14] = [
    Entry::Wlan,
    // 文件传输紧跟 WLAN：它俩都是「联不联得出去」，摆一起顺手。
    Entry::FileTransfer,
    Entry::Files,
    Entry::Modules,
    Entry::DataApps,
    Entry::Install,
    Entry::Backup,
    Entry::Restore,
    Entry::Systems,
    Entry::Settings,
    Entry::Reboot,
    Entry::Wipe,
    Entry::SkywardUpdate,
    Entry::HomeOrder,
];

/// 一栏：名字与其中的条目，按摆的顺序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub entries: Vec<Entry>,
    /// 栏目头上那个图标，从全部图标里挑。
    pub icon: Icon,
    /// 在首页折起来了。
    pub collapsed: bool,
}

impl Entry {
    /// 写进设置文件的短名字；改了就再读不回来，别动。
    pub fn id(self) -> &'static str {
        match self {
            Entry::Wlan => "wlan",
            Entry::Files => "files",
            Entry::Modules => "modules",
            Entry::DataApps => "data_apps",
            Entry::Install => "install",
            Entry::Backup => "backup",
            Entry::Restore => "restore",
            Entry::Systems => "systems",
            Entry::Settings => "settings",
            Entry::Reboot => "reboot",
            Entry::Wipe => "wipe",
            Entry::SkywardUpdate => "skyward_update",
            Entry::HomeOrder => "home_order",
            Entry::FileTransfer => "file_transfer",
            Entry::Decrypt => "decrypt",
            Entry::Terminal => "terminal",
            Entry::Partitions => "partitions",
            Entry::Serve => "serve",
            Entry::Fetch => "fetch",
            Entry::Avb => "avb",
            Entry::Root => "root",
            Entry::Update => "update",
            Entry::Logs => "logs",
            Entry::Charge => "charge",
            Entry::Usb => "usb",
            Entry::Relabel => "relabel",
            Entry::Tasks => "tasks",
            Entry::Record => "record",
            Entry::Mount => "mount",
            Entry::Filesystems => "filesystems",
            Entry::Shutdown => "shutdown",
        }
    }

    pub fn from_id(id: &str) -> Option<Entry> {
        Some(match id {
            "wlan" => Entry::Wlan,
            "files" => Entry::Files,
            "modules" => Entry::Modules,
            "data_apps" => Entry::DataApps,
            "install" => Entry::Install,
            "backup" => Entry::Backup,
            "restore" => Entry::Restore,
            "systems" => Entry::Systems,
            "settings" => Entry::Settings,
            "reboot" => Entry::Reboot,
            "wipe" => Entry::Wipe,
            "skyward_update" => Entry::SkywardUpdate,
            "home_order" => Entry::HomeOrder,
            "file_transfer" => Entry::FileTransfer,
            "decrypt" => Entry::Decrypt,
            "terminal" => Entry::Terminal,
            "partitions" => Entry::Partitions,
            "serve" => Entry::Serve,
            "fetch" => Entry::Fetch,
            "avb" => Entry::Avb,
            "root" => Entry::Root,
            "update" => Entry::Update,
            "logs" => Entry::Logs,
            "charge" => Entry::Charge,
            "usb" => Entry::Usb,
            "relabel" => Entry::Relabel,
            "tasks" => Entry::Tasks,
            "record" => Entry::Record,
            "mount" => Entry::Mount,
            "filesystems" => Entry::Filesystems,
            "shutdown" => Entry::Shutdown,
            _ => return None,
        })
    }

    /// 名字，排序页与首页用的是同一句。
    pub fn name(self, l: Lang) -> &'static str {
        match self {
            Entry::Wlan => l.pick(["无线网络", "無線網路", "WLAN"]),
            Entry::Files => l.pick(["文件管理", "檔案管理", "File manager"]),
            Entry::Modules => l.pick(["模块管理", "模組管理", "Modules"]),
            Entry::DataApps => l.pick(["应用管理", "應用程式管理", "Apps"]),
            Entry::Install => l.pick(["安装刷机", "安裝刷機", "Install"]),
            Entry::Backup => l.pick(["创建备份", "建立備份", "Backup"]),
            Entry::Restore => l.pick(["恢复备份", "還原備份", "Restore"]),
            Entry::Systems => l.pick(["第二系统", "第二系統", "Dual boot"]),
            Entry::Settings => l.pick(["选项设置", "選項設定", "Options"]),
            Entry::Reboot => l.pick(["重启系统", "重新啟動", "Reboot"]),
            Entry::Wipe => l.pick(["清除数据", "清除資料", "Wipe data"]),
            Entry::SkywardUpdate => l.pick(["版本更新", "版本更新", "Version update"]),
            Entry::HomeOrder => l.pick(["首页排序", "首頁排序", "Home order"]),
            Entry::FileTransfer => l.pick(["文件传输", "檔案傳輸", "File transfer"]),
            Entry::Decrypt => l.pick(["解密数据", "解密資料", "Decrypt data"]),
            Entry::Terminal => l.pick(["模拟终端", "模擬終端", "Terminal emulator"]),
            Entry::Partitions => l.pick(["分区管理", "分割區管理", "Partitions"]),
            Entry::Serve => l.pick(["无线传输", "無線傳輸", "Wireless transfer"]),
            Entry::Fetch => l.pick(["网络下载", "網路下載", "Download"]),
            Entry::Avb => l.pick(["校验启动", "校驗啟動", "Verified boot"]),
            Entry::Root => l.pick(["Root 管理", "Root 管理", "Root"]),
            Entry::Update => l.pick(["系统更新", "系統更新", "System update"]),
            Entry::Logs => l.pick(["导出日志", "匯出記錄", "Export logs"]),
            Entry::Charge => l.pick(["充电设置", "充電設定", "Charging"]),
            Entry::Usb => l.pick(["U 盘管理", "USB 隨身碟管理", "USB drive"]),
            Entry::Relabel => l.pick(["修复 SELinux 上下文", "修復 SELinux 上下文", "Fix SELinux contexts"]),
            Entry::Tasks => l.pick(["任务", "任務", "Tasks"]),
            Entry::Record => l.pick(["录屏", "錄影", "Screen recording"]),
            Entry::Mount => l.pick(["挂载", "掛載", "Mount"]),
            Entry::Filesystems => l.pick(["文件系统", "檔案系統", "Filesystems"]),
            Entry::Shutdown => l.pick(["关机", "關機", "Power off"]),
        }
    }

    /// 这一条在页面上显示的名字：自己起的名字优先，没有就用页面自带的。
    /// 自定义名按 `Entry::id()` 存在设置里。
    pub fn shown<'a>(self, l: Lang, names: &'a std::collections::BTreeMap<String, String>) -> &'a str {
        match names.get(self.id()) {
            Some(name) if !name.is_empty() => name.as_str(),
            _ => self.name(l),
        }
    }

    /// 点它要做的事：多数是开一页，U 盘那条是进 OTG。
    pub fn action(self) -> crate::app::Action {
        match self {
            Entry::Usb => crate::app::Action::FilesOtg,
            _ => crate::app::Action::Open(self.screen()),
        }
    }

    /// 打开哪一页。
    pub fn screen(self) -> Screen {
        match self {
            Entry::Wlan => Screen::Wlan,
            Entry::Files => Screen::Files,
            Entry::Modules => Screen::Modules,
            Entry::DataApps => Screen::DataApps,
            Entry::Install => Screen::Install,
            Entry::Backup => Screen::Backup,
            Entry::Restore => Screen::Restore,
            Entry::Systems => Screen::Systems,
            Entry::Settings => Screen::Settings,
            Entry::Reboot => Screen::Reboot,
            Entry::Wipe => Screen::Wipe,
            Entry::SkywardUpdate => Screen::SkywardUpdate,
            Entry::HomeOrder => Screen::HomeOrder,
            Entry::FileTransfer => Screen::FileTransfer,
            Entry::Decrypt => Screen::Decrypt,
            Entry::Terminal => Screen::Terminal,
            Entry::Partitions => Screen::Partitions,
            Entry::Serve => Screen::Serve,
            Entry::Fetch => Screen::Fetch,
            Entry::Avb => Screen::Avb,
            Entry::Root => Screen::Root,
            Entry::Update => Screen::Update,
            Entry::Logs => Screen::Logs,
            Entry::Charge => Screen::Charge,
            Entry::Usb => Screen::Files,
            Entry::Relabel => Screen::Relabel,
            Entry::Tasks => Screen::Tasks,
            Entry::Record => Screen::Record,
            Entry::Mount => Screen::Mount,
            Entry::Filesystems => Screen::Filesystems,
            Entry::Shutdown => Screen::Shutdown,
        }
    }

    /// 图标交给首页画行时决定颜色，形状在这里。
    pub fn icon(self) -> Icon {
        match self {
            Entry::Wlan => Icon::Wlan,
            Entry::Files => Icon::Folder,
            Entry::Modules | Entry::DataApps => Icon::Layers,
            Entry::Install | Entry::SkywardUpdate | Entry::Update | Entry::Tasks => Icon::Install,
            Entry::Backup => Icon::Backup,
            Entry::Restore => Icon::Restore,
            Entry::Systems => Icon::Systems,
            Entry::Settings => Icon::Gear,
            Entry::Reboot | Entry::Shutdown => Icon::Power,
            Entry::Wipe => Icon::Wipe,
            Entry::HomeOrder => Icon::Sort,
            Entry::FileTransfer => Icon::Usb,
            Entry::Decrypt => Icon::Lock,
            Entry::Terminal => Icon::Terminal,
            Entry::Partitions | Entry::Filesystems => Icon::Mount,
            Entry::Serve | Entry::Fetch => Icon::Globe,
            Entry::Avb | Entry::Relabel => Icon::Shield,
            Entry::Root => Icon::Shield,
            Entry::Logs => Icon::Log,
            Entry::Charge => Icon::Battery,
            Entry::Usb => Icon::UsbDrive,
            Entry::Record => Icon::Record,
            Entry::Mount => Icon::Mount,
        }
    }
}

/// 首页「其他」的默认内容：核心条目，加上高级页那十七条（高级页不再有了）。
/// 出厂的「其他」只有 [`LOOSE`] 那五条，见 [`factory`]；这一个是不重不漏的
/// 全集，`normalize` 拿它补回落在外面没人认领的条目。
pub fn default_rest() -> Vec<Entry> {
    let mut rest = DEFAULT_ORDER.to_vec();
    rest.extend(ADVANCED);
    rest
}

/// 把「分类 + 其他」修成不重不漏：分类里先到的留下，后到的重复丢掉；
/// 没被任何地方提到的条目按默认顺序补进「其他」。
pub fn normalize(sections: &mut Vec<Section>, top: &mut Vec<Entry>, rest: &mut Vec<Entry>) {
    let mut seen: Vec<Entry> = Vec::new();
    top.retain(|entry| {
        if seen.contains(entry) {
            false
        } else {
            seen.push(*entry);
            true
        }
    });
    for section in sections.iter_mut() {
        section.entries.retain(|entry| {
            if seen.contains(entry) {
                false
            } else {
                seen.push(*entry);
                true
            }
        });
    }
    rest.retain(|entry| {
        if seen.contains(entry) {
            false
        } else {
            seen.push(*entry);
            true
        }
    });
    // 每一条都得在某个地方：分类里、置顶里，或者「其他」里。
    for entry in default_rest() {
        if !seen.contains(&entry) {
            rest.push(entry);
        }
    }
}

/// 某一条现在在哪一栏：`None` 是「其他」，`Some(0..)` 是第几个分类。
pub fn section_of(sections: &[Section], entry: Entry) -> Option<usize> {
    sections.iter().position(|s| s.entries.contains(&entry))
}

/// 从它现在所在的地方拿掉。
pub fn detach(sections: &mut [Section], top: &mut Vec<Entry>, rest: &mut Vec<Entry>, entry: Entry) {
    for section in sections.iter_mut() {
        section.entries.retain(|e| *e != entry);
    }
    top.retain(|e| *e != entry);
    rest.retain(|e| *e != entry);
}

/// 「其他」那一栏还是默认顺序吗（是的话不写进设置文件）。
pub fn rest_is_default(rest: &[Entry]) -> bool {
    rest == default_rest()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn ids_go_both_ways() {
        for entry in default_rest() {
            assert_eq!(Entry::from_id(entry.id()), Some(entry));
        }
        assert_eq!(Entry::from_id("nope"), None);
    }

    #[test]
    fn a_stored_order_reads_back_and_gets_missing_entries_appended() {
        let mut order: Vec<Entry> = ids(&["restore", "install", "nonsense"])
            .iter()
            .filter_map(|id| Entry::from_id(id))
            .collect();
        let mut sections = Vec::new();
        normalize(&mut sections, &mut Vec::new(), &mut order);
        assert_eq!(&order[..2], &[Entry::Restore, Entry::Install], "记住的先后照旧");
        assert_eq!(order.len(), default_rest().len(), "一条不少");
        let mut seen = std::collections::BTreeSet::new();
        for entry in &order {
            assert!(seen.insert(entry.id()), "没有重复的条目");
        }
    }

    #[test]
    fn a_section_takes_entries_out_of_the_rest_bucket_and_duplicates_fall_away() {
        let mut sections = vec![
            Section { name: "日常".into(), entries: vec![Entry::Files, Entry::Wlan, Entry::Files], icon: Icon::Layers, collapsed: false },
            Section { name: "刷机".into(), entries: vec![Entry::Install], icon: Icon::Layers, collapsed: false },
        ];
        let mut rest = vec![Entry::Install, Entry::Backup];
        normalize(&mut sections, &mut Vec::new(), &mut rest);
        assert_eq!(sections[0].entries, vec![Entry::Files, Entry::Wlan], "重复的丢掉");
        assert!(!rest.contains(&Entry::Install), "分类拿走的不会同时留在其他");
        assert_eq!(rest[0], Entry::Backup, "其他里原有的先后不变");
        assert_eq!(sections.len(), 2);
    }

    #[test]
    fn the_default_rest_holds_every_entry_once() {
        let rest = default_rest();
        let mut seen = std::collections::BTreeSet::new();
        for entry in &rest {
            assert!(seen.insert(entry.id()), "{} 只出现一次", entry.id());
        }
        assert_eq!(rest.len(), DEFAULT_ORDER.len() + ADVANCED.len());
    }

    #[test]
    fn the_built_in_sections_are_the_three_on_the_home_page() {
        let sections = default_sections(Lang::Hans);
        let names: Vec<&str> = sections.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["刷机备份", "文件模块", "系统设置"]);
        assert_eq!(sections[0].entries.len(), 3, "刷机备份 3");
        assert_eq!(
            sections[0].entries,
            vec![Entry::FileTransfer, Entry::Backup, Entry::Restore]
        );
        assert_eq!(sections[1].entries.len(), 5, "文件模块 5");
        assert_eq!(sections[2].entries.len(), 11, "系统设置 11");
        assert!(sections.iter().all(|s| s.collapsed), "三栏出厂都折着");
    }

    #[test]
    fn the_factory_layout_holds_every_entry_exactly_once() {
        let (top, sections, rest) = factory(Lang::Hans);
        assert_eq!(
            top,
            vec![Entry::Wlan, Entry::Files, Entry::Modules, Entry::Install],
            "截图上顶置的四条"
        );
        let mut seen = std::collections::BTreeSet::new();
        for entry in top.iter().chain(sections.iter().flat_map(|s| s.entries.iter())).chain(rest.iter()) {
            assert!(seen.insert(entry.id()), "{} 只出现一次", entry.id());
        }
        assert_eq!(seen.len(), default_rest().len(), "三十一条一条不少");
        assert_eq!(rest, FACTORY_REST.to_vec(), "「其他」里是这 8 条");
        assert_eq!(rest.len(), 8);
    }

    #[test]
    fn the_old_single_section_layout_is_upgraded_once() {
        let mut state = crate::app::State::default();
        let mut old_rest = default_rest();
        old_rest.retain(|entry| !FLASH.contains(entry));
        state.home_sections = vec![Section {
            name: "刷机与备份".into(),
            entries: vec![Entry::Install, Entry::Backup, Entry::Restore],
            icon: Icon::Install,
            collapsed: false,
        }];
        state.home_rest = old_rest;
        upgrade(&mut state);
        assert_eq!(state.home_sections.len(), 3, "换成这一版的三栏");
        assert_eq!(state.home_rest, FACTORY_REST.to_vec());
        // 再跑一次不动它：升级只认那两种旧形状。
        let before = state.home_sections.len();
        upgrade(&mut state);
        assert_eq!(state.home_sections.len(), before);
    }

    #[test]
    fn the_0821_four_section_layout_is_upgraded_to_the_screenshot_one() {
        let mut state = crate::app::State::default();
        state.home_top = vec![Entry::Wlan];
        state.home_sections = old_four(Lang::Hans);
        state.home_rest = vec![Entry::Tasks, Entry::Record, Entry::Mount, Entry::Filesystems, Entry::Shutdown];
        upgrade(&mut state);
        assert_eq!(state.home_top, DEFAULT_TOP.to_vec());
        assert_eq!(state.home_sections.len(), 3);
        assert_eq!(state.home_rest, FACTORY_REST.to_vec());
    }

    #[test]
    fn a_layout_the_user_made_is_left_alone() {
        let mut state = crate::app::State::default();
        state.home_top = vec![Entry::Wlan, Entry::Files];
        state.home_sections = old_four(Lang::Hans);
        state.home_rest = vec![Entry::Tasks, Entry::Record, Entry::Mount, Entry::Filesystems, Entry::Shutdown];
        let before = shape(&state.home_sections);
        upgrade(&mut state);
        assert_eq!(shape(&state.home_sections), before, "顶置动过就不算旧形状");
    }
}
