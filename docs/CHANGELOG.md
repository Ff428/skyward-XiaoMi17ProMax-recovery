# 版本记录

镜像名固定：`skyward-OS4.0.8.0-XiaoMi17ProMax.img`（品牌版，首页与关于页写「卑鄙坑Baby」）
与 `skyward-OS4.0.8.0-XiaoMi17ProMax-official.img`（官方版，写 Skyward），两版只差品牌字，
其余字节一致。文件名里的 `4.0.8.0` 是交付代号，底座是官方 `OS4.0.6.0.XPBCNXM`。

出包见 [BUILD.md](BUILD.md)，用法见 [使用.md](使用.md)，环境与坑见 [../AGENTS.md](../AGENTS.md)。
CI 发布的 `<tag>-release-notes.md` 就是这份文件。

---

# 0.8.21 · 「首页分类展开」只留一处：排序页「新建分类」下面

上一版把它做成了首页条目（能拖进任何一栏），设置里也留了一份。现在按用户要的收成
一处：**首页排序页底部的操作列表里，紧跟在「新建分类」后面**，行尾是开关，点一下换档。

- 删掉 `Entry::HomeOpen`（首页条目、`default_rest` 里那一条、排序页里的可拖动条目）；
- 删掉设置·通用里那一行；
- 语义不变：开 = 记住每栏上次的展开/收起，关 = 每次起来全折起。

老 `settings.json` 里的 `"home_open"` 条目会被 `Entry::from_id` 过滤掉（未知 id
直接丢），不用手工清。

# 0.8.20 · 「首页分类展开」= 记住上次展开的那几栏，而且它自己也是一条

上一版理解错了：那个开关不是「一下全摊开」，是**记不记得**。现在：

- **开**（默认）：按每栏上次的折叠状态来，重进 recovery 就是那个样子。
- **关**：不用记，每次起来全折起。

`Settings.home_open`（老文件里没有这一项时按「开」算），进页面时若为关就把所有
`collapsed` 置真。

同时这条也做成了**首页上的一条普通条目**（`Entry::HomeOpen`，id `home_open`）：
排序页里能拖、能放进任何一栏，行尾直接显示开关；老设置里没有它，`normalize` 会
把它补进「其他」。

真机验证（改 settings.json 再重启，避免手点误差）：

| 文件里 | 重启后 |
|---|---|
| `home_open=true`，只第 3 栏 collapsed=false | 第 3 栏摊开，另两栏折着 |
| `home_open=false`，全部 collapsed=false | 三栏全折起 |

# 0.8.19 · 首页分类展开开关、拖动不再被每秒轮询拖住

## 一、首页分类展开（设置 · 通用）

新开关「首页分类展开」：开着 = 每栏都摊开，关掉 = 每栏都折起。单独点某一栏的栏头
只改那一栏，而且**改了就记住**（`Settings.home_sections[].collapsed`），重进 recovery
还是那个样子。真机验证：开关打开 → 四栏全摊开 → 重启 → 还是摊开。

## 二、拖动时不再跑每秒轮询

真机上量过：拖动时一轮循环 10-21 ms，而「画一帧」不到 1 ms —— 时间全花在每轮都要
跑的那一串：读电池、看 USB、各个后台任务的进度、设置落盘。这些一秒一次就够，
拖动时统统跳过（`App::dragging()`），只收输入 + 画那一帧。

**踩到的坑**：一开始写成「拖动时这一轮直接 return」，结果 `take_in`（读触摸事件）
被一起跳过了 —— 手指还在动、事件没人读，拖动直接僵死。现在只跳过轮询那一段，
`take_in` 照跑。修完再量：拖动全程没有一帧超过 8 ms。

拖动节流 28 ms → 16 ms（和屏幕帧对齐）。

## 三、落点按每一行的真实位置算

之前落点用「第一行的顶 + 行号 × 行高」硬算，可栏与栏之间还有**栏头**，每过一栏就
偏一格 —— 于是「拖到『充电设置』上面，结果跑到它下面」。现在拎起来那一刻把整页每一
行真实的顶记下来（`App.drag_edges`），手指落在哪一行就顶掉那一格；最后一行过了中线
就是整页最底下。条与条之间不再差一格。

## 四、分区表：导出行跟着选中的那条

「导出「…」到内部存储」从页顶搬到**选中那条的下面**（也走虚拟列表，序号后移一位），
选完顺手往下滚一行，让它落在屏幕里，不用再滑回顶部。

## 真机验证

- 拖「创建备份」到「充电设置」上松手：`top=[… Backup, Charge …]`，创建备份在**上面**。
- 顶置里把「应用管理」拖到第一条：`top=[DataApps, …]`，跟手且一次到位。
- 分区页勾最后一条：导出行就在它下面，且整行可见。
- 五条相关测试在手机上跑过（`dragging_onto_a_row_takes_its_place_not_the_one_below`、
  `the_home_open_switch_opens_and_folds_every_section`、
  `which_sections_are_open_is_kept_in_the_settings`、
  `picking_a_partition_brings_the_export_row_into_view`、
  `the_partition_list_runs_to_the_last_row_without_pages`）。

# 0.8.14 · 分区表一路滑到底、拖到最底落在末尾

真机上用电脑驱动手机点出来的（`sendevent` 注入 + 调试口截屏），不是只看编译过没过。

