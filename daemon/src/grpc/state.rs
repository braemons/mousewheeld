// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where the wheel is, and the stream that says so as it happens.

use std::pin::Pin;
use std::time::{Duration, Instant};

use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::convert::state::{rig_state_to_wire, stream_frame_to_wire, zero_request_from_wire};
use crate::model::state::StreamFrame;
use crate::wire;
use crate::wire::service::state_service_server::{StateService, StateServiceServer};

#[tonic::async_trait]
impl StateService for super::Rig {
    async fn read_state(
        &self,
        _request: Request<wire::ReadStateRequest>,
    ) -> Result<Response<wire::RigState>, Status> {
        Ok(Response::new(rig_state_to_wire(self.daemon.device.state())))
    }

    type WatchStateStream =
        Pin<Box<dyn Stream<Item = Result<wire::StreamFrame, Status>> + Send + 'static>>;

    /// The decimated state stream.
    ///
    /// **Decimation is not loss, and this stream keeps them apart.** A consumer
    /// cannot tell them apart on its own — `seq` skips either way — so what
    /// decimation drops here is counted and added to the next delivered
    /// sample's `lost_before`, together with whatever the daemon genuinely
    /// never received. A client breaks its line on that field and nothing else.
    ///
    /// Zone hits are never decimated: dropping one is dropping the event.
    async fn watch_state(
        &self,
        request: Request<wire::WatchStateRequest>,
    ) -> Result<Response<Self::WatchStateStream>, Status> {
        let rate_hz = f64::from(request.into_inner().rate_hz.max(1)).clamp(1.0, 1000.0);
        let period = Duration::from_secs_f64(1.0 / rate_hz);

        let mut last_sent: Option<Instant> = None;
        let mut pending_lost: u64 = 0;

        let frames = BroadcastStream::new(self.daemon.device.subscribe_frames());
        let stream = frames.filter_map(move |frame| match frame {
            // The broadcast itself fell behind: that is loss, and the next
            // sample says how much rather than the stream quietly shortening.
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(missed)) => {
                pending_lost += missed;
                None
            }
            Ok(StreamFrame::ZoneHit(hit)) => {
                Some(Ok(stream_frame_to_wire(StreamFrame::ZoneHit(hit))))
            }
            Ok(StreamFrame::Sample(mut sample)) => {
                let now = Instant::now();
                if last_sent.is_some_and(|at| now.duration_since(at) < period) {
                    pending_lost += sample.lost_before;
                    return None;
                }
                last_sent = Some(now);
                sample.lost_before += pending_lost;
                pending_lost = 0;
                Some(Ok(stream_frame_to_wire(StreamFrame::Sample(sample))))
            }
        });
        Ok(Response::new(Box::pin(stream)))
    }

    async fn zero_position(
        &self,
        request: Request<wire::ZeroRequest>,
    ) -> Result<Response<wire::RigState>, Status> {
        let zero = zero_request_from_wire(request.into_inner());
        self.daemon.device.zero(&zero.axes);
        Ok(Response::new(rig_state_to_wire(self.daemon.device.state())))
    }
}

pub fn server(rig: super::Rig) -> StateServiceServer<super::Rig> {
    StateServiceServer::new(rig)
}
