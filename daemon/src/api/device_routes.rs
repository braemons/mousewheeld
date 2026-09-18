//! The board, and the wire to it.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use crate::convert::device::{
    device_info_to_wire, firmware_versions_to_wire, version_report_to_wire, wire_line_to_wire,
    wire_log_to_wire,
};
use crate::daemon_state::Daemon;
use crate::model::device::{FirmwareVersions, WireLog};
use crate::model::version::{DeviceProtocol, VersionReport};
use crate::model::{ApiError, ApiResult};
use crate::wire;

/// What board is attached, and how the link behaves.
pub async fn read_device(State(daemon): State<Arc<Daemon>>) -> Json<wire::DeviceInfo> {
    let port = daemon.config.lock().unwrap().device.port.clone();
    Json(device_info_to_wire(daemon.device.info(port)))
}

/// What this daemon is, and what it speaks.
///
/// **Advertised, never gated.** A client that needs a field added last month
/// asks this first; a daemon that refused an older client's request on the
/// strength of a version would make every upgrade a coordinated one.
pub async fn read_version(State(daemon): State<Arc<Daemon>>) -> Json<wire::VersionReport> {
    let (speaks, floor, board) = daemon.device.protocol();
    Json(version_report_to_wire(VersionReport {
        daemon: env!("CARGO_PKG_VERSION").to_string(),
        api: API_VERSION,
        device_protocol: DeviceProtocol { speaks, floor, board },
        vinput_layout: vinput::layout::VERSION,
    }))
}

/// The API contract's major version — not the daemon's release.
///
/// Within it every change is additive: a field, a route or an enum value may
/// appear, and nothing is removed, renamed or given a new meaning. It moves
/// when that promise is broken, which from 1.0 onward is a deliberate and rare
/// event (`contracts/INTERACTIONS.md` §11).
const API_VERSION: u32 = 1;

/// Open the link, or say why not.
pub async fn connect(State(daemon): State<Arc<Daemon>>) -> ApiResult<Json<wire::DeviceInfo>> {
    if !daemon.device.connected() {
        return Err(ApiError::no_device().about(
            "start with --simulate for a wheel on a thread, or set [device] port in the rig config",
        ));
    }
    let port = daemon.config.lock().unwrap().device.port.clone();
    Ok(Json(device_info_to_wire(daemon.device.info(port))))
}

/// What the board runs, and what this daemon could put on it.
pub async fn read_firmware(State(daemon): State<Arc<Daemon>>) -> Json<wire::FirmwareVersions> {
    let port = daemon.config.lock().unwrap().device.port.clone();
    Json(firmware_versions_to_wire(FirmwareVersions {
        running: daemon.device.info(port).firmware_version,
        // Nothing to offer until there is firmware to build.
        available: Vec::new(),
    }))
}

/// The wire's recent past: the whole conversation, and the last hundred
/// samples.
///
/// A monitor that started at "now" would miss every fault that had already
/// happened, which is most of them — so the ring is handed over first and the
/// stream follows it. Samples are capped rather than the conversation, because
/// at 500 Hz they are the only thing that would come back.
pub async fn read_wire_log(State(daemon): State<Arc<Daemon>>) -> Json<wire::WireLog> {
    Json(wire_log_to_wire(WireLog {
        lines: daemon.device.wire_log(100),
    }))
}

/// The wire as it goes. One of the two endpoints that are streams by nature.
pub async fn wire_stream(State(daemon): State<Arc<Daemon>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |socket| follow_wire(socket, daemon))
}

async fn follow_wire(mut socket: WebSocket, daemon: Arc<Daemon>) {
    let mut lines = daemon.device.subscribe_wire();
    loop {
        match lines.recv().await {
            Ok(line) => {
                let Ok(text) = serde_json::to_string(&wire_line_to_wire(line)) else { continue };
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            // A slow reader on a log: skip what it missed and keep going. The
            // log is not the record, and blocking the wire for a browser tab
            // would be the wrong trade in the other direction.
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => return,
        }
    }
}