## 一、分区管理：整条长列表，一路滑到最底，没有翻页按钮

166 条分区一次画进画布要十几屏，画布最多六屏，于是老版本被截在屏幕口上：文字断、
卡片底是直角。中间试过「30 条一页 + 上一页/下一页」，真机上两处不行：按钮在列表尾
要滚到底才点得到；最后一页短了、滚动被夹，翻页像「跳到上面」。

改成和文件管理同一条**虚拟长列表**通道（`Frame.rows` + `pages::row`）：画布只画到
列表开头，下面的行滚进屏幕才画，画布不再被截，一路滑到底就是最后一条
（`xbl_sc_test_mode` 66KB），卡片底下是圆角 + 留白。翻页按钮全部删掉
（`Action::PartPage`、`part_page`、`part_slide` 一并清掉）。

两个只有真机才看得见的坑：

- **复用画布不擦会重影**：行是透明底直接叠在页面上的，回收来的画布比这一行高时
  留着上一行的像素，屏幕上就是两行字互相压。`Canvas::clear(0,0,0,0)` 一下解决。
- **行得自己铺卡片底**：`widgets::list` 是整块铺一次底的，虚拟行没有这层，不铺就
  只剩字浮在页面背景上。新增 `widgets::card_row(first, last)`：首行圆上边、末行
  圆下边，中间无缝，看上去还是一张卡。

## 二、排序页：拖到最底下能落在末尾

落点原来按「哪一行」算，拖到最末一行只能插到它前面，最后一条永远放不到底。改成
按**空档**算：指尖过了一行的中线就算落在它后面；`gap == 行数` 就是最底下，
`move_entry_to_end()` 把它塞进最后一栏的末尾。

真机验证（`sendevent` 长按 900 ms + 一路蹭底边让页面自己滚）：

- 长按「模块管理」→ 拖到最底 → 松手：`home_rest` 末位是 `Modules`
- 长按「充电设置」→ 拖到最顶 → 松手：`home_top` 首位是 `Charge`

## 三、调试口（开发用）

`/data/local/tmp/debug` 一出现，这一轮循环就存一张当前屏幕的 PNG 到
`/data/local/tmp/live.png`、把页面状态写进 kmsg、把这一屏可点区域写进
`/data/local/tmp/hits.txt`，然后删掉那个文件。`/data/local/tmp/frame-ms` 可以改
「一帧超过多少毫秒就记一条」的门限（默认 50，`turn` 也跟它）。recovery 里没有
framebuffer、`screencap` 是空壳，这是唯一能看见「此时此刻屏幕上是什么」的路子。

## 验证

- 手机上跑测试二进制：`dragging_past_the_last_row_puts_it_at_the_very_bottom`、
  `the_partition_list_runs_to_the_last_row_without_pages` 等四条全过。
- release 下量：排序页一帧「排版 1.93 ms + 合成 0.74 ms」——卡顿不在绘制。
- 手上真滑：分区页 20 多次滑动到 `scroll=26654`（顶到底），最后一屏见圆角。

# 0.8.3 · 跨栏拖动、长列表底部圆角、栏位滑动

## 一、一条能从「其他」拖进任何一栏

上一版的长按拖动只在同一列里换位置，`Drag::EntryDrag` 记的是「哪一列的第几格」。
现在记的是**整页第几行**：顶置 → 各栏（「其他」在它说的那个位置）→ 末尾，
`order_rows()` 按页面真实的绘制顺序给出这张表，手指落在第几行就把它挪到那一行
对应的那一栏 —— 所以「其他」里的「任务」能往上拖进「刷机备份」，也能往下拖进
「文件模块」。同一列里往下挪时先摘自己再插，落点不会差一格。

拖动时不再只取本列的命中框，而是整页所有 `HomePick` 行：第一行的顶、行距、
总行数三样就够，行高一致所以不必记每一行。

## 二、长列表的底边也有圆角

分区管理有四十几个分区，画布只有三屏（`long_page`），列表被截在屏幕口上：末尾
那张卡的圆角与后半截分区永远滚不到，看着就是硬切一刀。改成按条数要画布
（`long_page_rows(cx, count + 4)`，与 0.6.0 无线网络那次的修法一样），`fit` 仍
裁到真画到的位置——滚到底就能看见卡片底下的圆角与留白，跟无线网络页一样。

## 三、上移 / 下移有滑动的过场

`State.home_slide: Option<(usize, f32)>` 记「哪一栏、还要从多少行外滑回来」。
按上移/下移之后置成 ±3.2 行，`App::next_at` 每 16 ms 要一帧，`tick` 里每帧
`* 0.42` 收到 0.08 以下就清掉。画的时候只把那一栏的标题与条目卡画在偏移上，
排版仍按它该在的位置算，别的栏不跟着抖。跟系统设置里那个「菜单顺序」的手感
一致（2026-11-28 用户录屏给的参照）。

## 验证

- `cargo check --target aarch64-linux-android -p skyward-ui --tests -p skyward` 通过。
- 真机（popsicle，recovery_a）：刷入后分区 sha256 = `cd36171e…`，与出包一致；
  布局仍是四栏（0.8.2 那一份没有被覆盖）。

# 0.8.2 · 旧摆法升级：只比内容，不比先后

