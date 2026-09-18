// SPDX-License-Identifier: AGPL-3.0-or-later
//! Counts per centimetre, and how it was measured.

use crate::model::calibration as m;
use crate::wire;

pub fn calibration_to_wire(calibration: m::Calibration) -> wire::CalibrationState {
    wire::CalibrationState {
        axes: calibration
            .axes
            .into_iter()
            .map(axis_calibration_to_wire)
            .collect(),
        ball: calibration.ball.map(ball_to_wire),
    }
}

pub fn axis_calibration_to_wire(axis: m::AxisCalibration) -> wire::AxisCalibration {
    wire::AxisCalibration {
        name: axis.name,
        counts_per_cm: axis.counts_per_cm,
        counts_per_rev: axis.counts_per_rev,
        diameter_cm: axis.diameter_cm,
        invert: axis.invert,
        measured_at: axis.measured_at,
    }
}

pub fn ball_to_wire(ball: m::BallCalibration) -> wire::BallCalibration {
    wire::BallCalibration {
        diameter_cm: ball.diameter_cm,
        sensor_angles_deg: ball.sensor_angles_deg,
    }
}

pub fn calibration_patch_from_wire(patch: wire::CalibrationPatch) -> m::CalibrationPatch {
    m::CalibrationPatch {
        axes: patch
            .axes
            .into_iter()
            .map(|axis| m::AxisCalibrationPatch {
                name: axis.name,
                counts_per_cm: axis.counts_per_cm,
                counts_per_rev: axis.counts_per_rev,
                diameter_cm: axis.diameter_cm,
                invert: axis.invert,
            })
            .collect(),
    }
}

pub fn start_measurement_from_wire(start: wire::StartMeasurement) -> m::StartMeasurement {
    m::StartMeasurement {
        axis: start.axis,
        known_distance_cm: start.known_distance_cm,
    }
}

pub fn measurement_started_to_wire(started: m::MeasurementStarted) -> wire::MeasurementStarted {
    wire::MeasurementStarted {
        axis: started.axis,
        known_distance_cm: started.known_distance_cm,
        counts_at_start: started.counts_at_start,
        counts: started.counts,
    }
}

pub fn measurement_result_to_wire(result: m::MeasurementResult) -> wire::MeasurementResult {
    wire::MeasurementResult {
        axis: result.axis,
        known_distance_cm: result.known_distance_cm,
        counts: result.counts,
        measured_counts_per_cm: result.measured_counts_per_cm,
        configured_counts_per_cm: result.configured_counts_per_cm,
        counts_per_rev: result.counts_per_rev,
        nominal_counts_per_cm: result.nominal_counts_per_cm,
        implied_circumference_cm: result.implied_circumference_cm,
        decoding_suspect: result.decoding_suspect,
    }
}

pub fn measurement_applied_to_wire(applied: m::MeasurementApplied) -> wire::MeasurementApplied {
    wire::MeasurementApplied {
        axis: applied.axis,
        counts_per_cm: applied.counts_per_cm,
        zone_sets_invalidated: applied.zone_sets_invalidated,
    }
}
