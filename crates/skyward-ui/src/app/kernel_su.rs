//! KernelSU 的页：手机装的是哪一版、KMI 该选哪个，以及把选中的那个写进
//! boot 或 init_boot。

use super::*;

/// One module the recovery can write, as the page lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KmiRow {
    /// `android<release>-<kernel>`, as KernelSU names it.
    pub id: String,
    pub android: u8,
    pub kernel: String,
    /// The partition it goes into by default.
    pub target: String,
}

/// 从 `v3.3.2`、`3.3.1-56-gcfc60f8d`、`27.0` 这样的写法里取出三段数字。
pub fn version_parts(text: &str) -> Option<(u32, u32, u32)> {
    let trimmed = text.trim().trim_start_matches(['v', 'V']);
    let mut parts = [0u32; 3];
    let mut at = 0;
    for piece in trimmed.split(['.', '-', '+', ' ']) {
        if at >= 3 {
            break;
        }
        let digits: String = piece.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            break;
        }
        parts[at] = digits.parse().ok()?;
        at += 1;
    }
    (at > 0).then_some((parts[0], parts[1], parts[2]))
}

/// 在线的那一版比装着的这一版新吗：新 → Some(true)，一样或旧 → Some(false)，
/// 有一边读不出来 → None。只升不降，页面据此拦下写入（2026-11-26）。
pub fn is_newer(installed: Option<&str>, online: Option<&str>) -> Option<bool> {
    let (installed, online) = (version_parts(installed?)?, version_parts(online?)?);
    Some(online > installed)
}

/// What the phone and KernelSU's payloads say before anything is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelSuStatus {
    /// KernelSU's userspace was found in the data partition.
    pub installed: bool,
    /// The stock image this recovery wrote KernelSU over is still there.
    pub has_backup: bool,
    /// The partition the record's backup came from.
    pub backup_target: Option<String>,
    pub version: Option<String>,
    /// The bootloader lets its boot images be written.
    pub unlocked: bool,
    /// The slot whose boot image is written, where the phone has slots.
    pub slot: Option<String>,
    /// The row the kernel's own version picks, where it picks one.
    pub default: Option<usize>,
    pub kmis: Vec<KmiRow>,
}

/// How far writing KernelSU has come.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSuStage {
    Preparing,
    Patching,
    BackingUp,
    Writing,
    Checking,
}

/// How writing KernelSU ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelSuEnd {
    Installed { target: String, backup: String, manager: String },
    Restored { target: String },
    Failed(String),
}

/// Writing KernelSU, or how it last ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelSuJob {
    pub stage: KernelSuStage,
    pub outcome: Option<KernelSuEnd>,
    /// The latest release's version, once it has been downloaded.
    pub fetched: Option<String>,
    /// Why the latest release could not be fetched, when the built-in
    /// payloads were used instead.
    pub fallback: Option<String>,
}

impl KernelSuStage {
    pub fn done(self) -> bool {
        matches!(self, KernelSuStage::Checking)
    }
}

impl App {
    /// What the phone and KernelSU's payloads say, read as the page opens.
    pub fn set_kernel_su(&mut self, status: Result<KernelSuStatus, String>) {
        if let Ok(s) = &status
            && self.state.kernel_su_kmi.is_none()
        {
            self.state.kernel_su_kmi = s.default;
        }
        self.state.kernel_su = Some(status);
        self.refresh();
    }

    /// Choose the KMI to write, by its place in the list.
    pub fn set_kernel_su_kmi(&mut self, at: usize) {
        if self.state.kernel_su.as_ref().is_some_and(|s| s.as_ref().is_ok_and(|s| at < s.kmis.len())) {
            self.state.kernel_su_kmi = Some(at);
            self.refresh();
        }
    }

    /// Writing KernelSU has begun.
    pub fn kernel_su_started(&mut self) {
        self.state.kernel_su_job = Some(KernelSuJob { stage: KernelSuStage::Preparing, outcome: None, fetched: None, fallback: None });
        self.refresh();
    }