0.8.1 的 `home::upgrade()` 拿「其他」跟出厂顺序一字不差地比，真机上「其他」里
`usb` 被挪过一位，于是判定「用户动过」，四栏没换上——刷完还是老样子。

现在只比有没有、不比先后（排序后逐条比 `Entry::id()`）：条目集合一样就认作
出厂形状。「栏名对得上、刷机备份那三条对得上、置顶是空的、其余条目一条不差」
四条一起成立才动手，自己分过组、改过名、加删过条目的都不动。

真机验证（popsicle，recovery_a）：刷入后 `/metadata/skyward/settings.json` 里
`home_sections` 就是四栏（刷机备份 / 文件模块 / 其他分类 / 系统设置），
`home_top` 是 `["wlan"]`，「其他」剩 `tasks record mount filesystems shutdown`。

# 0.8.1 · 首页出厂四栏、长按拖动、布局备份能删

0.8.0 的首页是「一栏 + 其余全在『其他』」，用户看到的是拉平的一大条，跟图上的
分类状态不一样。这一版把出厂摆法改成四栏，排序页补上长按拖动，布局备份能删。

## 一、出厂就是四栏

`default_sections()` 从一栏（刷机与备份 3 条）改成四栏，无线网络单独摆最上面：

| 栏 | 条数 | 里面有 |
|---|---|---|
| 刷机备份 | 3 | 安装刷机、创建备份、恢复备份（摊开） |
| 文件模块 | 8 | 文件传输、文件管理、模块管理、应用管理、U 盘、修复 SELinux、无线传输、网络下载 |
| 其他分类 | 6 | 解密数据、模拟终端、分区管理、校验启动、Root 管理、系统更新 |
| 系统设置 | 8 | 第二系统、选项设置、重启系统、清除数据、版本更新、首页排序、导出日志、充电设置 |

本仓库比上游多出来的五条（任务、录屏、挂载、文件系统、关机）出厂留在「其他」。
`factory(l)` 一次给出「顶置 + 四栏 + 其他」，首页、排序页、第一次落盘三处用的是
同一份，不再各算各的。

**已经装过 0.8.0 的**：`home::upgrade()` 认得出上一版那种「一栏『刷机与备份』
+ 其余全在『其他』」的出厂形状，读设置时换成四栏；自己分过组、改过名的一律不动。

## 二、排序页：长按拖动、上移下移换了图标

- 按住任一条 400 ms（`LONG_PRESS_MS`）就拎起来，手指往上往下，它跟着落到第几格；
  每挪一格震一下，松手即存。数据在 `EntryDrag`：哪一条、在哪一列、起手第几格、
  那一列第一行的顶与行距——几何全取这一帧画出来的命中框，不另算一遍。
- 拎着时不滚页、不翻页；短按仍然是「拎起来 → 点别处放下」那套老办法。
- 笔右边那两枚原来是同一枚「排序」图标（三条横杠 + 下箭头），看不出谁上谁下。
  新增 `Icon::SortUp` / `Icon::SortDown`：三条横杠 + 上箭头是上移，+ 下箭头是下移。
  `Icon` 从 27 枚到 29 枚。

## 三、布局备份能删

「布局备份」页列出的每一份右边多一枚红色垃圾桶，点它删掉那个 `.json`，删完
重新列一遍；左边点名字仍然是「按这一份改」。行尾不再画箭头，那枚圆钮就是它的去处。
`Effect::DeleteLayout { path }` 走 `fs::remove_file`，kmsg 记一行，失败也照实报。

## 验证

- `cargo check --target aarch64-linux-android -p skyward-ui --tests -p skyward` 通过。
- 新增单测：`the_factory_layout_holds_every_entry_exactly_once`（三十一条一条不少、
  五条落在「其他」）、`the_old_single_section_layout_is_upgraded_once`（旧形状换四栏、
  再跑一次不动）。
- 真机前只剩出包与刷入。

# 0.8.0 · 首页自己编排、布局备份（移植自 skyward-pudding v0.5.13）

v0.5.13 六件事，逐条落在本仓库的树上。做法和上一版一样：**读上游怎么写的，按我们
自己的结构重写**，不是把上游的补丁打上来——我们的首页早就不是一张写死的行表了
（`Tasks` / `Record` / `Mount` / `Filesystems` / `Shutdown` 这几条上游没有），
`Frame.title` 我们还是 `Cow`，键盘那套也多一个 `chinese` 参数。

## 一、首页可以自建分类、任意排序，「其他」也能挪

新增 `crates/skyward-ui/src/app/home.rs`：

| 名字 | 是什么 |
|---|---|
| `Entry` | 27 个首页条目，`id` / `name` / `action` / `screen` / `icon` |
| `Section` | 一栏：`name` / `entries` / `icon` / `collapsed` |
| `default_sections` | 首次开机那四栏（系统与安装 / 数据与安全 / 外设与界面 / 关于） |
| `default_rest` | 「其他」里剩下的 |
| `normalize()` | 把「顶置 + 各栏 + 其他」过一遍，不重不漏 |
| `section_of` / `detach` | 找某条目在哪一栏 / 从原处摘掉 |
| `rest_is_default()` | 「其他」还是不是出厂那一套 |

`normalize()` 是这一版的核心：所有改动（挪进栏、挪出栏、删栏、改名、导入布局）
最后都过它一遍再落进 state。条目只会被移动，不会消失——「其他」被拖走、
某一栏被删掉，里面的条目回「其他」，不会跟着栏一起没。

