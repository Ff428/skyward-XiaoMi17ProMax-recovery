//! The phone's readings taken on a thread of their own, so the screen keeps
//! up: the battery's driver takes milliseconds to answer each read.

use anyhow::{Context, Result};
use skyward_config::DeviceConfig;
use skyward_info::{Battery, DeviceInfo};
use skyward_ui::app::ChargeNow;
use skyward_sys::worker::{self, Told};
use skyward_vab::SnapshotState;
use std::os::fd::RawFd;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// The battery is read this often, and at once when asked; all the
/// readings every so many times instead, and every time while the update's
/// state is not yet known.
const BATTERY_EVERY: Duration = Duration::from_secs(1);
const ALL_EVERY: u32 = 30;

/// What the charger was doing as the readings thread read it: `None` where
/// nothing is plugged in. Reading it walks the charger's and the battery's own
/// nodes, which take milliseconds to answer, so it is taken on this thread
/// rather than where the page is drawn.
pub type Charge = Option<ChargeNow>;

pub enum Reading {
    /// The battery, what the charger is doing, and when reading them started.
    Battery(Result<Option<Battery>>, Charge, Instant),
    /// All the readings, and when taking them started.
    All(Box<Result<DeviceInfo>>, Instant),
}

pub struct Readings {
    taken: Told<Reading>,
    asks: Sender<()>,
}

impl Readings {
    /// Start taking the readings of the phone `cfg` describes, the first a
    /// second from now. The thread ends once the screen lets them go.
    pub fn start(cfg: &DeviceConfig) -> Result<Readings> {
        let (tell, taken) = worker::telling().context("make the readings' wake pipe")?;
        let (asks, asked) = mpsc::channel::<()>();
        let cfg = cfg.clone();
        thread::Builder::new()
            .name("readings".into())
            .spawn(move || {
                let mut ticks = 0u32;
                // All of them are taken at once first, and once a second while
                // the update's state cannot be read: /metadata is mounted by
                // the work behind the screen, and a page that said the state
                // was unreadable for half a minute after it became readable
                // would be saying something that is no longer true.
                let mut full_next = true;
                loop {
                    let at_once = match asked.recv_timeout(BATTERY_EVERY) {
                        Ok(()) => {
                            while asked.try_recv().is_ok() {}
                            true
                        }
                        Err(RecvTimeoutError::Timeout) => false,
                        Err(RecvTimeoutError::Disconnected) => return,
                    };
                    if !at_once {
                        ticks += 1;
                    }
                    let at = Instant::now();
                    let reading = if !at_once && (full_next || ticks >= ALL_EVERY) {
                        let read = skyward_info::collect(&cfg);
                        full_next = !matches!(&read, Ok(info) if info.snapshot != SnapshotState::Unknown);
                        ticks = 0;
                        Reading::All(Box::new(read), at)
                    } else {
                        // The battery is read before the charger, since reading
                        // it is what has the kernel bring the charger's own
                        // nodes up to date: read first, the charger answered
                        // with the figures it had before the cable went in.
                        let battery = skyward_info::battery_now();
                        let charge = crate::charge::reading();
                        Reading::Battery(battery, charge, at)
                    };
                    if !tell.tell(reading) {
                        return;
                    }
                }
            })
            .context("start the readings thread")?;
        Ok(Readings { taken, asks })
    }

    /// Readable while readings wait.
    pub fn fd(&self) -> RawFd {
        self.taken.fd()
    }

    /// Read the battery at once.
    pub fn ask(&self) {
        self.asks.send(()).expect("the readings thread runs while its readings are taken");
    }

    /// The readings taken since the last call.
    pub fn taken(&self) -> Vec<Reading> {
        self.taken.take()
    }
}
