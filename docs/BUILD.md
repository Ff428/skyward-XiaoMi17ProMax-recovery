# 构建与打包

只想要一个能刷的包：`python scripts/local_build.py` 一条命令跑完（见「本机出包」）。
本文其余部分是环境、打包格式、AVB、底座升级与这台机器的定制点。

## 需要什么

| 组件 | 版本 / 说明 |
|---|---|
| Rust | 1.100.0（由 `rust-toolchain.toml` 钉死），带 `aarch64-linux-android` 标准库 |
| Android NDK | r29（r26 以上均可），要 `toolchains/llvm/prebuilt/<host>/bin/clang` |
| Python 3 | 出包脚本用，需要 `lz4` 模块 |
| MinGW（仅 Windows 主机） | `dlltool.exe` 与 gcc；过程宏与 C 代码要用 |
| `lz4.exe` | 打包器压 ramdisk 用 |

```sh
rustup target add aarch64-linux-android
```

`rustup` 下载卡住时换镜像：

```sh
export RUSTUP_DIST_SERVER=https://rsproxy.cn
export RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup
```

## 编译

```sh
export ANDROID_NDK_HOME=/path/to/android-ndk-r29
./scripts/build-android.sh          # 等价于下面这一串环境变量 + cargo build
```

手工命令：

```sh
TC=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin   # Windows: windows-x86_64
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$TC/clang
export CC_aarch64_linux_android=$TC/clang
export AR_aarch64_linux_android=$TC/llvm-ar
export CXX_aarch64_linux_android=$TC/clang++
export CFLAGS_aarch64_linux_android="--target=aarch64-linux-android30"
export CXXFLAGS_aarch64_linux_android="--target=aarch64-linux-android30"

cargo build --release -p skyward --target aarch64-linux-android
```

产物 `target/aarch64-linux-android/release/skyward`（约 8.6 MiB，未 strip）。
`.cargo/config.toml` 里写了 `link-arg=--target=aarch64-linux-android30` 与国内 crates
镜像；换网络环境时把那几行 `[source.crates-io]` 删掉。

只做类型检查：

```sh
cargo check -p skyward-ui
```

## 本机出包

```sh
python scripts/local_build.py
```

八步：

| 步 | 干什么 |
|---|---|
| 1 | `cargo build -p skyward --target aarch64-linux-android` |
| 2 | `cargo build -p skyward-system-init`（`+crt-static`，第二系统的第一个程序） |
| 3 | `cargo build -p skyward-pack`（跑在电脑上的打包器） |
| 4 | `skyward-pack stock`：从官方全量包里取出 recovery 分区 |
| 5 | `augment_stock.py`：把 HAL 补丁层注入原厂 ramdisk |
| 6 | `skyward-pack recovery`：换二进制、写配置、加字体、打包、重算 AVB |
| 7 | `check_build.py`：条目数、关键文件、AVB 自洽、摘要四项门槛 |
| 8 | 算 SHA-256（`--zip` 才压 zip） |

常用参数：

```sh
python scripts/local_build.py --deliver         # 出完放到交付目录，只留当前这一版
python scripts/local_build.py --official        # 出官方版（品牌常量换掉）
python scripts/local_build.py --skip-build      # 二进制没变，只重打包
python scripts/local_build.py --out-dir D:/out  # 产物放别处
python scripts/local_build.py --zip             # 额外压一份 zip
python scripts/local_build.py --keep-old        # 不归档旧包
```

- 产物名固定 `skyward-OS4.0.8.0-XiaoMi17ProMax.img`（`--official` 加 `-official`）；版本号只进 tag 与 Release 标题。
- `stock/recovery.img` 有缓存：摘要与 `devices/popsicle.toml` 的 `[stock] recovery` 对得上就跳过下载。
- 同树同机重出逐字节相同。本机与 CI 的 ramdisk 压缩数据长度会差一点（`lz4.exe` 1.10 与 ubuntu 的 1.9.4 压缩参数不同），但条目、权限、AVB 与分区大小一致。

### 怎么出得快

增量约 2 分钟，全量十几分钟，差别在 `target/` 有没有留住：