上一版的 `Screen::Advanced` 整页拿掉了：它那 17 条现在是首页条目（`home::ADVANCED`），
页面本身、`Tile::Advanced`、搜索里那条 `Advanced` 全部删除。

## 二、分类、条目、「其他」、首页大字都能改名，搜索认新名字

- 条目：`Screen::EntryName`，写进 `state.entry_names`（`BTreeMap<id, 名字>`）。
- 分类 / 「其他」：`Screen::HomeSection`，写进 `Section::name` / `state.home_rest_name`。
- 首页大字：`Screen::HomeTitle`，写进 `state.home_title`，`Frame.title` 跟着换。
- 搜索：`search::find(query, l, names)` 把内置那 27 条先套一遍 `names` 再建索引，
  搜新名字能搜到，搜旧名字搜不到（除非新名字就是旧的）。`search::home_name()`
  给「这条在首页上叫什么」用。

## 三、布局备份：导出到任意目录，从任意 .json 导入

新增 `crates/skyward-ui/src/layout.rs`：

```json
{ "kind": "skyward-layout", "version": 1,
  "home_title": "…", "home_sections": [...], "home_top": [...],
  "home_rest": [...], "home_rest_name": "…", "home_rest_at": 2,
  "home_rest_collapsed": false, "entry_names": {...}, "tiles": [...] }
```

- 导出：`Effect::ExportLayout` → `/sdcard/Skyward/layout-<stamp>.json`，
  或者 `ExportLayoutTo{path}` 写到你挑的目录（`Screen::LayoutBrowse` 里挑）。
- 导入：`Screen::LayoutBrowse` 列 `layout-*.json`（`Effect::ReadLayouts`），
  点一行 → `Effect::ImportLayout{path}` → `Layout::apply(&mut state)`。
- `kind` 不对、`version` 不认识就当没读，不动现在的首页。

挑位置那一页复用的是文件管理那套 `Effect::ReadDir`，但页面读的是 `state.layout_dir`
而不是 `state.files`——所以 `App::set_files()` 现在按当前页分派：在
`Screen::LayoutBrowse` 上列出来的东西落进 `layout_dir`，别处还是落进 `files`。

## 四、文件传输点进独立页，开关状态说真话

`Screen::FileTransfer` 从设置页里独立出来（设置页那段删掉）。开关不再只是记住
「上次点了什么」，两个地方让它对上事实：

- `skyward/src/ui.rs` 的 `offer_storage()` 每次先把 `state.transfer_running`
  写成 `self.transfer.is_some()`，不一致就 `refresh()`。
- 页面上的「现在」按 `(transfer_running, data.storage_open())` 三种情况分别说实话：
  没开 / 开着但没连 / 连着。

`file_transfer()` 里原来的 `unreachable!` 改成纠偏：以实际跑着的服务为准，并
kmsg 记一行——开关被外部关掉时不再整页 panic。

## 五、磁贴：删「高级工具」，补新的，按开关与页面分两组

- `Tile::Advanced` 删除（它只有进那个已删页面一个用途）。
- 补 5 枚：`Decrypt` 解密数据、`SkywardUpdate` 版本更新、`Relabel` 文件标签、
  `HomeOrder` 首页排序、`Layout` 布局备份。`TILES` 30 → **34**。
- 上游这一步是「补八枚」，其中 `Serve` / `Avb` / `Update` 我们早就有了，
  `Charge` 本仓库没有充电那套，所以实际补 5 枚——功能对齐，枚数按我们的家底算。
  `Tile::left()` 现在是公开函数，「还剩下哪些能加」只在一处算。
- 「添加磁贴」面板按 `screen().is_none()`（开关与动作）/ `is_some()`（页面）
  分两段，每段 7 列，段与段之间多留一行标题的高度。编辑态的画布高度改由
  `candidate_height()` 现算，不跟着两段之间的空隙一起缩。

## 六、名字页点输入框闪退

根因在 `Action::TypeInto`：切字段时无条件 `self.refresh()`，而输入框的重绘路径
会回来再读一次 `adding`，切到一半的字段让那一页以为自己不在编辑态。现在只在
`a.field != field` 真的变了才刷新：

```rust
if let Some(a) = self.state.adding.as_mut()
    && a.field != field
{
    a.field = field;
    self.refresh();
}
```

新加的 `Screen::HomeSection` / `Screen::EntryName` / `Screen::HomeTitle` 三页
共用这套输入框，`Cap::Enter` 各自收口到 `HomeSectionDone` / `EntryNameDone` /
`HomeTitleDone`。

## 验证

- `cargo check --target aarch64-linux-android -p skyward-ui --tests -p skyward` 通过。
- 新增单测：`app/home.rs` 5 条（归一、摘取、出厂判定……）、`layout.rs` 3 条
  （往返、坏 JSON、版本不认）、`app/act.rs` 7 条（其他栏折叠、建栏删栏、
  拖动落点、名字页不闪退、首页大字、布局写读覆盖、挑位置页的目录）。
- 两版镜像重新出过，`sha256` 见 `output/SHA256SUMS`。

# 0.7.0 · Root 管理（移植自 skyward-pudding v0.5.9）

