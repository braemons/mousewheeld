// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where the wheel is.

use crate::model::state as m;
use crate::wire;

pub fn rig_state_to_wire(state: m::RigState) -> wire::RigState {
    wire::RigState {
        axes: state.axes.into_iter().map(axis_state_to_wire).collect(),
        health: Some(wire::LinkHealth {
            seq_gaps: state.health.seq_gaps,
            ring_drops: state.health.ring_drops,
            measured_rate_hz: state.health.measured_rate_hz,
            stale: state.health.stale,
        }),
        link: Some(wire::LinkState {
            connected: state.link.connected,
        }),
    }
}

pub fn axis_state_to_wire(axis: m::AxisState) -> wire::AxisState {
    wire::AxisState {
        name: axis.name,
        counts: axis.counts,
        position_cm: axis.position_cm,
        distance_cm: axis.distance_cm,
        velocity_cm_s: axis.velocity_cm_s,
        device_velocity_cm_s: axis.device_velocity_cm_s,
    }
}

pub fn stream_frame_to_wire(frame: m::StreamFrame) -> wire::StreamFrame {
    wire::StreamFrame {
        frame: Some(match frame {
            m::StreamFrame::Sample(sample) => wire::stream_frame::Frame::Sample(wire::Sample {
                seq: sample.seq,
                device_us: sample.device_us,
                host_monotonic_ns: sample.host_monotonic_ns,
                lost_before: sample.lost_before,
                axes: sample.axes.into_iter().map(axis_state_to_wire).collect(),
            }),
            m::StreamFrame::ZoneHit(hit) => {
                wire::stream_frame::Frame::ZoneHit(wire::ZoneHitEvent {
                    zone: hit.zone,
                    arm_id: hit.arm_id,
                    seq: hit.seq,
                    host_monotonic_ns: hit.host_monotonic_ns,
                    position_cm: hit.position_cm,
                })
            }
        }),
    }
}

pub fn zero_request_from_wire(request: wire::ZeroRequest) -> m::ZeroRequest {
    m::ZeroRequest {
        axes: request.axes,
    }
}

pub fn set_position_request_from_wire(request: wire::SetPositionRequest) -> m::SetPositionRequest {
    m::SetPositionRequest {
        axes: request.axes,
        position_cm: request.position_cm,
    }
}
