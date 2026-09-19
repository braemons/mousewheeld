//! Counts per centimetre, and where that number comes from.
//!
//! **This daemon owns the calibration.** The firmware knows counts and nothing
//! else; vstimd is told centimetres and restates nothing; triald needs
//! centimetres too and cannot read either one's config. One copy, here, and
//! every number that leaves is in centimetres.
//!
//! **A measurement beats the arithmetic, and the type says which it has.**
//! `counts_per_rev / (π · diameter_cm)` describes a wheel the animal does not
//! run on: what a corridor position is made of is how far the feet travelled,
//! on the surface they touch, with whatever tread is on it, a few percent off
//! the nominal every time. So `measured_at` is part of the record, and a
//! calibration that has never been measured says so rather than looking as
//! settled as one that has.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One axis' calibration, as the rig config stores it and the API returns it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AxisCalibration {
    /// The axis this describes, matching the axis names everywhere else.
    pub name: String,
    /// Counts per centimetre of travel at the running surface. The number that
    /// converts the device's only unit into everybody else's.
    pub counts_per_cm: f64,
    /// Encoder counts per revolution, after quadrature ×4. Kept for the
    /// arithmetic a measurement is checked against — a whole-number ratio
    /// between the two is a decoder counting ×1 or ×2, not a wheel of the wrong
    /// size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counts_per_rev: Option<u32>,
    /// The wheel's diameter — a full extent, never a radius. Porting from a
    /// package that specifies radii means doubling at the boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diameter_cm: Option<f64>,
    /// Count the other way. Applied by the device to its own accumulator, so
    /// everything downstream reads a wheel that turns forwards.
    #[serde(default)]
    pub invert: bool,
    /// When `counts_per_cm` was last measured rather than computed. `None` is
    /// not a missing field: it means nobody has ever rolled this wheel against
    /// a tape, and the panels say so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_at: Option<String>,
}

impl AxisCalibration {
    /// What the datasheet implies, for comparison. `None` when the wheel's
    /// dimensions were never given — in which case there is nothing to check
    /// a measurement against, which is itself worth knowing.
    pub fn nominal_counts_per_cm(&self) -> Option<f64> {
        match (self.counts_per_rev, self.diameter_cm) {
            (Some(counts), Some(diameter)) if diameter > 0.0 => {
                Some(f64::from(counts) / (std::f64::consts::PI * diameter))
            }
            _ => None,
        }
    }

    /// Centimetres to counts. **Not** where `invert` is applied: the device
    /// negates its own counting, once, so that every number above it —
    /// displacement, distance, a compiled zone's interval — is already in the
    /// direction the animal runs. A sign applied in two places is a sign
    /// applied in neither.
    pub fn cm_to_counts(&self, cm: f64) -> f64 {
        cm * self.counts_per_cm
    }
}

/// Every axis' calibration, plus the 2-D skeleton.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Calibration {
    pub axes: Vec<AxisCalibration>,
    /// The ball's transform — sensor mounting angle, ball diameter, the map
    /// onto `x`, `y` and `yaw`. Always `None` today; its routes answer 501. It
    /// exists in the model so that adding the hardware is filling in rather
    /// than redesigning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball: Option<BallCalibration>,
}

impl Calibration {
    pub fn axis(&self, name: &str) -> Option<&AxisCalibration> {
        self.axes.iter().find(|axis| axis.name == name)
    }
}

/// Declared, refused, and here so the shape of the answer is not a surprise.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BallCalibration {
    pub diameter_cm: f64,
    /// Where each sensor sits, in degrees around the ball.
    pub sensor_angles_deg: Vec<f64>,
}

/// A change to one axis: everything optional, so a panel can send one field.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AxisCalibrationPatch {
    pub name: String,
    pub counts_per_cm: Option<f64>,
    pub counts_per_rev: Option<u32>,
    pub diameter_cm: Option<f64>,
    pub invert: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CalibrationPatch {
    pub axes: Vec<AxisCalibrationPatch>,
}

// ----------------------------------------------------------- measurement ---

/// `Calibration.StartMeasuring`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StartMeasurement {
    pub axis: String,
    /// How far the wheel is about to be rolled, by hand, along its running
    /// surface. A metre in one continuous motion: a metre averages away where
    /// you started and stopped, and one revolution does not.
    pub known_distance_cm: f64,
}

/// A measurement in progress.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MeasurementStarted {
    pub axis: String,
    pub known_distance_cm: f64,
    /// The counter when counting began, so a UI can show progress without
    /// asking the daemon to keep a second copy of it.
    pub counts_at_start: i64,
    #[serde(rename = "counts")]
    pub counts: i64,
}

/// What the wheel said, beside what the config says. Never applied on its own:
/// a calibration change invalidates every compiled zone set, so it is a second,
/// deliberate press.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MeasurementResult {
    pub axis: String,
    pub known_distance_cm: f64,
    pub counts: i64,
    pub measured_counts_per_cm: f64,
    pub configured_counts_per_cm: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts_per_rev: Option<u32>,
    /// What the wheel's stated dimensions imply — the arithmetic this
    /// measurement exists to correct. On the report rather than left to a
    /// client, so a result is readable on its own a month later.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nominal_counts_per_cm: Option<f64>,
    /// The circumference the measurement implies, when the counts per
    /// revolution are known — the number to hold against the wheel with a tape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implied_circumference_cm: Option<f64>,
    /// Set when the measurement is a whole-number multiple of the configured
    /// value: a decoding mistake rather than a wheel, and worth saying in
    /// words before somebody applies it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoding_suspect: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MeasurementApplied {
    pub axis: String,
    pub counts_per_cm: f64,
    /// Compiled zone sets are stale now, and the next arm recompiles. A set
    /// armed under one calibration is never silently reinterpreted under
    /// another.
    pub zone_sets_invalidated: bool,
}
