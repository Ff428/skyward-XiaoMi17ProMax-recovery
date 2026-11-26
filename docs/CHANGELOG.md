# 版本记录

镜像名固定：`skyward-OS4.0.8.0-XiaoMi17ProMax.img`（品牌版，首页与关于页写「卑鄙坑Baby」）
与 `skyward-OS4.0.8.0-XiaoMi17ProMax-official.img`（官方版，写 Skyward），两版只差品牌字，
其余字节一致。文件名里的 `4.0.8.0` 是交付代号，底座是官方 `OS4.0.6.0.XPBCNXM`。

出包见 [BUILD.md](BUILD.md)，用法见 [使用.md](使用.md)，环境与坑见 [../AGENTS.md](../AGENTS.md)。
CI 发布的 `<tag>-release-notes.md` 就是这份文件。

---

# 未发布 · Root 管理（移植自 skyward-pudding v0.5.9）

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
