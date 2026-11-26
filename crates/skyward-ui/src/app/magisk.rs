//! Magisk 的页：手机装的是哪一版，以及把 Magisk 的启动补丁器写进当前槽。

use super::*;

/// What the phone and Magisk's APK say before anything is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagiskStatus {
    pub installed: bool,
    pub has_backup: bool,
    /// The partition the record's backup came from, with its slot.
    pub backup_target: Option<String>,
    pub version: Option<String>,
    pub unlocked: bool,
    pub slot: Option<String>,
    /// The partition Magisk would patch on this phone.
    pub target: String,
}

impl App {
    /// What the phone and Magisk's APK say, read as the page opens.
    pub fn set_magisk(&mut self, status: Result<MagiskStatus, String>) {
        self.state.magisk = Some(status);
        self.refresh();
    }

    /// Writing Magisk has begun.
    pub fn magisk_started(&mut self) {
        self.state.magisk_job = Some(KernelSuJob { stage: KernelSuStage::Preparing, outcome: None, fetched: None, fallback: None });
        self.refresh();
    }

    /// How far writing Magisk has come.
    pub fn magisk_progress(&mut self, stage: KernelSuStage) {
        let job = self.state.magisk_job.get_or_insert(KernelSuJob { stage, outcome: None, fetched: None, fallback: None });
        if job.outcome.is_none() {
            job.stage = stage;
            self.refresh();
        }
    }

    /// Writing Magisk ended, and the page says how.
    pub fn magisk_ended(&mut self, end: KernelSuEnd, now: u64) {
        let job = self.state.magisk_job.get_or_insert(KernelSuJob { stage: KernelSuStage::Checking, outcome: None, fetched: None, fallback: None });
        job.outcome = Some(end);
        self.wake(now);
        self.refresh();
    }
}
