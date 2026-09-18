//! Where the wheel is, and the stream that says so as it happens.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::Json;

use crate::api::ApiJson;
use crate::convert::state::{rig_state_to_wire, stream_frame_to_wire, zero_request_from_wire};
use crate::daemon_state::Daemon;
use crate::model::state::StreamFrame;
use crate::wire;

/// Per axis: counts, position, distance, both velocities. Plus whether the
/// link is still delivering, because everything above is only as true as that.
pub async fn read_state(State(daemon): State<Arc<Daemon>>) -> Json<wire::RigState> {
    Json(rig_state_to_wire(daemon.device.state()))
}

/// Move the API origin — never the accumulator a camera differences.
pub async fn zero_position(
    State(daemon): State<Arc<Daemon>>,
    ApiJson(request): ApiJson<wire::ZeroRequest>,
) -> Json<wire::RigState> {
    let request = zero_request_from_wire(request);
    daemon.device.zero(&request.axes);
    Json(rig_state_to_wire(daemon.device.state()))
}

#[derive(serde::Deserialize)]
pub struct StreamQuery {
    /// What the browser wants, not what the device sends. Default 30.
    rate_hz: Option<f64>,
}

/// The decimated state stream.
///
/// **Decimation is not loss, and this socket keeps them apart.** A consumer
/// cannot tell them apart on its own — `seq` skips either way — so what
/// decimation drops here is counted and added to the next delivered sample's
/// `lost_before`, together with whatever the daemon genuinely never received.
/// A client breaks its line on that field and on nothing else.
///
/// Zone hits are never decimated: dropping one is dropping the event.
pub async fn state_stream(
    State(daemon): State<Arc<Daemon>>,
    Query(query): Query<StreamQuery>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let rate_hz = query.rate_hz.unwrap_or(30.0).clamp(1.0, 1000.0);
    upgrade.on_upgrade(move |socket| follow_state(socket, daemon, rate_hz))
}

async fn follow_state(mut socket: WebSocket, daemon: Arc<Daemon>, rate_hz: f64) {
    let mut frames = daemon.device.subscribe_frames();
    let period = Duration::from_secs_f64(1.0 / rate_hz);
    let mut last_sent: Option<Instant> = None;
    let mut pending_lost: u64 = 0;

    loop {
        let frame = match frames.recv().await {
            Ok(frame) => frame,
            // The broadcast itself fell behind: that is loss, and the next
            // sample says how much rather than the stream quietly shortening.
            Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                pending_lost += missed;
                continue;
            }
            Err(_) => return,
        };

        let frame = match frame {
            StreamFrame::ZoneHit(hit) => StreamFrame::ZoneHit(hit),
            StreamFrame::Sample(mut sample) => {
                let now = Instant::now();
                if last_sent.is_some_and(|at| now.duration_since(at) < period) {
                    pending_lost += sample.lost_before;
                    continue;
                }
                last_sent = Some(now);
                sample.lost_before += pending_lost;
                pending_lost = 0;
                StreamFrame::Sample(sample)
            }
        };

        let Ok(text) = serde_json::to_string(&stream_frame_to_wire(frame)) else { continue };
        if socket.send(Message::Text(text.into())).await.is_err() {
            return;
        }
    }
}

/// Query parameters are a map on the way in; this keeps the import honest for
/// handlers that take none.
#[allow(dead_code)]
type Params = HashMap<String, String>;