| 做法 | 为什么快 |
|---|---|
| 固定一个 `CARGO_TARGET_DIR`，不要清 | 只重编改过的 crate 与链接 |
| 只换品牌时 `touch crates/skyward-ui/src/pages.rs` | 品牌串是 `option_env!`，只重编 `skyward-ui` 与最终链接 |
| 二进制没变时用 `--skip-build` | 跳过第 1–3 步 |

会让增量白攒的动作：换 `CARGO_TARGET_DIR`、清 `target/`、改 `Cargo.toml` 的 profile
或 `RUSTFLAGS`、改 `[workspace.package] version`。

## CI 发布

推 `main` 只编译自检（产物随 job 走完即丢）；打 `v*` 标签才出镜像并发 Release。

| job | 干什么 |
|---|---|
| `tools` | 编 `skyward`、静态的 `system-init`、`skyward-pack` |
| `tests` | 只跑 `skyward-crypto` / `skyward-config` / `skyward-modules`，不挡发布 |
| `recovery` | matrix 两版：品牌版用默认常量，官方版带 `SKYWARD_BRAND_*` 重编一次；取底包 → 补 HAL → 打包 → 自检 → 上传 |
| `release` | 一个 tag 一个 Release：两个 img + `SHA256SUMS` + 版本说明（`docs/CHANGELOG.md` 拷成 `<tag>-release-notes.md`） |

`skyward-ui` 的测试不在 CI 跑：时区断言按 +8 写、图标与字形量像素、vendored 拼音在
debug 下会 glibc `double free`。本机跑法：

```sh
cargo test --locked -p skyward-ui --lib
```

## 打包原理

popsicle 的 recovery 分区是 **Android boot image v4**（`ANDROID!` 魔数，kernel 为空，
只有 ramdisk），ramdisk 用 **LZ4 legacy 帧**（magic `0x184C2102`，每块 8 MiB），
整分区固定 **100 MiB（104857600 B）**，末尾带 **AVB vbmeta + footer**。

出包不改 kernel，只换 ramdisk 里的条目、追加固件，然后重算压缩与 AVB。

### 为什么是「骨架替换」

ramdisk 里条目的顺序、inode、4 字节对齐都参与校验与挂载顺序。整目录重新打 cpio 会把
顺序打乱，个别机型直接起不来。`repack_image.py` 逐条复制原始 cpio，只替换 `MANAGED`
里列出的条目，其余原样保留（含 `TRAILER!!!` 之后的填充）。

手工复现：

```sh
python scripts/unpack_image.py --ref stock.img --out work
cp target/aarch64-linux-android/release/skyward work/ramdisk/system/bin/recovery
python scripts/repack_image.py --ref stock.img --cpio work/ramdisk.cpio \
    --tree work/ramdisk --fw work/fw --out output/skyward.img
python scripts/patch_avb.py --ref stock.img --img output/skyward.img
python scripts/verify_image.py --ref stock.img --img output/skyward.img \
    --bin target/aarch64-linux-android/release/skyward
```

`verify_image.py` 通过才打印 `ALL CHECKS PASSED`，它检查：

1. boot v4 头与 LZ4 legacy 帧能完整解压、cpio 可解析；
2. 与参考镜像相比，改动只能是 `skyward/device.toml`、`system/bin/recovery`，新增只能是 `lib/*`、`skyward/*`，不允许条目缺失；
3. 镜像里的 `system/bin/recovery` 与刚编出的二进制逐字节一致；
4. vbmeta 偏移 / 摘要、footer 偏移、分区大小自洽。

### AVB 怎么补

原厂镜像在 `align4096(4096 + ramdisk_size)` 处放 512 字节 vbmeta，分区最后 64 字节放
footer。改了 ramdisk 大小后：

- vbmeta 整体后移，`vbmeta_offset`（偏移 272，大端 u64）与新位置的镜像前缀 sha256（偏移 396，32 字节）一起重写；
- footer 里两处 `vbmeta_offset`（12 / 20）与 vbmeta 大小（28）重写。

`patch_avb.py` 从 `--ref` 抄模板再改这三个字段，写完自检。bootloader 严格校验时仍需
`fastboot flashing unlock` / 关闭 verification，这与原厂镜像一样。

### HAL 补丁层

