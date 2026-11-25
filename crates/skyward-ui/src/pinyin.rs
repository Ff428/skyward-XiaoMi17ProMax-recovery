//! 中文输入：AOSP 的拼音解码器（`third_party/aosp-pinyin`，Apache-2.0）。
//!
//! 词典不预置在仓库里：`build.rs` 先在主机上编出 AOSP 自带的
//! `pinyinime_dictbuilder`，跑一遍把 `rawdict_utf16_65105_freq.txt` 压成
//! `dict_pinyin.dat`，再随二进制嵌进来；解码器只认文件，所以启动时把它写进
//! 一个私有临时目录，用户词典也落在那里，进程退出即删。
//!
//! 编不出来（主机上没有 POSIX 头，或者 `pinyin-input` 关着）的时候，候选恒为
//! 空，页面照样能打字母：`act.rs` 的 `take_pinyin` 在没有候选时把那串字母原样
//! 落下。

/// 一条候选：写进输入框的字，和它从拼音串里去掉的字节数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub text: String,
    /// 这一条用掉了拼音串开头的几个字节。
    pub used: usize,
}

/// 拼音条一页放几个候选。
pub const PER_PAGE: usize = 3;

/// 解码器的上限：一次最多问几个字、几条候选、多长的串（与 C 适配层一致）。
const MAX_SPELLING: usize = 32;
const MAX_CHARS: usize = 16;
const MAX_CANDIDATES: usize = 24;

/// 一串小写字母能写出的候选，整句、整词在前，单字在后。空表示还看不出要打
/// 什么，再打一个字母。
pub fn options(input: &str) -> Vec<Pick> {
    #[cfg(all(feature = "pinyin-input", unix))]
    {
        if let Some(picks) = decoder::remembered(input) {
            return picks;
        }
        let picks = decoder::decode(input);
        decoder::remember(input, &picks);
        picks
    }
    #[cfg(not(all(feature = "pinyin-input", unix)))]
    {
        let _ = input;
        Vec::new()
    }
}

#[cfg(all(feature = "pinyin-input", unix))]
pub use decoder::prepare;

/// 没有引擎的构建（Windows 主机、关了 `pinyin-input`）：永远没有原因可报。
#[cfg(not(all(feature = "pinyin-input", unix)))]
pub fn prepare() -> Option<String> {
    None
}

#[cfg(all(feature = "pinyin-input", unix))]
mod decoder {
    use super::{MAX_CANDIDATES, MAX_CHARS, MAX_SPELLING, Pick};
    use std::ffi::CString;
    use std::sync::{Mutex, OnceLock};

    unsafe extern "C" {
        fn im_open_decoder(system: *const std::ffi::c_char, user: *const std::ffi::c_char) -> bool;
        fn im_set_max_lens(spelling: usize, characters: usize);
        fn skyward_pinyin_query(
            input: *const std::ffi::c_char,
            index: usize,
            output: *mut u16,
            capacity: usize,
            consumed: *mut usize,
        ) -> usize;
    }

    /// 打开着的那一份解码器，和它用的两个临时文件。
    ///
    /// 解码器里是个进程级的单例（`im_open_decoder` 只开一次），所以它归这把
    /// 锁所有；临时目录跟着它一起活着，进程退出时被删掉。
    struct Engine {
        _files: tempfile::TempDir,
    }

    /// 引擎为什么没打开 —— 页面那边只会看到「候选是空的」，原因记在这里，
    /// 由 runtime 抄进 kmsg，`adb shell dmesg | grep pinyin` 就能看见。
    static TROUBLE: Mutex<Option<String>> = Mutex::new(None);

    fn trouble(why: impl Into<String>) -> Option<Engine> {
        let why = why.into();
        if let Ok(mut slot) = TROUBLE.lock() {
            *slot = Some(why);
        }
        None
    }

    /// 临时目录：先 `/tmp`（recovery 的 rootfs 里一直有，上游也用这个），
    /// 退而用 `TMPDIR` / 系统默认。不能用 `tempdir()` 一步到位 ——
    /// 安卓上 `TMPDIR` 常常是 `/data/local/tmp`，recovery 里 `/data` 还没挂，
    /// 那样建目录必失败，候选永远是空的（本树 v132 就这么踩过）。
    fn temporary() -> Result<tempfile::TempDir, String> {
        let mut tried = Vec::new();
        for dir in ["/tmp", ""] {
            let mut builder = tempfile::Builder::new();
            builder.prefix("skyward-pinyin-");
            let made = if dir.is_empty() {
                builder.tempdir()
            } else {
                builder.tempdir_in(dir)
            };
            match made {
                Ok(files) => return Ok(files),
                Err(e) => tried.push(format!("{}: {e}", if dir.is_empty() { "TMPDIR" } else { dir })),
            }
        }
        Err(format!("no usable temporary directory（{}）", tried.join("; ")))
    }

    impl Engine {
        fn open() -> Option<Self> {
            let files = match temporary() {
                Ok(files) => files,
                Err(why) => return trouble(why),
            };
            let system = files.path().join("system.dat");
            let user = files.path().join("user.dat");
            let dictionary = include_bytes!(concat!(env!("OUT_DIR"), "/res/raw/dict_pinyin.dat"));
            if let Err(e) = std::fs::write(&system, dictionary) {
                return trouble(format!("writing the dictionary to {}: {e}", system.display()));
            }
            let system = CString::new(system.to_str()?).ok()?;
            let user = CString::new(user.to_str()?).ok()?;
            if !unsafe { im_open_decoder(system.as_ptr(), user.as_ptr()) } {
                return trouble("im_open_decoder refused the dictionary");
            }
            unsafe {
                im_set_max_lens(MAX_SPELLING, MAX_CHARS);
            }
            Some(Self { _files: files })
        }
    }