    /// How far writing KernelSU has come.
    pub fn kernel_su_progress(&mut self, stage: KernelSuStage) {
        let job = self.state.kernel_su_job.get_or_insert(KernelSuJob { stage, outcome: None, fetched: None, fallback: None });
        if job.outcome.is_none() {
            job.stage = stage;
            self.refresh();
        }
    }

    /// The latest release has been downloaded; the page and the toast say
    /// which version is being written.
    pub fn kernel_su_fetched(&mut self, version: String, now: u64) {
        if let Some(job) = &mut self.state.kernel_su_job {
            job.fetched = Some(version.clone());
        }
        let said = format!("{} {version}", tr!(self.state.lang, "已下载最新 KernelSU", "已下載最新 KernelSU", "Downloaded the newest KernelSU"));
        self.report(&said, now);
        self.refresh();
    }

    /// The latest release could not be fetched; the built-in payloads are
    /// used, and both the page and the toast say why.
    pub fn kernel_su_fallback(&mut self, why: String, now: u64) {
        if let Some(job) = &mut self.state.kernel_su_job {
            job.fallback = Some(why.clone());
        }
        let said = format!("{} {why}", tr!(self.state.lang, "在线获取失败，使用内置 KernelSU。", "線上取得失敗，使用內建 KernelSU。", "Couldn't fetch online; using the built-in KernelSU."));
        self.report(&said, now);
        self.refresh();
    }

    /// Writing KernelSU ended, and the page says how.
    pub fn kernel_su_ended(&mut self, end: KernelSuEnd, now: u64) {
        let job = self.state.kernel_su_job.get_or_insert(KernelSuJob { stage: KernelSuStage::Checking, outcome: None, fetched: None, fallback: None });
        job.outcome = Some(end);
        self.wake(now);
        self.refresh();
    }

    /// A root image was not written, and the toast says why.
    pub fn root_failed(&mut self, why: &str, now: u64) {
        let said = format!("{} {why}", tr!(self.state.lang, "未写入：", "未寫入：", "Not written:"));
        self.report(&said, now);
    }

    /// 下载进度与速度，页面照它写「42% · 1.2 MiB/s」。
    pub fn set_root_bytes(&mut self, done: u64, total: Option<u64>, rate: u64) {
        self.state.root_bytes = Some((done, total, rate));
        self.refresh();
    }

    /// 在线上的两个最新版本号，页面先告诉用户会装到哪一版。
    pub fn set_latest_root(&mut self, kernelsu: Option<String>, magisk: Option<String>) {
        self.state.latest_kernelsu = kernelsu;
        self.state.latest_magisk = magisk;
        self.refresh();
    }

    /// 下载结束（成功、失败或取消），不再显示进度。
    pub fn clear_root_bytes(&mut self) {
        self.state.root_bytes = None;
        self.refresh();
    }

    /// The KMI the page would write.
    pub fn kernel_su_kmi(&self) -> Option<&KmiRow> {
        let status = self.state.kernel_su.as_ref()?.as_ref().ok()?;
        let at = self.state.kernel_su_kmi.or(status.default)?;
        status.kmis.get(at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_online_version_is_read_and_compared_with_the_installed_one() {
        assert_eq!(version_parts("v3.3.2"), Some((3, 3, 2)));
        assert_eq!(version_parts("3.3.1-56-gcfc60f8d"), Some((3, 3, 1)));
        assert_eq!(version_parts("27.0"), Some((27, 0, 0)));
        assert_eq!(version_parts("nightly"), None);
        assert_eq!(is_newer(Some("3.3.1-56-gcfc60f8d"), Some("v3.3.2")), Some(true), "升级");
        assert_eq!(is_newer(Some("3.3.2"), Some("v3.3.2")), Some(false), "一样不算升级");
        assert_eq!(is_newer(Some("3.3.3"), Some("v3.3.2")), Some(false), "不降级");
        assert_eq!(is_newer(None, Some("v3.3.2")), None, "没装过就不比");
    }
}