官方 recovery 的 ramdisk 里**没有** `sbin/` 那套 HAL：`qseecomd`、
`android.hardware.security.onekeymint-service-qti`、`gatekeeper-rust-service-qti`、
`android.hardware.weaver-service.nxp-qti`、`secure_element-service.qti`、`se_omapi`、
`android.hardware.boot-service.qti`、`hlosminkdaemon`，也没有配套的 `system/lib64` 库与
`system/etc/vintf/manifest/*.xml`。缺了它们解不开 `/data`；CS40L26 的 DSP 固件也不在
里面，马达起不来。

```sh
python scripts/augment_stock.py --stock stock/recovery.img \
    --hal resources/popsicle/recovery-hal.zip --out stock/base.img
python scripts/augment_stock.py --stock stock/recovery.img \
    --hal resources/popsicle/recovery-hal.zip --list      # 看它包含什么
```

它只做「补」：解出原厂 cpio，把补丁层里原厂没有的条目（外加 `skyward/resonance`，内容
`1724\n`）按原顺序追加，重新压成 LZ4 legacy，分区尺寸与尾部 AVB 页脚保持原样。

补丁层取自本机 4.0.6.0 的 TWRP 容器，共 226 个文件 / 约 41 MiB，压缩后 13.9 MiB。

只有两种情况要重新抽：新固件换了 vendor 侧 HAL 接口（解密时 `weaver` / `keymint`
起不来），或者换了机型 / 容器。

## 底座升级

镜像拿**官方全量包里的 recovery 分区**当底。官方每发一版系统，内核、init、SELinux
策略、adbd、一堆 so 都跟着换；基线不跟会出现触屏或显示异常、WiFi 起不来、AVB 校验
不过。

先对一下版本：

```sh
adb shell getprop ro.mi.os.version.incremental
grep -A4 '^\[stock\]' devices/popsicle.toml
```

两个一样就不用动。不一样时一条命令：

```sh
python scripts/bump_stock.py --version OS4.0.7.0.XPBCNXM
```

它按顺序做完：

1. 查这个版本的全量包文件名，拼出官方下载地址；
2. 改 `devices/popsicle.toml` 的 `[stock]`，把 `recovery` 摘要清空；
3. 提交并推送，等 CI 跑完；
4. 从 CI 日志里抓出新包的 recovery 分区 SHA-256，回填 `recovery`；
5. 再提交推送一次——从此每次构建都做摘要强校验。

已知摘要时加 `--sha256 <64 位>`；只想改文件不推送加 `--dry-run`。

手工做法：

1. 打开 <https://xmfirmwareupdater.com/hyperos/popsicle/>，找目标版本，**Type 必须是 `Recovery`**（`Fastboot` 包里没有 `payload.bin`），记下包名；
2. 地址规律是「服务器 + 版本号目录 + 包名」，先 `curl -sI` 确认 200；
3. 改 `[stock]` 的 `version` / `package`，`recovery = ""`（空着不校验，只打印实际值）；
4. 推送，CI 日志里会打出 `has the SHA-256 …, which the configuration does not name yet`，把它填进 `recovery` 再推一次。

| 坑 | 说明 |
|---|---|
| 拿成 Fastboot 包 | 只有 Recovery 类型的 zip 里有 `payload.bin` |
| 版本串不完整 | `XPBCNXM` 后缀不能省 |
| 忘了回填摘要 | 不影响出包，但失去「包被换过」的保护 |
| 以为要重下整包 | CI 按版本号缓存 recovery 分区，同版本第二次起秒过 |

## 这台机器的定制点

跨版本改代码时，下面几处不能丢；丢了的表现都写在表里。