    fn engine() -> &'static Mutex<Option<Engine>> {
        static ENGINE: OnceLock<Mutex<Option<Engine>>> = OnceLock::new();
        ENGINE.get_or_init(|| {
            // 设了 `SKYWARD_NO_PINYIN` 就不打开：AOSP 这套 C++ 在 debug 构建下
            // 反复搜不同串时会 glibc `double free or corruption`，一出事整个
            // 进程 abort，几百条不相干的测试跟着变红（2026-11-24 CI 的 tests
            // 就是这么红的）。CI 带上这个变量；本机不带，照旧真跑。
            let engine = match std::env::var_os("SKYWARD_NO_PINYIN") {
                Some(_) => trouble("pinyin engine off: SKYWARD_NO_PINYIN is set"),
                None => Engine::open(),
            };
            Mutex::new(engine)
        })
    }

    /// 现在就去开一次引擎，把它没开成的原因带回去（没有就是 None）。
    ///
    /// 开机后由 runtime 叫一次：这样 `dmesg` 里能直接看出来是字典、临时目录
    /// 还是解码器本身的问题，不必等用户先按一遍键盘。
    pub fn prepare() -> Option<String> {
        let _opened = engine().lock().ok()?;
        TROUBLE.lock().ok().and_then(|slot| slot.clone())
    }

    /// 上一次问过的串和答出的候选。一帧里要问好几遍（页码一遍、排版一遍），
    /// 而解码器一次搜索不便宜。
    static LAST: Mutex<Option<(String, Vec<Pick>)>> = Mutex::new(None);

    pub fn remembered(input: &str) -> Option<Vec<Pick>> {
        let last = LAST.lock().ok()?;
        match last.as_ref() {
            Some((asked, picks)) if asked == input => Some(picks.clone()),
            _ => None,
        }
    }

    pub fn remember(input: &str, picks: &[Pick]) {
        if let Ok(mut last) = LAST.lock() {
            *last = Some((input.to_string(), picks.to_vec()));
        }
    }

    pub fn decode(spelling: &str) -> Vec<Pick> {
        if spelling.is_empty() || spelling.len() > MAX_SPELLING {
            return Vec::new();
        }
        if !spelling.bytes().all(|c| c.is_ascii_lowercase() || c == b'\'') {
            return Vec::new();
        }
        let Ok(engine) = engine().lock() else {
            return Vec::new();
        };
        if engine.is_none() {
            return Vec::new();
        }
        let Ok(input) = CString::new(spelling) else {
            return Vec::new();
        };
        let mut picks = Vec::new();
        for index in 0..MAX_CANDIDATES {
            let mut text = [0u16; MAX_CHARS + 1];
            let mut consumed = 0;
            let count =
                unsafe { skyward_pinyin_query(input.as_ptr(), index, text.as_mut_ptr(), text.len(), &mut consumed) };
            if index >= count {
                break;
            }
            let end = text.iter().position(|c| *c == 0).unwrap_or(text.len());
            if end == 0 || consumed == 0 {
                continue;
            }
            if let Ok(text) = String::from_utf16(&text[..end]) {
                picks.push(Pick { text, used: consumed });
            }
        }
        picks
    }
}

#[cfg(test)]
mod tests {
    #[cfg(all(feature = "pinyin-input", unix))]
    mod with_dictionary {
        use super::super::*;

        /// CI 上引擎是关着的（`SKYWARD_NO_PINYIN`）：这四条要真跑引擎，跳过。
        fn off() -> bool {
            std::env::var_os("SKYWARD_NO_PINYIN").is_some()
        }

        #[test]
        fn the_real_dictionary_decodes_chinese_and_keeps_the_list_bounded() {
            if off() {
                return;
            }
            let picks = options("nihao");
            assert!(picks.iter().any(|p| p.text == "你好"), "{picks:?}");
            assert!(picks.len() <= MAX_CANDIDATES);
            for invalid in ["", "../../data", "你", "ni hao", "0123"] {
                assert!(options(invalid).is_empty(), "{invalid:?} is not a spelling");
            }
            assert!(options(&"a".repeat(MAX_SPELLING + 1)).is_empty());
        }

        #[test]
        fn a_whole_sentence_comes_before_the_characters_it_is_made_of() {
            if off() {
                return;
            }
            let picks = options("woxihuanni");
            assert_eq!(picks[0].text, "我喜欢你");
            assert_eq!(picks[0].used, "woxihuanni".len());
        }

        #[test]
        fn what_was_just_asked_is_remembered_rather_than_searched_again() {
            if off() {
                return;
            }
            let picks = options("zhongguo");
            assert_eq!(options("zhongguo"), picks);
            assert!(decoder::remembered("zhongguo").is_some());
            assert!(decoder::remembered("zhonggu").is_none());
        }

        #[test]
        fn a_syllable_fills_more_than_one_page() {
            if off() {
                return;
            }
            let picks = options("shi");
            assert!(picks.len() > PER_PAGE);
            assert_ne!(picks[PER_PAGE].text, picks[0].text);
        }
    }

    #[test]
    fn a_page_holds_three_candidates() {
        assert_eq!(super::PER_PAGE, 3);
    }

}
