# Skyward for Xiaomi 17 Pro Max (popsicle)

小米 17 Pro Max 的第三方 recovery。Rust 写的，源码、设备配置、HAL 补丁层与构建脚本都在这个仓库里，自己编、自己出包，构建逐字节可复现。

| 项 | 值 |
|---|---|
| 机型 | Xiaomi 17 Pro Max（`popsicle`，双槽 A/B） |
| 底座 | 官方 `OS4.0.6.0.XPBCNXM` 的 recovery 分区 |
| 镜像 | `skyward-OS4.0.8.0-XiaoMi17ProMax.img` / `…-official.img`（只差品牌字） |
| 许可 | GPL-3.0（上游 Skyward 的许可声明原样保留） |

基线是 [kirklin/skyward](https://github.com/kirklin/skyward)，[mariusarva/skyward-pudding](https://github.com/mariusarva/skyward-pudding) v0.5.5 的功能已全部并入。从这里开始本仓库独立演进，不再同步上游。

## 能做什么

**系统与安装**

| 功能 | 说明 |
|---|---|
| 双系统 | 主系统之外再装一套，启动时选；第二系统的第一个程序静态链接，不依赖任何库 |
| 安装系统 | 自写 payload applier：多线程解 xz / bz2 直写目标槽、不建 COW、并行读回校验、同会话补 root；支持 super 逻辑分区与 Android 稀疏镜像 |
| 安装门槛 | 写第一个字节之前把整个包扫一遍，差量包、块大小不符、布局要变、分区装不下都在这里判掉，写不了的整包交给 `update_engine` 并说明原因 |
| 备份与恢复 | 按分区备份到内部存储或 U 盘，恢复时校验 |
| 分区管理 | 挂载、擦写、格式化，逻辑分区单独一页 |
| 模块管理 | 从 `vendor_boot` / `vendor_dlkm` / `vendor` / `system_dlkm` 取模块，按机型配置加载 |
| 应用管理 | 系统应用的停用 / 启用，停用的收在「已停用」页，点一下排 `enable` 回来 |

**数据与安全**

| 功能 | 说明 |
|---|---|
| 解密 data | 原厂 ramdisk 缺的那层 HAL（`qseecomd`、keymint、gatekeeper、weaver、secure element、boot service）由补丁层补进镜像，恢复出厂密码 / 图案 / PIN 都能解 |
| 段位 | 启动控制服务在 recovery 里起不来时，按当前运行的系统显示「A（当前）」，不再「无法读取」 |
| 充电设置 | 三档（默认 / 快充 / 超快充）决定 recovery 启动哪些小米充电服务，另有电池温度上限；入口在「高级工具 → 充电」 |

**外设与界面**

| 功能 | 说明 |
|---|---|
| 触控 | 双面板按分辨率打分选屏（主屏 1200×2608、副屏 976×596），后注册的更高分自动替换 |
| 震动 | cs40l26 用 `FF_SINE` 的 BUZZGEN 发周期波，返回键与点击都有，强度可调 |
| 录屏 | H.264 Baseline + MP4，分辨率 4 档 / 画质 3 档 / 帧率 4 档，定速出帧，可拖动的计时胶囊；`scripts/check_record.py` 不上手机就能验容器 |
| 控制中心 | 磁贴可拖拽排序、可移除，28 种任选，摆法存 `/metadata/skyward/tiles` |
| 文件管理 | 「内部存储」旁一枚 `sdcard` 一步进 `/sdcard`；U 盘单独一页 |
| 无线 | 最多记 8 张网、最近优先，连不上换下一张、最多三轮 |
| 输入法 | 中文 26 键打拼音（候选条一页 3 条），英文 26 键小写直落；密码、网址这类页不收汉字 |
| 首页设备卡 | 名字下面一行四格：`当前槽位` / `数据分区` / `可用空间` / 电池。插着线时电池格带这一刻的功率（`100% / 充电中 · 7.9W`，充满也给），拔线只剩状态 |

**出包**

| 功能 | 说明 |
|---|---|
| 一条命令 | `scripts/local_build.py` 跑完编译、取底包、补 HAL、打包、AVB、四项自检、算摘要 |
| 两版品牌 | 同一棵树出品牌版与官方版，品牌常量是编译期环境变量，其余字节一致 |
| CI | 推 `main` 只编译自检；打 `v*` 标签才出镜像、发 Release（两版 + `SHA256SUMS` + 版本说明） |
| 可复现 | 同树同机重出逐字节相同，两次 CI 出的镜像摘要一致 |

## 下载与刷入

到 [Releases](../../releases) 下最新的两个 `.img` 与 `SHA256SUMS`，校验后：

```sh
adb shell getprop ro.boot.slot_suffix       # _a 就写 recovery_a，_b 就写 recovery_b
adb reboot bootloader
fastboot flash recovery_a skyward-OS4.0.8.0-XiaoMi17ProMax.img
fastboot reboot recovery
```

- 需要已解锁 BL；只动 `recovery_<slot>`，`boot` / `init_boot` 不碰
- 出问题刷回原厂 recovery 即可恢复
- 不要用 recovery 里的「检查更新」：它会把 Skyward 官方镜像写回分区，本分支的定制会被覆盖

## 校验

```
镜像    skyward-OS4.0.8.0-XiaoMi17ProMax.img            104857600 字节
sha256  70d78c6bd32e62f99abab04b5a1bde02c909b2275cac796965378f4c028c8e19
md5     9dbaadd8178872da42d68bb8b3b279fa

镜像    skyward-OS4.0.8.0-XiaoMi17ProMax-official.img   104857600 字节
sha256  0641230eac812fde89399bab4832778ab0630203992bf9ae2e36f689bf9e7e21
md5     e813570637fee7b730c329878ba5fb7b
```

要 zip（给只吃 zip 的刷机工具）自己压一份，内容就是上面的 img：

```sh
zip -9 -j skyward-OS4.0.8.0-XiaoMi17ProMax.zip skyward-OS4.0.8.0-XiaoMi17ProMax.img
```

## 从源码构建

一条命令在本机跑完八步（编译、取原厂 recovery、补 HAL 层、打包、AVB、四项自检、算 sha256）：

```sh
rustup target add aarch64-linux-android        # Rust 1.100.0，版本由 rust-toolchain.toml 钉死
CARGO_TARGET_DIR=<目标目录> python scripts/local_build.py --deliver
```

- 需要：Rust 1.100.0 + `aarch64-linux-android` 标准库、Android NDK r29、mingw 的 `dlltool.exe`、`lz4`（Python 包）。细节见 [docs/BUILD.md](docs/BUILD.md)
- 产物：`out/skyward-OS4.0.8.0-XiaoMi17ProMax.img`（裸 100 MiB）；`--official` 出官方版
- 走 CI：推 `main` 只做编译自检；打 `v*` 标签才出镜像并发布 Release（[.github/workflows/build.yml](.github/workflows/build.yml)）

出官方版（品牌常量是编译期 `option_env!`，必须先 `touch` 让库重编）：

```sh
touch crates/skyward-ui/src/pages.rs
SKYWARD_BRAND_TITLE=Skyward SKYWARD_BRAND_WORDMARK=Skyward \
SKYWARD_BRAND_AUTHOR="© 2026 Skyward" \
CARGO_TARGET_DIR=<目标目录> python scripts/local_build.py --deliver --official
```

## 仓库结构

| 目录 | 内容 |
|---|---|
| `crates/` | 23 个 workspace 成员：`skyward`（recovery 主程序）、`skyward-ui`（界面与输入法）、`skyward-pack`（打包器）、`skyward-install`（applier）、`skyward-bootimg`、`skyward-config` 与十几个系统层 |
| `devices/` | `popsicle.toml`（生产用）+ `nezha.toml` / `ishtar.toml`（`skyward-config` 的解析回归夹具） |
| `resources/` | `fonts/`（MiSansVF、KirkMono、两种 emoji）、`popsicle/recovery-hal.zip`（原厂 ramdisk 缺的 HAL 层） |
| `third_party/` | `aosp-pinyin`（Apache-2.0，词典构建时生成并嵌进二进制） |
| `vendor/` | `qrcodegen`（局域网传文件的二维码） |
| `scripts/` | 出包与运维：`local_build.py`、`augment_stock.py`、`check_build.py`、`bump_stock.py`、`push.py` 等 |
| `docs/` | [BUILD.md](docs/BUILD.md) 构建与打包、[使用.md](docs/使用.md) 刷入与各项功能、[CHANGELOG.md](docs/CHANGELOG.md) 版本记录 |
| `.github/` | 一个 workflow：推 main 自检，打 tag 出两版镜像发 Release |

## 文档

| 想知道 | 看 |
|---|---|
| 怎么刷、怎么解密、录屏与充电怎么用 | [docs/使用.md](docs/使用.md) |
| 怎么编、怎么打包、AVB 怎么算、底座怎么升级 | [docs/BUILD.md](docs/BUILD.md) |
| 每一版改了什么 | [docs/CHANGELOG.md](docs/CHANGELOG.md) |
| 接手这个仓库要先知道的事 | [AGENTS.md](AGENTS.md) |

## 出处

- 本体：[kirklin/skyward](https://github.com/kirklin/skyward)（GPL-3.0）
- 功能补丁：[mariusarva/skyward-pudding](https://github.com/mariusarva/skyward-pudding)
- 早期镜像与解密记录：[mariusarva/skyward-popsicle](https://github.com/mariusarva/skyward-popsicle)

许可证：[GPL-3.0](LICENSE)。
