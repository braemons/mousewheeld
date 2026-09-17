//! Rates, and the line map.
//!
//! The split this file exists to hold: a **session** changes rates, so they are
//! writable; **wiring** is changed where wiring is described, so the line map
//! is read-only here. A rig that was rewired edits one file and restarts; a rig
//! that is running an experiment does not get its pins moved from a browser.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::api::ApiJson;
use crate::daemon_state::Daemon;
use crate::model::config::{ConfigPatch, ConfigView};
use crate::model::line_map::LineMap;
use crate::model::{ApiError, ApiResult};

#[utoipa::path(get, path = "/api/config", tag = "config",
    responses((status = 200, body = ConfigView)))]
pub async fn read_config(State(daemon): State<Arc<Daemon>>) -> Json<ConfigView> {
    let config = daemon.config.lock().unwrap();
    Json(ConfigView {
        rate_hz: config.stream.rate_hz,
        display_hz: config.stream.display_hz,
        ring_minutes: config.stream.ring_minutes,
        shm_name: config.publish.shm_name.clone(),
        event_port: config.publish.event_port,
        port: config.device.port.clone(),
        starves_the_display: config.stream.starves_the_display(),
    })
}

#[utoipa::path(patch, path = "/api/config", tag = "config",
    request_body = ConfigPatch,
    responses((status = 200, body = ConfigView), (status = 422, description = "refused")))]
pub async fn patch_config(
    State(daemon): State<Arc<Daemon>>,
    ApiJson(patch): ApiJson<ConfigPatch>,
) -> ApiResult<Json<ConfigView>> {
    {
        let mut config = daemon.config.lock().unwrap();
        if let Some(rate_hz) = patch.rate_hz {
            if !(1..=5_000).contains(&rate_hz) {
                return Err(ApiError::refused(
                    "bad_rate",
                    "rate_hz is between 1 and 5000 — above the scan there is nothing more to send",
                ));
            }
            config.stream.rate_hz = rate_hz;
        }
        if let Some(display_hz) = patch.display_hz {
            config.stream.display_hz = display_hz;
        }
        if let Some(ring_minutes) = patch.ring_minutes {
            config.stream.ring_minutes = ring_minutes;
        }
        // Warned rather than refused: a rig with no camera on the wheel is
        // entitled to a low rate, and the daemon does not know which it is.
        if config.stream.starves_the_display() {
            log::warn!(
                "stream rate {} Hz is at or below the display's {} Hz: some frames will see no \
                 new sample and the next will see two, which is visible stutter",
                config.stream.rate_hz,
                config.stream.display_hz
            );
        }
    }
    daemon
        .save_config()
        .map_err(|problem| ApiError::internal("config_unwritable", problem))?;
    Ok(read_config(State(daemon)).await)
}

/// The output line map, as the rig config describes it. Not writable: see the
/// module note.
#[utoipa::path(get, path = "/api/lines", tag = "config",
    responses((status = 200, body = LineMap)))]
pub async fn read_lines(State(daemon): State<Arc<Daemon>>) -> Json<LineMap> {
    Json(LineMap {
        lines: daemon.config.lock().unwrap().lines.clone(),
    })
}