把 mariusarva/skyward-pudding v0.5.9 的 Root 管理搬到本仓库。两边的树同源
（都从 kirklin/skyward v0.5.0 分出来），所以是「新增一个 crate + 两个页面 + 接线」，
没有打上游补丁——我们的 WLAN、三语言、模块安装、AVB 都是自己的，直接 `patch` 会冲突。

## 入口与页面

- 高级页在「AVB 校验」下面多一行「Root 管理」；控制中心的磁贴候选里也多一枚
  `Root`（`TILES` 从 29 枚到 30 枚，默认摆法不变，它在「添加磁贴」里等着）。
- `Screen::Root`：KernelSU、Magisk、模块管理三行看现状；下面「在线安装」两张卡
  本身就是写入动作（滑动确认）。
- `Screen::KernelSu`：状态、在线最新（可升级 / 已是最新）、当前槽位、开机锁、
  版本、KMI 列表（默认按内核版本选中）、写入卡、恢复原厂。
- `Screen::KernelSuKmi`：只列 KMI，点一行就是选它，返回上一页。
- `Screen::Magisk`：状态、在线最新、目标分区（`init_boot` 还是 `boot`）、写入卡、
  恢复原厂。

## 素材全部在线取

`crates/skyward-root`（新增，零依赖）只有 KMI 表和取舍逻辑：

| KMI | Android | 内核 | 默认分区 |
|---|---|---|---|
| `android12-5.10` | 12 | 5.10 | `boot` |
| `android13-5.10` | 13 | 5.10 | `boot` |
| `android13-5.15` | 13 | 5.15 | `init_boot` |
| `android14-5.15` | 14 | 5.15 | `init_boot` |
| `android14-6.1` | 14 | 6.1 | `init_boot` |
| `android15-6.6` | 15 | 6.6 | `init_boot` |
| `android16-6.12` | 16 | 6.12 | `init_boot` |
| `android17-6.18` | 17 | 6.18 | `init_boot` |

写 Root 时才从 GitHub 取：`ksud-aarch64-linux-android`、`lkm-aarch64-<KMI>_kernelsu.ko`、
`KernelSU_*-release.apk`；Magisk 取 `Magisk-*.apk`（补丁器就在里面）。所以
ramdisk 体积不变——上游算过，8 个 KMI + ksud + 两个 APK 共 32.4 MiB，内置会把
ramdisk 从 53.7 顶到 81.1 MiB。

下载复用「在线更新本体」那一套：直连 + 8 个镜像前缀，`updates::by_speed` 并行测速后
按快慢依次试（为此把 `by_speed` 从私有改成 `pub(crate)`，`skyward-net::github::mirrors()`
新开出来）。每份文件 300 秒上限，整件事 420 秒上限（`root::Worker` 的 `DEADLINE`），
每 2 秒往内核日志写一条 `root: <name> N/M KiB`，日志里能看出走的是直连还是哪家镜像。

## 只升不降

`version_parts` 把 `v3.3.2`、`3.3.1-56-gcfc60f8d`、`27.0` 都拆成三段数字比较。
进页问一次 GitHub 最新版本，界面上写「可升级 / 已是最新」。两道闸：

1. 滑动确认时 `root_would_downgrade()` 拦一次，不新就只说一句「已是最新，不降级」；
2. 后端 `install_latest` 拿到 `tag_name` 后再比一次，不新就 `bail!`，一个字节都不写。

## 备份与写入

- 写入前先把原厂 boot 整份复制到 `/data/media/0/Skyward/root`，文件名叫
  `<stem>-<槽>-<target>-<时间戳>.img`；分区里已经带着 KernelSU/Magisk 的痕迹时，
  复用记录里那份**确认还是原厂**的备份（`stock=1` 且 sha256 对得上），不重复复制。
- KernelSU：把 `ksud` 与 `.ko` 写到 `/tmp/skyward-kernelsu`，跑
  `ksud boot-patch -b <分区> -m <ko> -o <目录> --out-name patched.img --kmi <KMI>`，
  检查产物是 boot 镜像、不超过分区大小，再 `image::flash` 写进当前槽。
- Magisk：从 APK 里解出 `magiskboot`、`magisk`、`magiskinit`、`init-ld`、
  `boot_patch.sh`、`util_functions.sh`，跑 `boot_patch.sh <分区>` 出 `new-boot.img`，
  同样检查后写入。
- 两边的管理器 APK 都落在内部存储 `Download/Skyward/KernelSU.apk` 或 `Magisk.apk`，
  **不调 `pm install`**，不会往桌面装 App。
- 记录写到 `Skyward/root/kernelsu.conf` / `magisk.conf`：`version`、`kmi`、
  `target`、`slot`、`backup`、`manager`、`sha256`、`stock`、`written`。
  「恢复原厂」就是读这份记录、核对 sha256、把备份 flash 回同一个分区，再记 `restored=1`。
- 分区选择 `target_partition`：先按 KMI 说的那个（`boot` / `init_boot`），
  再看这一版手机到底把 ramdisk 放在哪（`has_ramdisk` 读 boot 头），两处不一致时听手机的。

## 进度与取消

