//! 两件本机编译的活：openh264 的 shim，和 AOSP 的拼音引擎。
//!
//! **openh264**：它自己没法在构建时现编（要 NDK 的 clang、八十多个源文件、
//! 几分钟），所以 `native/aarch64-linux-android/libopenh264.a` 是提交进仓库
//! 的二进制，来源、版本和重编命令写在 `native/README.md`。这一段只做三件事：
//!
//! 1. 用 NDK 的 clang++ 编 `h264_shim.cpp`（C 包装，见那边的注释）；
//! 2. 把两个静态库解包再打成一个 —— `ar` 不能把一个 `.a` 塞进另一个 `.a`，
//!    而链接时库的顺序又不好控（shim 引 openh264 的符号），合成一个最省心；
//! 3. 告诉 rustc 去链这一个。
//!
//! **拼音**（`pinyin-input`，默认开）：`third_party/aosp-pinyin` 是原样的
//! AOSP 解码器，词典不预置 —— 先在**主机**上编出字典构建器，跑一遍生成
//! `dict_pinyin.dat`，再把解码器编给目标，词典随二进制嵌进去。上游那份 ARM32
//! 的词典不用：它按 32 位 `size_t` 序列化，64 位目标读不了。
//!
//! 主机上没有 POSIX 头（Windows）时，拼音这条链跳过：押拼音的代码按 `unix`
//! 条件编译，那种主机上候选恒为空，不影响交叉编给 Android 的产物。openh264
//! 只对 aarch64-linux-android 做，别的目标（`cargo test`、`skyward-pack` 的
//! 依赖检查）不引用 `sky_h264_*`，也就不会去找这个库。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    if !out.status.success() {
        panic!(
            "{:?} 失败：\n{}\n{}",
            cmd,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn env_any(names: &[&str], fallback: &str) -> String {
    for name in names {
        if let Ok(v) = env::var(name)
            && !v.is_empty()
        {
            return v;
        }
    }
    fallback.to_string()
}

/// 真的 libc++ 会不会被静态接上：拼音开着、而且目标带 POSIX 头（Android/Linux）。
fn links_cxx_runtime() -> bool {
    env::var_os("CARGO_FEATURE_PINYIN_INPUT").is_some() && env::var_os("CARGO_CFG_UNIX").is_some()
}

/// 拼音引擎：主机上生成词典，目标上编解码器，再把 NDK 的 C++ 运行库静态接上。
///
/// NDK 把 `libc++_static.a` 和 API 头、`libc.a`/`libm.a` 放在同一个 `lib/`
/// 目录里。那条目录一进 `link-search`，C 运行库就会先被找到，成品变成静态
/// Bionic 的动态可执行文件，跑到 `main` 之前就崩。所以只把两个 C++ 归档单独
/// 放一处，Android 的 C 运行库仍旧动态。
fn build_pinyin() {
    if !links_cxx_runtime() {
        return;
    }
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("manifest directory");
    let root = Path::new(&manifest);
    // 不做 `canonicalize`：Windows 上它给出 `\\?\` 前缀的 verbatim 路径，mingw 的
    // g++ 认不出，`..` 交给编译器自己折叠。
    let source = root.join("../../third_party/aosp-pinyin/jni");
    let native = root.join("native");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("build directory"));

    let mut files: Vec<PathBuf> = fs::read_dir(source.join("share"))
        .expect("pinyin sources")
        .map(|f| f.expect("pinyin source").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "cpp"))
        .collect();
    files.sort();

    // 词典的二进制格式里有 `size_t`，只能给 64 位小端的目标用。
    assert_eq!(env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap(), "64");
    assert_eq!(env::var("CARGO_CFG_TARGET_ENDIAN").unwrap(), "little");

    // 一、字典构建器编给主机，跑一遍。
    let host = env::var("HOST").expect("host triple");
    let compiler = cc::Build::new().cpp(true).host(&host).target(&host).get_compiler();
    let builder = out.join("pinyin-dictbuilder");
    let mut command = compiler.to_command();
    command
        .args(["-std=c++11", "-O2", "-w", "-D___BUILD_MODEL___"])
        .arg("-I")
        .arg(source.join("include"))
        .arg("-I")
        .arg(&native);
    for file in &files {
        command.arg(file);
    }
    command.arg(source.join("command/pinyinime_dictbuilder.cpp")).arg("-o").arg(&builder);
    // mingw 的 libstdc++ / libwinpthread 是 DLL：不静态链，构建器在没有 mingw 的
    // `PATH` 下（出包脚本自己拼的那一份）根本起不来，报出来的却只是「构建词典
    // 失败」。静态链一次，这只 exe 就不再依赖那几个 DLL。
    if host.contains("windows") {
        command.arg("-static");
    }
    run(&mut command);

    // 构建器把词典写进 cwd 的 `../../res/raw/`，所以在一层空目录里跑它。
    let cwd = out.join("jni/command");
    fs::create_dir_all(&cwd).expect("dictionary work directory");
    fs::create_dir_all(out.join("res/raw")).expect("dictionary output directory");
    let made = Command::new(&builder)
        .current_dir(&cwd)
        .arg(source.join("data/rawdict_utf16_65105_freq.txt"))
        .arg(source.join("data/valid_utf16.txt"))
        .output()
        .expect("generate the 64-bit dictionary");
    assert!(
        made.status.success(),
        "the dictionary builder failed ({:?}):\n{}\n{}",
        made.status,
        String::from_utf8_lossy(&made.stdout),
        String::from_utf8_lossy(&made.stderr)
    );
    let dictionary = out.join("res/raw/dict_pinyin.dat");
    assert!(dictionary.is_file(), "the dictionary builder wrote nothing: {}", dictionary.display());

    // 二、解码器编给目标。
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++11")
        .warnings(false)
        .include(source.join("include"))
        .include(&native);
    for file in &files {
        build.file(file);
    }
    build.file(native.join("pinyin.cpp"));
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        build.cpp_link_stdlib(None);
    }
    build.compile("skyward_pinyin");

    // 三、Android 上把 libc++ 静态接上，且只暴露这两个归档。
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        let compiler = cc::Build::new().cpp(true).get_compiler();
        let location = compiler
            .to_command()
            .arg("-print-file-name=libc++_static.a")
            .output()
            .expect("locate the NDK C++ runtime");
        assert!(location.status.success());
        let library = PathBuf::from(String::from_utf8(location.stdout).unwrap().trim());
        assert!(library.is_file(), "NDK static C++ runtime not found: {}", library.display());
        let runtime = out.join("cxx-runtime");
        fs::create_dir_all(&runtime).expect("C++ archive staging");
        for name in ["libc++_static.a", "libc++abi.a"] {
            fs::copy(library.parent().unwrap().join(name), runtime.join(name)).expect("stage the NDK C++ archive");
        }
        println!("cargo:rustc-link-search=native={}", runtime.display());
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=static=c++abi");
    }

    println!("cargo:rerun-if-changed=../../third_party/aosp-pinyin");
    println!("cargo:rerun-if-changed=native/pinyin.cpp");
    println!("cargo:rerun-if-changed=native/cutils/log.h");
}

