# 给接手的人

小米 17 Pro Max（代号 `popsicle`，系统 `OS4.0.8.0.XPBCNXM`）的第三方 recovery，Rust 写的。
上游 [kirklin/skyward](https://github.com/kirklin/skyward)，功能补丁来自
[mariusarva/skyward-pudding](https://github.com/mariusarva/skyward-pudding)。

**本仓库是 `Ff428/skyward-XiaoMi17ProMax-recovery`（公开），不是 fork**：源码按当前功能重建过一遍，
历史只有一个初始提交，外部 tag 不是 HEAD 的祖先。要借别人的东西按内容抄，**不要 `git merge`**。

2026-11-24 起不再跟上游：布丁的功能已全部搬完，只在这里往前走。
旧开发仓库（`Ff428/skyward-popsicle`）已删除，本仓库接手它的全部功能。

---

## 目录地图

| 位置 | 干什么 |
|---|---|
| `crates/skyward` | recovery 主程序（开机进的就是它）：页面调度、worker、kmsg、触控与马达 |
| `crates/skyward-ui` | 界面、输入法（26 键拼音 + 候选条）、主题、屏绘、录屏 |
| `crates/skyward-pack` | 打包器（主机上跑）：读原厂镜像、注入 ramdisk、重算 AVB |
| `crates/skyward-install` | applier（把 zip / img 写进分区） |
| `crates/skyward-bootimg` | boot image v4 / LZ4 legacy ramdisk 的读写 |
| `crates/skyward-config` | `devices/*.toml` 的解析与校验 |
| `crates/skyward-programs` | 生成 `/data/system/…` 的一套描述（服务、账户、权限）；`tests/described/popsicle/` 是原厂分区里抽出的夹具 |
| `crates/skyward-block` `-lp` `-vab` `-omapi` `-mtp` `-net` `-crypto` `-files` `-info` `-backup` `-modules` `-systems` `-sys` | 各自的系统层 |
| `crates/skyward-system-init` | 第二系统的第一个程序，静态链接 |
| `crates/skyward-preview` | 开发用：把一页画成 PNG（只在 Linux 上编译，不进产品） |
| `devices/` | `popsicle.toml` 生产用；`nezha.toml` / `ishtar.toml` 是 `skyward-config` 与 `skyward-preview` 的夹具 |
| `resources/` | `fonts/`、`popsicle/recovery-hal.zip`（原厂 ramdisk 缺的 HAL 层） |
| `third_party/aosp-pinyin` | AOSP 拼音引擎（Apache-2.0），词典构建时生成并嵌进二进制 |
| `vendor/qrcodegen` | 二维码（局域网传文件用） |
| `scripts/` | 出包与运维，见下表 |
| `docs/` | `BUILD.md` 构建与打包、`使用.md` 刷入与各项功能、`CHANGELOG.md` 版本记录 |

`scripts/`：

| 脚本 | 干什么 |
|---|---|
| `local_build.py` | 本机一条命令出包（八步全跑，含自检） |
| `augment_stock.py` | 把 HAL 补丁层注入原厂 ramdisk（缺它解不开 `/data`） |
| `check_build.py` | 成品四项门槛：条目数、关键文件、AVB 自洽、摘要 |
| `repack_image.py` `patch_avb.py` `verify_image.py` `unpack_image.py` | 手工拆包 / 重打包 / AVB 重算与校验 |
| `bump_stock.py` | 换官方底座版本（改基线、跑 CI、回填摘要） |
| `push.py` | 推仓库（自带代理探测与令牌查找） |
| `gh_device_flow.py` | 设备码授权，拿令牌 |
| `check_record.py` | 录屏 mp4 的容器结构检查（不解码画面） |
| `build-android.sh` `dev-env.sh` | Linux/macOS 上的编译入口与环境变量 |

---

## 环境与凭据

编译在 GitHub Actions 上也能跑，**新电脑只要能 clone、能推、能跑 python 就能出包**。
下表只在「想在本机编译验证」时需要配齐。

| 组件 | 版本 | 本机实测路径 |
|---|---|---|
| Python | 3.13+ | `C:\Users\F\.workbuddy\binaries\python\versions\3.13.12\python.exe` |
| git | 2.55+ | `D:\软件\应用\PortableGit\mingw64\bin\git.exe` |
| Rust | **1.100.0** | `C:\Users\F\.rustup\toolchains\1.100.0-x86_64-pc-windows-gnu` |
| Rust target | `aarch64-linux-android` | 同上 `lib/rustlib/` |
| mingw64 | 任意 | `D:\软件\应用\mingw64\mingw64\bin`（只为 `dlltool.exe` / gcc） |
| Android NDK | r29 | `D:\WorkBuddyAI\downloads\ndk_r29\android-ndk-r29\toolchains\llvm\prebuilt\windows-x86_64\bin` |
| lz4（Python） | 任意 | `pip install --target D:\WorkBuddyAI\cache\pylibs lz4` |

Linux/macOS：`rustup target add aarch64-linux-android` + 一个 NDK，然后 `scripts/build-android.sh`。

**凭据（唯一不能进仓库的东西）**：令牌故意不进版本库。换机器走一次设备码授权，约 2 分钟：

```bash
python scripts/gh_device_flow.py     # 生成八位码，浏览器打开 github.com/login/device
```

- scope 要 **`repo` + `workflow`**（没有 `workflow` 就推不了 `.github/workflows/`）
- 令牌落点：`D:\WorkBuddyAI\github\.token`，该目录不在时落 `~/.config/skyward/token`
- `push.py` 按 `--token-file` → `GITHUB_TOKEN` / `GH_TOKEN` → 上面两个路径的顺序找
- 代理自动探测（本机 `127.0.0.1:7890` 通就走），`--no-proxy` 强制直连

---

## 出包与发布

### 本机（增量约 2 分钟）

```powershell
$env:CARGO_TARGET_DIR="D:/WorkBuddyAI/tmp/target-v44"
python scripts/local_build.py --deliver

# 官方版：品牌常量是编译期 option_env!，必须先 touch 让库重编
(Get-Item crates\skyward-ui\src\pages.rs).LastWriteTime = Get-Date
$env:SKYWARD_BRAND_TITLE="Skyward"; $env:SKYWARD_BRAND_AUTHOR="© 2026 Skyward"; $env:SKYWARD_BRAND_WORDMARK="Skyward"
python scripts/local_build.py --deliver --official
```

- 产物名固定：`skyward-OS4.0.8.0-XiaoMi17ProMax.img` / `…-official.img`；版本号只进 tag 与 Release 标题。
- `--deliver` 拷到 `D:\WorkBuddyAI\2026-11-13-20-45-53\output\`，只留当前这一对，更旧的进 `output/archive/`。出官方版会把品牌版挪进 archive，记得拷回来并重写 `SHA256SUMS`。
- **不设 `RUSTUP_HOME`**。同树同机逐字节相同；zip 不可复现（内含时间戳）。

### CI（发布用）

推 `main` → 只编译自检。打 `v*` 标签 → `build.yml` 的 matrix 出**两版**，release job 建
**一个** Release（两个 img + `SHA256SUMS` + `<tag>-release-notes.md`，说明来自 `docs/CHANGELOG.md`）。

```powershell
python scripts/push.py                                    # 推 main
# 建 tag 走 API 更稳（避开 github.com:443 抖动）：POST /git/refs {ref: refs/tags/vN, sha: <main HEAD>}
```

- 用 GitHub API 的 python 脚本必须带 `dangerouslyDisableSandbox`（沙箱里 TLS 报 `UNEXPECTED_EOF`）；
  Windows 上的 python 不认 `/d/` 路径，脚本里写 `D:/`。
- 删 tag 用 `git/refs/tags/<name>`（不是 `git/refs/refs/...`）。

---

## 改之前先确认

1. **别 merge 任何外部历史。** 按内容抄。
2. **别手工拼镜像。** 出包只有两条路：`local_build.py` 或 Actions。成品必须过
   `scripts/check_build.py` 四项门槛、ramdisk **1282 条**。
3. **品牌可以改，许可不能动。** 大字 / 署名在 `crates/skyward-ui/src/pages.rs` 的
   `HOME_TITLE` / `WORDMARK` / `AUTHOR`；`LICENSE_ORIGINAL` 里 Skyward / Copyright (C) 2026
   Kirk Lin 必须原样保留（GPL）。
4. **`devices/popsicle.toml` 用本 fork 的版本**：`[data] weaver_first`、`[backup]`、
   9 个 `[[service]]`、cs40l26 的 `[haptics] resonance`。
5. **原厂 ramdisk 缺 HAL 层**，打包前必须过 `augment_stock.py`，否则能开机但解不开 `/data`、马达不起。
6. **系统应用的停用/启用靠开机脚本**（recovery 改不了包管理器状态）：队列在
   `/data/adb/skyward/pm-queue`，脚本在 `/data/adb/service.d/`（+ `post-fs-data.d`、
   `boot-completed.d`），结果写 `pm-state` / `pm.log`。脚本里 `pm` 的 stdin 必须指
   `/dev/null`，否则 binder 回 `Failed transaction`。
7. **CPU 频率那套已经整条删掉**（`skyward-sys/src/cpu.rs` 不存在了）。recovery 不读也不写
   `scaling_*`，别加回来。

---

## 踩过的坑

| 现象 | 原因 | 处理 |
|---|---|---|
| `error calling dlltool 'dlltool.exe'` | mingw 不在 PATH | `source scripts/dev-env.sh` |
| `os error 3 找不到指定的路径` | 给 cc 传了 `/d/...` | `CC_*` 写 `D:\...` |
| `cargo check` 报找不到 rustc | 走了 rustup 代理 | 用工具链里的绝对路径 `cargo.exe` |
| 推到 GitHub 超时 | 沙箱连不上 / 没设代理 | 用 `scripts/push.py` |
| 推 `.github/workflows/` 被拒 | 令牌没有 `workflow` scope | 重新授权，scope 写 `repo,workflow` |
| 镜像 63 MB，怀疑错了 | 那是 zip，解开是 100 MiB | 别按 zip 体积判断 |
| 能开机但解不开 `/data` | 原厂 ramdisk 缺 HAL 层 | 确认 `augment_stock.py` 跑过 |
| `cargo test` 在 Windows 上编不过 | `skyward-bootimg` / `skyward-preview` 用 `std::os::unix` | 单元测试只在 CI 与 Linux 上跑 |
| CI 的 `skyward-ui` 测试红 | 时区断言按 +8 写、量像素字形、vendored 拼音 debug 下 `double free` | CI 只跑 `crypto/config/modules`；`skyward-ui` 在本机跑 |
| `pm list packages -d` 看不到已停用的 | 它不列 `enabled=3` 的包 | 看 `dumpsys package <包> \| grep enabled=`，3 = 停用 |
| 应用停用后出现在 KernelSU 模块列表 | 早期版本把脚本装成模块 | 已改成脚本位，并回收旧模块目录 |

---

## 仓库里不放什么

成品镜像（100 MiB/个）走 GitHub Releases，旧版本不留档；`stock/`、`out/`、`target/`、
令牌、keystore 一律在 `.gitignore` 里。版本记录在 `docs/CHANGELOG.md`，使用与构建分别在
`docs/使用.md`、`docs/BUILD.md`。