`root::Worker::bytes()` 给「已下 / 总共」，「写入」卡上写 `42% · 1.2 MiB/s`
（`write_state` 组装，速率由 `kernelsu_told` 每 0.5 秒算一次）。手指把卡滑回左端
（`unconfirmed`）就发 `Effect::CancelRoot`，`Worker::stop()` 置位，下载循环下一次读
就 `bail!("已取消")`；已经写出去的分区不回滚。失败时卡回到原位并写「写入未完成，
可以再滑一次」。

## 顺带补的三处底层

- `skyward_install::image::open()`：读 sparse 布局 + 展开成流 + `detect`，返回
  `Image { kind, layout }`。
- `skyward_install::image::ramdisk_holds()`：判 boot 镜像的 ramdisk 里有没有某个条目
  （KernelSU 看 `kernelsu.ko`，Magisk 看 `.backup/.magisk` 与 `overlay.d/sbin/magisk.xz`）。
- `skyward_net::github::mirrors()`：把原来私有的 8 个镜像前缀给别的下载路径共用。

## 没搬的部分

上游 v0.5.9 还有「刷机后自动修补新槽」（`AutoRoot` / `SetAutoRoot`，安装系统后、
重启前用已安装的 Root 修补新槽的 boot 镜像），它挂在安装流程里，与本次 Root 页面
不是一件事；本次只做用户列出的那一组（入口、两页、在线素材、备份、只升不降、
进度与取消）。上游另外合并的 Kinaxie 六项修复（服务复用、optional 服务、KeyMint
现场、解锁状态多处读、keep 双槽、快照合并门禁）本仓库此前已各自实现，这里未重复搬。

## 验证

- `cargo check --target aarch64-linux-android -p skyward --bin skyward`：通过。
- `cargo test -p skyward-root`：4 个测试全过（KMI 唯一、按内核与 Android 版本挑、
  分区跟着内核走、版本串解析）。
- `local_build.py --deliver`：八步全跑，`ALL CHECKS PASSED`，AVB 自洽。

### 真机（小米 17 Pro Max / popsicle，recovery_a）

出包 `sha256 8dd4d134…`，`dd` 写入后回读 md5 一致，重启 12 秒回 recovery。

| 项 | 结果 |
|---|---|
| 高级页「Root 管理」入口 | 在「AVB 校验」下一行，点进得去 |
| 控制中心磁贴候选 | 末行 `AVB / 无线传输 / 分区镜像 / Root`，30 枚齐 |
| Root 页 | KernelSU、Magisk、模块管理三行 + 两张在线写入卡 |
| KernelSU 页 | 已安装 `3.3.1-56-gcfc60f8d`、在线最新 `v3.3.1 · 已是最新`、槽位 A、已解锁 |
| KMI 列表 | 8 项齐，默认勾 `android16-6.12`（popsicle 内核 6.12，Android 17 回退到同内核最新一版） |
| Magisk 页 | 未安装、在线最新 `v31.1`、目标分区 `init_boot`、槽位 A |
| 只升不降 | 滑满卡 → toast「已是最新，不降级」，`dmesg` 无 `fetching`，`Skyward/root/` 未创建 |
| 互斥 | 装 Magisk 时 8 ms 内 `Failed("KernelSU is installed; restore its boot image before installing Magisk")` |
| 在线版本查询 | 开机 21 s 一次、进 Root 页再一次：`Some("v3.3.1") and Some("v31.1")` |

真机暴露并修掉两处：

1. Root 页第一行被版本串 `3.3.1-56-gcfc60f8d (uapi: 6) (32736)` 挤成 `K...`；
   `short_version` 只留第一段。
2. 没有 `kernelsu.conf` 时仍摆一行「已恢复」，点了必然失败（日志
   `no KernelSU backup record`）。`has_backup` 为假时改成一句说明，不给可点的行。

未验证（前置状态不满足）：真实下载进度与速率、滑回取消、备份/patch/写分区、
恢复原厂。这台机器的 KernelSU 由 KSU 管理器装入，没有本恢复模式的 `kernelsu.conf`；
要跑通整条写入链，需先有一台没有 Root 的机器，或手动把 `init_boot` 恢复原厂。

---

# 0.6.0

应用版本从并入布丁时的 `0.5.5` 升到 `0.6.0`（关于页、开机那行内核日志、`Cargo.lock`
一起上去）。这一版修的是无线网络页拉到最后时的底部。

## 无线网络

- **列表再长也能滚到底**。这一页把整个扫描结果一次画进画布，画布固定三屏：热点一多
  （几十个 2.4G/5G 同名网），列表尾、卡片底圆角和「添加网络」就落到画布外面，往下拉
  只能拉到一条被屏幕边缘切断的行，看着像「加载异常」。现在先数这一页要摆多少行，再按
  `content_top + 行数 × 行高` 要画布（至少三屏，最多六屏），`fit` 之后照旧按真画到的
  位置裁短。真机上 13 个热点时仍按三屏，底部圆角与「添加网络」照图二那样收得住。

---

# v3

开机到主页快 1.75 秒（popsicle 真机，7.22 s → 5.47 s，-24%）。五处改动，全部在
真机上逐项量过；解密页的进度条也同一版改了样子。

## 开机（4.09 s → 2.35 s）

