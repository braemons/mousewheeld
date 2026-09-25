//! Counts per centimetre, and the procedure that measures it.
//!
//! The measurement is three presses on purpose. `start` notes the counter,
//! `finish` reports measured against configured, and `apply` is a separate
//! decision — because applying invalidates every compiled zone set, and because
//! the number is worth looking at before it becomes the rig's truth.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::convert::calibration::{
    calibration_patch_from_wire, calibration_to_wire, measurement_applied_to_wire,
    measurement_result_to_wire, measurement_started_to_wire, start_measurement_from_wire,
};
use crate::daemon_state::{Daemon, MeasurementInProgress};
use crate::model::calibration::{MeasurementApplied, MeasurementResult, MeasurementStarted};
use crate::model::{ApiError, ApiResult};
use crate::wire;
use crate::wire::service::calibration_server::{Calibration, CalibrationServer};

use super::status_of;

/// Ratios that mean a decoder rather than a wheel: quadrature counted ×1 or ×2
/// where the counts-per-revolution assumed ×4. It looks exactly like a wheel of
/// the wrong size, so the daemon says which it thinks it is.
const DECODING_RATIOS: [(f64, &str); 4] = [(4.0, "4×"), (2.0, "2×"), (0.5, "½×"), (0.25, "¼×")];

fn read_calibration_body(daemon: &Arc<Daemon>) -> wire::CalibrationState {
    calibration_to_wire(daemon.calibration())
}

/// Change a calibration by hand. Every field optional; the rest is left alone.
fn replace_calibration_body(
    daemon: &Arc<Daemon>,
    patch: wire::CalibrationPatch,
) -> ApiResult<wire::CalibrationState> {
    let patch = calibration_patch_from_wire(patch);
    {
        let mut config = daemon.config.lock().unwrap();
        for change in &patch.axes {
            let axis = config
                .axes
                .iter_mut()
                .find(|axis| axis.name == change.name)
                .ok_or_else(|| {
                    ApiError::not_found("no_such_axis", format!("no axis named {}", change.name))
                        .about(change.name.clone())
                })?;
            if let Some(value) = change.counts_per_cm {
                if value <= 0.0 {
                    return Err(ApiError::refused(
                        "bad_calibration",
                        "counts_per_cm is above zero — a wheel that counts nothing is not a calibration",
                    )
                    .about(change.name.clone()));
                }
                axis.counts_per_cm = value;
                // Typed in rather than measured: the record says so, so nobody
                // later mistakes a guess for a measurement.
                axis.measured_at = None;
            }
            if let Some(value) = change.counts_per_rev {
                axis.counts_per_rev = Some(value);
            }
            if let Some(value) = change.diameter_cm {
                axis.diameter_cm = Some(value);
            }
            if let Some(value) = change.invert {
                // Onto the running axis first: it is the one that can refuse.
                daemon
                    .device
                    .set_invert(&change.name, value)
                    .map_err(|problem| {
                        ApiError::refused("armed", problem).about(change.name.clone())
                    })?;
                axis.invert = value;
            }
        }
    }
    apply_to_running_axes(daemon);
    daemon
        .save_calibration()
        .map_err(|problem| ApiError::internal("calibration_unwritable", problem))?;
    Ok(calibration_to_wire(daemon.calibration()))
}

/// The 2-D skeleton: modelled, routed, and refused by name.
fn read_ball_body() -> ApiResult<wire::BallCalibration> {
    Err(ball_is_not_here())
}

fn replace_ball_body() -> ApiResult<wire::BallCalibration> {
    Err(ball_is_not_here())
}

fn ball_is_not_here() -> ApiError {
    ApiError::not_implemented(
        "ball calibration — mounting angle, ball diameter, the transform onto x, y and yaw — \
         arrives with the hardware. The model and this route exist so that adding it is filling \
         in rather than redesigning",
    )
}

/// Note the counter and start counting.
fn start_measurement_body(
    daemon: &Arc<Daemon>,
    request: wire::StartMeasurement,
) -> ApiResult<wire::MeasurementStarted> {
    let request = start_measurement_from_wire(request);
    if request.known_distance_cm <= 0.0 {
        return Err(ApiError::refused(
            "bad_distance",
            "name the distance you are about to roll, in centimetres",
        ));
    }
    let counts =
        daemon
            .device
            .counts_of(&request.axis)
            .ok_or_else(|| match daemon.device.connected() {
                true => {
                    ApiError::not_found("no_such_axis", format!("no axis named {}", request.axis))
                        .about(request.axis.clone())
                }
                false => ApiError::no_device(),
            })?;
    *daemon.measuring.lock().unwrap() = Some(MeasurementInProgress {
        axis: request.axis.clone(),
        known_distance_cm: request.known_distance_cm,
        counts_at_start: counts,
    });
    *daemon.measured.lock().unwrap() = None;
    Ok(measurement_started_to_wire(MeasurementStarted {
        axis: request.axis,
        known_distance_cm: request.known_distance_cm,
        counts_at_start: counts,
        counts,
    }))
}