fn build_openh264() {
    let target = env::var("TARGET").unwrap_or_default();
    if !target.starts_with("aarch64-linux-android") {
        return;
    }
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let native = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR")).join("native");
    let shim = native.join("h264_shim.cpp");
    println!("cargo:rerun-if-changed={}", shim.display());
    println!("cargo:rerun-if-changed={}", native.join("aarch64-linux-android/libopenh264.a").display());

    let cxx = env_any(&["CXX_aarch64_linux_android", "CXX_aarch64-linux-android", "CXX"], "clang++");
    let ar = env_any(&["AR_aarch64_linux_android", "AR_aarch64-linux-android", "AR"], "llvm-ar");

    let obj = out.join("h264_shim.o");
    let mut cc = Command::new(&cxx);
    cc.arg(format!("--target={target}"))
        .arg("-std=c++11")
        .arg("-O2")
        .arg("-fPIC")
        .arg("-fno-rtti")
        .arg("-fno-exceptions")
        .arg(format!("-I{}", native.join("include").display()))
        .arg("-c")
        .arg(&shim)
        .arg("-o")
        .arg(&obj);
    // 真的 libc++ 已经接上时（拼音那一路），shim 里那几个桩要让位，否则重复定义。
    if links_cxx_runtime() {
        cc.arg("-DSKYWARD_HAS_CXX_RUNTIME");
    }
    run(&mut cc);

    let unpack = out.join("openh264");
    let _ = fs::remove_dir_all(&unpack);
    fs::create_dir_all(&unpack).expect("mkdir openh264");
    let mut x = Command::new(&ar);
    x.current_dir(&unpack)
        .arg("x")
        .arg(native.join("aarch64-linux-android/libopenh264.a"));
    run(&mut x);

    let mut objs: Vec<PathBuf> = fs::read_dir(&unpack)
        .expect("read openh264")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "o").unwrap_or(false))
        .collect();
    objs.sort();
    let count = objs.len();
    objs.push(obj);

    let merged = out.join("libskyward_h264.a");
    let _ = fs::remove_file(&merged);
    let mut pack = Command::new(&ar);
    pack.current_dir(&out).arg("rcs").arg(&merged).args(&objs);
    run(&mut pack);

    println!("cargo:rustc-link-search=native={}", out.display());
    // 整个归档都要进去：openh264 内部靠函数指针表互相引用（`WelsInitMeFunc`
    // 之类），纯按需拉取时 ld 只解出引用方那一个成员，被指向的 `.S.o` 就漏了。
    // 链接时 `--gc-sections` 还会把真正用不到的丢掉。
    println!("cargo:rustc-link-lib=static:+whole-archive=skyward_h264");
    println!("cargo:warning=openh264: {count} objects + the shim → libskyward_h264.a");
}

fn main() {
    build_pinyin();
    build_openh264();
}
