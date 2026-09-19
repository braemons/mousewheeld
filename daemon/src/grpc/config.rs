// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rates, and the line map.
//!
//! The split this file exists to hold: a **session** changes rates, so they are
//! writable; **wiring** is changed where wiring is described, so the line map
//! is read-only here. A rig that was rewired edits one file and restarts; a rig
//! running an experiment does not get its pins moved from a browser.

use tonic::{Request, Response, Status};

use crate::convert::config::{config_patch_from_wire, config_view_to_wire, line_map_to_wire};
use crate::model::config::ConfigView;
use crate::model::line_map::LineMap;
use crate::wire;
use crate::wire::service::config_server::{Config, ConfigServer};

#[tonic::async_trait]
impl Config for super::DaemonServices {
    async fn read_config(
        &self,
        _request: Request<wire::ReadConfigRequest>,
    ) -> Result<Response<wire::ConfigView>, Status> {
        Ok(Response::new(config_view_to_wire(self.view())))
    }

    async fn patch_config(
        &self,
        request: Request<wire::ConfigPatch>,
    ) -> Result<Response<wire::ConfigView>, Status> {
        let patch = config_patch_from_wire(request.into_inner());
        {
            let mut config = self.daemon.config.lock().unwrap();
            if let Some(rate_hz) = patch.rate_hz {
                if !(1..=5_000).contains(&rate_hz) {
                    return Err(Status::invalid_argument(
                        "rate_hz is between 1 and 5000 — above the scan there is nothing \
                         more to send",
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
                    "stream rate {} Hz is at or below the display's {} Hz: some frames will \
                     see no new sample and the next will see two, which is visible stutter",
                    config.stream.rate_hz,
                    config.stream.display_hz
                );
            }
        }
        self.daemon
            .save_config()
            .map_err(|problem| Status::internal(format!("the rig config is unwritable: {problem}")))?;
        Ok(Response::new(config_view_to_wire(self.view())))
    }

    async fn read_lines(
        &self,
        _request: Request<wire::ReadLinesRequest>,
    ) -> Result<Response<wire::LineMap>, Status> {
        Ok(Response::new(line_map_to_wire(LineMap {
            lines: self.daemon.config.lock().unwrap().lines.clone(),
        })))
    }
}

impl super::DaemonServices {
    /// The settings a session may change and the ones it may only read, joined
    /// with what the publisher is actually doing.
    fn view(&self) -> ConfigView {
        let publishing = self.daemon.device.publishing();
        let config = self.daemon.config.lock().unwrap();
        ConfigView {
            shm_open: publishing.is_some(),
            shm_writes: publishing.map(|(_, writes)| writes).unwrap_or(0),
            rate_hz: config.stream.rate_hz,
            display_hz: config.stream.display_hz,
            ring_minutes: config.stream.ring_minutes,
            shm_name: config.publish.shm_name.clone(),
            event_port: config.publish.event_port,
            port: config.device.port.clone(),
            starves_the_display: config.stream.starves_the_display(),
        }
    }
}

pub fn server(services: super::DaemonServices) -> ConfigServer<super::DaemonServices> {
    ConfigServer::new(services)
}
