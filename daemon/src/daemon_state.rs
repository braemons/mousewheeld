//! What every route shares.
//!
//! One `Daemon`, held by the router. The rig config lives behind a mutex
//! because a calibration change rewrites it; the store is a directory and needs
//! no lock of its own; the device has its own.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::device::Device;
use crate::model::calibration::{Calibration, MeasurementResult};
use crate::model::config::RigConfig;
use crate::zones::ZoneSetStore;

/// A measurement someone started and has not finished.
pub struct MeasurementInProgress {
    pub axis: String,
    pub known_distance_cm: f64,
    pub counts_at_start: i64,
}

pub struct Daemon {
    pub config_path: PathBuf,
    pub config: Mutex<RigConfig>,
    pub store: ZoneSetStore,
    pub device: Arc<Device>,
    pub measuring: Mutex<Option<MeasurementInProgress>>,
    /// The last finished measurement, waiting to be applied or discarded. Kept
    /// rather than applied at `finish`, because a calibration change
    /// invalidates every compiled zone set and deserves its own press.
    pub measured: Mutex<Option<MeasurementResult>>,
}

impl Daemon {
    pub fn calibration(&self) -> Calibration {
        self.config.lock().unwrap().calibration()
    }

    /// Persist the rig config. The file **is** the runtime shape, so this is a
    /// serialization of what is in memory and not a merge into something else.
    pub fn save_config(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap();
        config.save(&self.config_path)
    }
}
