// SPDX-License-Identifier: AGPL-3.0-or-later
//! The board, and the wire to it.

use std::pin::Pin;

use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::convert::device::{
    device_info_to_wire, firmware_versions_to_wire, version_report_to_wire, wire_line_to_wire,
    wire_log_to_wire,
};
use crate::model::device::{FirmwareVersions, WireLog};
use crate::model::version::{DeviceProtocol, VersionReport};
use crate::wire;
use crate::wire::service::device_server::{Device, DeviceServer};

#[tonic::async_trait]
impl Device for super::DaemonServices {
    async fn read_version(
        &self,
        _request: Request<wire::ReadVersionRequest>,
    ) -> Result<Response<wire::VersionReport>, Status> {
        let (speaks, floor, board) = self.daemon.device.protocol();
        Ok(Response::new(version_report_to_wire(VersionReport {
            daemon: env!("CARGO_PKG_VERSION").to_string(),
            api: 1,
            device_protocol: DeviceProtocol { speaks, floor, board },
            vinput_layout: vinput::layout::VERSION,
        })))
    }

    async fn read_device(
        &self,
        _request: Request<wire::ReadDeviceRequest>,
    ) -> Result<Response<wire::DeviceInfo>, Status> {
        let port = self.daemon.config.lock().unwrap().device.port.clone();
        Ok(Response::new(device_info_to_wire(self.daemon.device.info(port))))
    }

    async fn open_link(
        &self,
        _request: Request<wire::OpenLinkRequest>,
    ) -> Result<Response<wire::DeviceInfo>, Status> {
        if !self.daemon.device.connected() {
            // A gRPC status rather than a hand-mapped HTTP code. `unavailable`
            // is the standard spelling of "nothing about the request is wrong
            // and it will work when the link comes back", which is exactly
            // what `ApiError::no_device` says in this daemon's own words.
            return Err(Status::unavailable(
                "no board is connected — start with --simulate, or set [device] port",
            ));
        }
        let port = self.daemon.config.lock().unwrap().device.port.clone();
        Ok(Response::new(device_info_to_wire(self.daemon.device.info(port))))
    }

    async fn read_firmware(
        &self,
        _request: Request<wire::ReadFirmwareRequest>,
    ) -> Result<Response<wire::FirmwareVersions>, Status> {
        let port = self.daemon.config.lock().unwrap().device.port.clone();
        Ok(Response::new(firmware_versions_to_wire(FirmwareVersions {
            running: self.daemon.device.info(port).firmware_version,
            available: Vec::new(),
        })))
    }

    async fn read_wire_log(
        &self,
        _request: Request<wire::ReadWireLogRequest>,
    ) -> Result<Response<wire::WireLog>, Status> {
        Ok(Response::new(wire_log_to_wire(WireLog {
            lines: self.daemon.device.wire_log(100),
        })))
    }

    type WatchWireStream =
        Pin<Box<dyn Stream<Item = Result<wire::WireLine, Status>> + Send + 'static>>;

    async fn watch_wire(
        &self,
        _request: Request<wire::WatchWireRequest>,
    ) -> Result<Response<Self::WatchWireStream>, Status> {
        // The hand-written websocket loop, as a type. Lag is a value rather
        // than a `continue` buried in a match, and cancellation is the
        // framework's problem rather than a `send` that happens to fail.
        let lines = BroadcastStream::new(self.daemon.device.subscribe_wire());
        let stream = lines.filter_map(|line| line.ok().map(|line| Ok(wire_line_to_wire(line))));
        Ok(Response::new(Box::pin(stream)))
    }
}

pub fn server(services: super::DaemonServices) -> DeviceServer<super::DaemonServices> {
    DeviceServer::new(services)
}
