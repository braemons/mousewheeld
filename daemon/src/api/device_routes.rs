//! The board, and the wire to it.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use crate::daemon_state::Daemon;
use crate::model::device::{DeviceInfo, FirmwareVersions, WireLog};
use crate::model::{ApiError, ApiResult};

/// What board is attached, and how the link behaves.
#[utoipa::path(
    get, path = "/api/device", tag = "device",
    responses((status = 200, body = DeviceInfo)),
)]
pub async fn read_device(State(daemon): State<Arc<Daemon>>) -> Json<DeviceInfo> {
    let port = daemon.config.lock().unwrap().device.port.clone();
    Json(daemon.device.info(port))
}

/// Open the link, or say why not.
#[utoipa::path(
    post, path = "/api/device/connect", tag = "device",
    responses(
        (status = 200, body = DeviceInfo),
        (status = 503, description = "no board on this host"),
    ),
)]
pub async fn connect(State(daemon): State<Arc<Daemon>>) -> ApiResult<Json<DeviceInfo>> {
    if !daemon.device.connected() {
        return Err(ApiError::no_device().about(
            "start with --simulate for a wheel on a thread, or set [device] port in the rig config",
        ));
    }
    let port = daemon.config.lock().unwrap().device.port.clone();
    Ok(Json(daemon.device.info(port)))
}

/// What the board runs, and what this daemon could put on it.
#[utoipa::path(get, path = "/api/device/firmware", tag = "device",
    responses((status = 200, body = FirmwareVersions)))]
pub async fn read_firmware(State(daemon): State<Arc<Daemon>>) -> Json<FirmwareVersions> {
    let port = daemon.config.lock().unwrap().device.port.clone();
    Json(FirmwareVersions {
        running: daemon.device.info(port).firmware_version,
        // Nothing to offer until there is firmware to build.
        available: Vec::new(),
    })
}

/// The wire's recent past.
///
/// A monitor that started at "now" would miss every fault that had already
/// happened, which is most of them — so the ring is handed over first and the
/// stream follows it.
#[utoipa::path(get, path = "/api/device/monitor", tag = "device",
    responses((status = 200, body = WireLog)))]
pub async fn read_wire_log(State(daemon): State<Arc<Daemon>>) -> Json<WireLog> {
    Json(WireLog {
        lines: daemon.device.wire_log(400),
    })
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
                let Ok(text) = serde_json::to_string(&line) else { continue };
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