| # | 位置 | 做什么 | 丢了会怎样 |
|---|---|---|---|
| 1 | `crates/skyward/src/haptics.rs` | cs40l26 用 `FF_SINE = 0x5a` 发周期波（BUZZGEN），不是 `FF_CUSTOM` | 震动毫无反应或极弱 |
| 2 | `crates/skyward/src/input.rs` `scan()` | 双面板按分辨率打分选屏，主屏 1200×2608 完全匹配给 `1<<60` 权重，后注册的更高分自动替换 | 抓到副屏（976×596），主屏点不动 |
| 3 | `crates/skyward/src/ui.rs` | 马达打开失败不致命，`take_up_motor()` 每帧最多一秒试一次，两分钟为限 | 模块加载早期 UI 线程崩掉，触控模块 probe 不完整，主屏触控消失 |
| 4 | `crates/skyward-ui/src/app.rs` | 每一下真的触发动作的点按都回一次 `Effect::Haptic`；默认强度 0.7 | 点按钮没有反馈 |
| 5 | `crates/skyward-ui/src/pages.rs` | 品牌三串走 `option_env!`：`SKYWARD_BRAND_TITLE` / `WORDMARK` / `AUTHOR` | 两版品牌出不来 |

**不要改 `LICENSE_ORIGINAL`**：GPL 许可页里 `Skyward / Copyright (C) 2026 Kirk Lin`
是许可声明，必须原样保留。

### 录屏：三条不能动的参数

`crates/skyward-ui/src/record.rs` 与 `native/`：

- `iMultipleThreadIdc = 1`，且只有一个编码器实例（P 帧引同一实例的上一帧，多实例混写会让 ffmpeg 报 `reference picture missing during reorder`）；
- `iEntropyCodingModeFlag = 0`（Baseline 只有 CAVLC）；
- `uiIntraPeriod = fps`（每秒一个 IDR）。

`libopenh264.a` 是提交进仓库的静态库（87 个对象，openh264 2.4.1，纯 C++）。重编方式见
`crates/skyward-ui/native/README.md`。`build.rs` 只在 aarch64 target 下跑，且只在 shim
或 `.a` 的 mtime 变了才重跑。

定速出帧：`push` 只把画布放进「最新一帧」那一格（`Mutex<Option<Job>>` + `Condvar`），
编码线程自己按 `1/fps` 起表，那一格没有新画面就重编手里的 `Yuv420`。`Recorder::push`
里先拿 `spare()` 再锁 `mail`，编码线程先取 `mail` 再锁 `pool`，顺序反过来就是死锁。

## 常见问题

| 报错 / 现象 | 原因与处理 |
|---|---|
| `can't find crate for 'std'` / `E0463` | 没装 Android 标准库：`rustup target add aarch64-linux-android`；确认没被 `rust-toolchain.toml` 切到别的 toolchain |
| `error: linker 'clang' not found` | `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` 没设或路径不对 |
| `gcc not found`（Windows 主机） | 把 MinGW 的 `bin` 加进 `PATH` |
| `error calling dlltool 'dlltool.exe'` | 同上，或 `source scripts/dev-env.sh` |
| `os error 3 系统找不到指定的路径` | 给 cc 传了 `/d/...` 形式，`CC_*` 要写 `D:\...` |
| 编译成功但文件被锁 / `os error 5` | 杀毒软件在扫新二进制，重跑一次；把 `target/` 加白名单 |
| `lld: error: unknown argument: --eh-frame-hdr` | `RUSTFLAGS` 顶掉了 `--target`，把 `-C link-arg=--target=aarch64-linux-android30` 一起写进去 |
| `需要 lz4 模块：pip install lz4` | 用错 Python，要用带 `lz4` 的那个 venv |
| `run lz4 (install it, e.g. brew install lz4)` | `lz4.exe` 不在 `PATH` |
| 取底包卡住不动 | 官方包在境外 OSS，设 `SKYWARD_PROXY` |
| 取出来的分区摘要不对 | 包被换过或下了半截，删 `stock/recovery.img` 重来 |
| 镜像 63 MB，怀疑错了 | 那是 zip，解开是 100 MiB |
| 能开机但解不开 `/data` | 补丁层没进镜像，确认 `augment_stock.py` 跑过 |

## 版本号

`crates/skyward-info/build.rs` 把 git 描述写进二进制，关于页的版本行来自这里；
workspace 版本在 `Cargo.toml` 的 `[workspace.package] version`（当前 `0.6.0`，2026-11-25
起；`0.5.5` 是并入布丁功能时的基线）。改它要跟着跑一次
`cargo update --offline --workspace`，`Cargo.lock` 里 25 个成员包的版本才会一起上去，
否则 CI 的 `--locked` 会直接红。对外发布另有一个 tag（`v0.6.0`），两处一起改。
