//! What every route shares.
//!
//! One `Daemon`, held by the router. The rig config lives behind a mutex
//! because a calibration or a rate changes it in memory; the store is a
//! directory and needs no lock of its own; the device has its own.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::device::Device;
use crate::model::calibration::{Calibration, MeasurementResult};
use crate::model::config::{RigConfig, StoredCalibration};
use crate::zones::ZoneSetStore;

/// A measurement someone started and has not finished.
pub struct MeasurementInProgress {
    pub axis: String,
    pub known_distance_cm: f64,
    pub counts_at_start: i64,
}

pub struct Daemon {
    /// `calibration.toml` in the storage directory. The rig config itself is
    /// never written.
    pub calibration_path: PathBuf,
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

    /// Persist every axis's calibration, so that a measurement survives a
    /// restart. Written whole: the file is what is in memory, not a merge.
    pub fn save_calibration(&self) -> Result<(), String> {
        let axes = self.config.lock().unwrap().axes.clone();
        StoredCalibration { axes }.save(&self.calibration_path)
    }
}