- **ADSP 不再挡在解密前面**。`charge::raise_the_adsp()` 要等两段：先给正在读的固件
  0.7 s（`ADSP_GRACE`），再把没起来的那个叫起来，而叫它那一下要等固件从 `/firmware`
  读完——这台机上一共 1.36 s，以前全压在模块加载之后、页面前面。它只是充电页要用的
  （glink），和解密无关：现在分区挂好、ueventd 索引建好就起在另一条线上，和加载模块
  并行。真机上 ADSP 反而更早起（5.9 s → 4.78 s），micharge 仍在它之后启动。
- **定充电快慢的程序不再排在密钥链里**。micharge 与 batterysecret 读的是 ADSP 的
  glink，和解密不相干；它们在自己那条线上起（4.53 s 起、4.65 s 就绪），密钥链在
  `secure_element`/weaver 就绪时收尾（1.94 → 1.59 s）。
- **`goodix_core` 与 `nxp-nci` 挪到旁线**（配置里新增 `modules.beside`）。Goodix 的
  probe 在这台机（控制器是 Synaptics）上要重试到一秒上下才放弃，NXP 的 NFC 驱动起来
  还要读一遍芯片，恢复模式没有它们的页面：模块这一段 1.02 s → 0.63 s。触摸驱动
  `synaptics_tcm2` 仍留在前面等，面板（4.49 s）照旧先于页面可用。
  ——整份都挪到旁线试过（synaptics 也挪）：模块和密钥程序抢起来，qseecomd 自己那
  0.3 s 被拖长，页面反而晚了 0.26 s（5.48 → 5.73），所以只挪这两颗。
- **`kmsg` 不再每写一行都翻一遍 `/proc/mounts` 再开一次文件**。`keep_besides` 现在
  把第一个写得进去的文件（persist 上的 `skyward-boot.log`）留在手里，找一次用到底：
  模块加载那一批两百多行日志原本自己就要一秒。
- 真机时间表（旧 → 新）：modules 0.84 → 0.62 s、drivers 2.67 → 0.97 s、
  key programs 3.33 → 1.59 s、secure world ready 3.38 → 1.68 s、到主页 7.22 → 5.47 s。

## 解密

- **每一步都报耗时**（`skyward: opening the data partition: … took … s`）：页面只说
  「正在打开」，哪一步慢以前只能猜。这一趟整段 0.47 s：取分区密钥 0.042、dm 0.010、
  读文件系统名 0.024、挂载 0.045、共享密钥 0.055、各用户密钥 0.068、读屏锁 0.019、
  scrypt 0.015、安全世界核验 0.071、keystore2+GCM 0.021、放进 fscrypt 0.057，
  整段「用保存的密码开 user 0」0.167 s。
- **没有 made-up 用户时不再向安全世界要 note**。那份 note 只给「手机替它编了密码、
  藏在机主后面的用户」用；这一台没有，省掉 0.046 s。
- 共享密钥与各用户密钥**不能**并行：用户密钥所在的目录本身由共享密钥加密，先读它
  会 `ENOENT`（试过，解密当场失败）。
- 判断与顺序没改：开出的 `closed` 仍是 `[]`，`/data` 与 `/sdcard` 照旧可读。

## 解密页

- **进度条缩短到整行的 62% 并居中**，条头那个点由蓝改白（2026-11-25 用户要求）。
  高度、位置、百分比文字都没动。

---

# v2

真机全量点验之后的一批修正。用户直接报的三条都在这里（模块装不上、无线关掉再开不连、
AVB 页两张卡片贴死），全部在真机上验过。

## 模块安装

- **模块 zip 不再被当成刷机包**。文件管理里点一个既有 `module.prop` 又有
  `META-INF/com/google/android/update-binary` 的 zip（LSPosed 就是），以前会去跑那段
  update-binary，报「安装脚本运行失败，退出码为 1」。现在先看 zip 里有没有
  `module.prop`，有就按模块装：解进 `/data/adb/modules/<id>`，带 `customize.sh` 的
  照 Magisk 的方式跑一遍（`ui_print` / `abort` / `grep_prop` / `set_perm` 由这里给，
  脚本里那句 `unzip` 换成从已经解开的树上搬），装完页面报「模块已安装 / 下次启动时加载」。
- **XZ 压缩的条目能解了**。模块里的 `module.prop`、`customize.sh`、`lspd` 常用 XZ
  （zip method 95），toybox 的 `unzip` 只会 `compression method not supported: 95`。
  给 `zip` 打开 `xz` feature，整份包先由 recovery 解开。
- **`$MODPATH` 只放脚本自己 `extract` 的东西**。LSPosed 的 `customize.sh` 声明了
  `SKIPUNZIP=1`，Magisk / KernelSU 都只把它搬出来的文件放进模块目录；以前这里把整份
  zip 解进去，目录里多出 `lib/arm64-v8a/libframework.so`、`bin/arm64-v8a/dex2oat`、
  `machikado.arm` 这些原始路径副本，`lspd` 一启动就在 native 段 SIGSEGV（`code=139`）。
  现在拆成 stage（整份解开的 zip，给脚本里的 `unzip` 当源）与 `$MODPATH` 两个目录，
  装完的目录与 KernelSU 装的逐项一致。
- `set_perm` 补上第 5 个参数（SELinux context，LSPosed 给 `bin/dex2oat` 写的是
  `u:object_r:dex2oat_exec:s0`）。

## 无线

- **关掉再打开会自己连回来**。以前 `wlan_joined` 一旦置上，自动连接那一支就再也不进，
  radio 起来了却没人去连。现在按开关复位这一轮的状态，从名单第一张重新试。
