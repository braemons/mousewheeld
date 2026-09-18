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
use crate::convert::config::{config_patch_from_wire, config_view_to_wire, line_map_to_wire};
use crate::daemon_state::Daemon;
use crate::model::config::ConfigView;
use crate::model::line_map::LineMap;
use crate::model::{ApiError, ApiResult};
use crate::wire;

pub async fn read_config(State(daemon): State<Arc<Daemon>>) -> Json<wire::ConfigView> {
    let publishing = daemon.device.publishing();
    let config = daemon.config.lock().unwrap();
    Json(config_view_to_wire(ConfigView {
        shm_open: publishing.is_some(),
        shm_writes: publishing.map(|(_, writes)| writes).unwrap_or(0),
        rate_hz: config.stream.rate_hz,
        display_hz: config.stream.display_hz,
        ring_minutes: config.stream.ring_minutes,
        shm_name: config.publish.shm_name.clone(),
        event_port: config.publish.event_port,
        port: config.device.port.clone(),
        starves_the_display: config.stream.starves_the_display(),
    }))
}

pub async fn patch_config(
    State(daemon): State<Arc<Daemon>>,
    ApiJson(patch): ApiJson<wire::ConfigPatch>,
) -> ApiResult<Json<wire::ConfigView>> {
    let patch = config_patch_from_wire(patch);
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
pub async fn read_lines(State(daemon): State<Arc<Daemon>>) -> Json<wire::LineMap> {
    Json(line_map_to_wire(LineMap {
        lines: daemon.config.lock().unwrap().lines.clone(),
    }))
}