/// Stop counting and report — measured beside configured, and what the
/// difference looks like.
fn finish_measurement_body(daemon: &Arc<Daemon>) -> ApiResult<wire::MeasurementResult> {
    let started = daemon
        .measuring
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| ApiError::conflict("not_measuring", "no measurement is in progress"))?;

    let counts_now = daemon
        .device
        .counts_of(&started.axis)
        .ok_or_else(ApiError::no_device)?;
    let counts = (counts_now - started.counts_at_start).abs();
    if counts < 100 {
        return Err(ApiError::refused(
            "too_few_counts",
            format!("{counts} counts is not enough to calibrate anything — did the wheel turn?"),
        )
        .about(started.axis));
    }

    let configured = daemon
        .calibration()
        .axis(&started.axis)
        .map(|axis| axis.counts_per_cm)
        .unwrap_or(1.0);
    let counts_per_rev = daemon
        .calibration()
        .axis(&started.axis)
        .and_then(|axis| axis.counts_per_rev);
    let measured = counts as f64 / started.known_distance_cm;
    let ratio = measured / configured;
    let axis_name = started.axis.clone();

    let result = MeasurementResult {
        axis: started.axis,
        known_distance_cm: started.known_distance_cm,
        counts,
        measured_counts_per_cm: measured,
        configured_counts_per_cm: configured,
        counts_per_rev,
        nominal_counts_per_cm: daemon
            .calibration()
            .axis(&axis_name)
            .and_then(|axis| axis.nominal_counts_per_cm()),
        implied_circumference_cm: counts_per_rev.map(|rev| f64::from(rev) / measured),
        decoding_suspect: DECODING_RATIOS
            .iter()
            .find(|(factor, _)| (ratio - factor).abs() < 0.05)
            .map(|(_, label)| {
                format!(
                    "the measurement is {label} the configured value — a whole-number ratio is a \
                     decoding mistake, not a wheel. Check the quadrature multiplier before applying it"
                )
            }),
    };
    *daemon.measured.lock().unwrap() = Some(result.clone());
    Ok(measurement_result_to_wire(result))
}

/// Make the measurement the rig's truth.
fn apply_measurement_body(daemon: &Arc<Daemon>) -> ApiResult<wire::MeasurementApplied> {
    let result = daemon.measured.lock().unwrap().take().ok_or_else(|| {
        ApiError::conflict(
            "nothing_measured",
            "finish a measurement before applying one",
        )
    })?;

    {
        let mut config = daemon.config.lock().unwrap();
        let axis = config
            .axes
            .iter_mut()
            .find(|axis| axis.name == result.axis)
            .ok_or_else(|| {
                ApiError::not_found("no_such_axis", format!("no axis named {}", result.axis))
            })?;
        axis.counts_per_cm = result.measured_counts_per_cm;
        axis.measured_at = Some(now_as_text());
    }
    apply_to_running_axes(daemon);
    daemon
        .save_calibration()
        .map_err(|problem| ApiError::internal("calibration_unwritable", problem))?;

    // Every compiled set was compiled against the old number. Disarming is the
    // honest response: a set armed under one calibration is never silently
    // reinterpreted under another, and the next arm recompiles.
    daemon.device.disarm();

    Ok(measurement_applied_to_wire(MeasurementApplied {
        axis: result.axis,
        counts_per_cm: result.measured_counts_per_cm,
        zone_sets_invalidated: true,
    }))
}

/// Push the calibration into the running axes, so what the API reports in
/// centimetres changes the moment the number behind it does.
fn apply_to_running_axes(daemon: &Arc<Daemon>) {
    let calibration = daemon.calibration();
    for axis in &calibration.axes {
        daemon.device.recalibrate(&axis.name, axis.counts_per_cm);
    }
}

/// A timestamp a person reads, not one a machine parses: this lands in a TOML
/// file somebody opens in an editor.
fn now_as_text() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    let days = seconds / 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let rest = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        rest / 3600,
        (rest % 3600) / 60
    )
}

/// Howard Hinnant's days-to-civil, which is public domain and shorter than a
/// date crate for the one timestamp this daemon writes.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ------------------------------------------------------------- the service ---
//
// Thin on purpose: convert, call the body above, map a refusal to a status.
// The bodies are unchanged from when these were HTTP handlers, which is the
// point — moving transport was not meant to be an opportunity to rewrite the
// measurement procedure.

#[tonic::async_trait]
impl Calibration for super::DaemonServices {
    async fn read_calibration(
        &self,
        _request: Request<wire::ReadCalibrationRequest>,
    ) -> Result<Response<wire::CalibrationState>, Status> {
        Ok(Response::new(read_calibration_body(&self.daemon)))
    }

    async fn replace_calibration(
        &self,
        request: Request<wire::CalibrationPatch>,
    ) -> Result<Response<wire::CalibrationState>, Status> {
        replace_calibration_body(&self.daemon, request.into_inner())
            .map(Response::new)
            .map_err(status_of)
    }

    async fn read_ball(
        &self,
        _request: Request<wire::ReadBallRequest>,
    ) -> Result<Response<wire::BallCalibration>, Status> {
        read_ball_body().map(Response::new).map_err(status_of)
    }

    async fn replace_ball(
        &self,
        _request: Request<wire::BallCalibration>,
    ) -> Result<Response<wire::BallCalibration>, Status> {
        replace_ball_body().map(Response::new).map_err(status_of)
    }

    async fn start_measuring(
        &self,
        request: Request<wire::StartMeasurement>,
    ) -> Result<Response<wire::MeasurementStarted>, Status> {
        start_measurement_body(&self.daemon, request.into_inner())
            .map(Response::new)
            .map_err(status_of)
    }

    async fn finish_measuring(
        &self,
        _request: Request<wire::FinishMeasuringRequest>,
    ) -> Result<Response<wire::MeasurementResult>, Status> {
        finish_measurement_body(&self.daemon)
            .map(Response::new)
            .map_err(status_of)
    }

    async fn apply_measurement(
        &self,
        _request: Request<wire::ApplyMeasurementRequest>,
    ) -> Result<Response<wire::MeasurementApplied>, Status> {
        apply_measurement_body(&self.daemon)
            .map(Response::new)
            .map_err(status_of)
    }
}

pub fn server(services: super::DaemonServices) -> CalibrationServer<super::DaemonServices> {
    CalibrationServer::new(services)
}