- **首页照实说连不上的原因**：密码错、范围内没有这张、连接失败、拿不到 IP 各有各的
  说法，不再一律「正在连接」。
- 详情页里没扫到的网络，信号强度写「未知」，不写「极佳」。

## 界面

- **AVB 页**：「备份当前槽的 vbmeta」与下面那份备份列表之间补了一道卡片间距
  （`card_gap`），以前两张卡片上下贴死。
- **无线传输**：没连 WLAN 时不再显示 `http://<phone>:8080/` 这个字面占位符。
- **一个输入设备读失败不再让整块屏幕哑掉**：放掉那个设备、重扫 `/dev/input`、
  需要时补发按下，不牵连其它设备。

## 真机验过的

`recovery_a` 刷入后逐页点验；模块先用 KernelSU 卸载、再从 REC 重装，重启进系统
`lspd` 在跑、LSPosed 管理器显示「已激活 2.2.1 (7949)」、Xposed 模块在
`com.android.settings` 里完成 hook；WLAN 关 → 开 16 秒内连回 `F1_5G` 并拿到
`192.168.1.8`；AVB 页对齐；`logcat` 里 skyward 进程 0 条 E/F，`dmesg` 无
`oops/panic/BUG/watchdog/kasan`。

# v1

本仓库从零建立（旧的开发仓库已删除）。源码按当前功能整理过一遍：编译零警告、
没有 `allow(dead_code)` 兜着的死结构、没有一次性脚本，镜像功能与上一版的最终状态一致。

## 系统与安装

- **双系统**：主系统之外再装一套，启动时选；第二系统的第一个程序静态链接，不依赖任何库。
- **安装系统**：自写 payload applier —— 多线程解 xz / bz2 直写目标槽、不建 COW、
  并行读回校验、同会话补 root；支持 super 逻辑分区与 Android 稀疏镜像。
- **安装门槛**：写第一个字节之前把整包扫一遍，差量包、块大小不符、布局要变、
  分区装不下都在这里判掉；写不了的整包交给 `update_engine` 并说明原因。
- **备份与恢复**：按分区备份到内部存储或 U 盘，恢复时校验。
- **分区管理**：挂载、擦写、格式化，逻辑分区单独一页。
- **模块管理**：从 `vendor_boot` / `vendor_dlkm` / `vendor` / `system_dlkm` 取模块，按机型配置加载。
- **应用管理**：系统应用的停用 / 启用，停用的收在「已停用」页，点一下排 `enable` 回来。

## 数据与安全

- **解密 data**：原厂 ramdisk 缺的那层 HAL（`qseecomd`、keymint、gatekeeper、weaver、
  secure element、boot service）由 `resources/popsicle/recovery-hal.zip` 补进镜像，
  恢复出厂密码 / 图案 / PIN 都能解。
- **段位**：启动控制服务在 recovery 里起不来时，按当前运行的系统显示「A（当前）」。
- **充电设置**：三档（默认 / 快充 / 超快充）决定 recovery 启动哪些小米充电服务，
  另有电池温度上限。入口在「高级工具 → 充电」。
- **CPU 频率不动**：早先移植的 `adjust_cpu_governor.sh` 与整套调频代码已删除，
  内核的频率策略由原厂 `init.kernel.post_boot.sh` 决定。

## 外设与界面

- **触控**：双面板按分辨率打分选屏（主屏 1200×2608、副屏 976×596），后注册的更高分自动替换。
- **震动**：cs40l26 用 `FF_SINE` 的 BUZZGEN 发周期波，返回键与点击都有，强度可调。
- **录屏**：H.264 Baseline + MP4，分辨率 4 档 / 画质 3 档 / 帧率 4 档，定速出帧，
  可拖动的计时胶囊；`scripts/check_record.py` 不上手机就能验容器。
- **控制中心**：磁贴可拖拽排序、可移除，28 种任选，摆法存 `/metadata/skyward/tiles`。
- **文件管理**：「内部存储」旁一枚 `sdcard` 一步进 `/sdcard`；U 盘单独一页。
- **无线**：最多记 8 张网、最近优先，连不上换下一张、最多三轮；名单存 `/metadata/skyward/wlan`。
- **输入法**：中文 26 键打拼音（候选条一页 3 条），英文 26 键小写直落；
  密码、网址这类不收汉字的页始终小写。
- **首页设备卡**：名字下面一行四格 —— `当前槽位` / `数据分区` / `可用空间` / 电池。
  插着线时电池格带这一刻的功率（`100% / 充电中 · 7.9W`，充满也给），拔线只剩状态；
  点整卡或点读数都进设备详情，按住高亮用卡片自己的圆角。

## 出包

- 一条命令：`scripts/local_build.py` 跑完编译、取底包、补 HAL、打包、AVB、四项自检、算摘要。
- 两版品牌：品牌常量是编译期 `option_env!`，出官方版前先 `touch crates/skyward-ui/src/pages.rs`。
- CI：推 `main` 只编译自检；打 `v*` 标签出两版镜像、发 Release（两版 + `SHA256SUMS` + 版本说明）。
  运行记录的名写标签名 / 分支名，跑完把本 workflow 更早的运行删掉，列表只留一条。
- 可复现：同树同机重出逐字节相同。
