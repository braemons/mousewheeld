//! What the wheel is doing, and how well the link is carrying it.
//!
//! Two quantities per axis, and they are not the same question:
//!
//! - **displacement** (`position_cm`) — signed, `counts − origin`. What a
//!   corridor position is.
//! - **distance** (`distance_cm`) — a direction-free odometer. What "how much
//!   did it run" is.
//!
//! And two velocities, recorded separately because they answer different
//! questions and a record that silently kept one could not be checked against
//! the other: what the **board** acted on, and what the **host** computed from
//! the path it received.

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AxisState {
    pub name: String,
    /// The device's 64-bit accumulator. The only unit the firmware knows.
    pub counts: i64,
    /// Signed displacement from the origin.
    pub position_cm: f64,
    /// Direction-free odometer since the origin was last moved.
    pub distance_cm: f64,
    /// Computed by the host from the sample path.
    pub velocity_cm_s: f64,
    /// As the device reported it: counts over a fixed window in its scan, at
    /// scan resolution. Coarser, and the one the analog output and the zone
    /// evaluation were actually done against.
    pub device_velocity_cm_s: f64,
}

/// Whether what the rest of this answer says can be believed.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LinkHealth {
    /// Samples the daemon never received. **A path with a hole in it says so**
    /// rather than reporting a shorter distance that looks complete.
    pub seq_gaps: u64,
    /// The device's own rings overflowed: the firmware telling on itself.
    pub ring_drops: u64,
    /// What the samples are actually arriving at, as opposed to what was asked
    /// for. The two differing is the finding.
    pub measured_rate_hz: f64,
    /// No sample for longer than the staleness window. Everything above is the
    /// last thing that was true.
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RigState {
    pub axes: Vec<AxisState>,
    pub health: LinkHealth,
    pub link: LinkState,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LinkState {
    pub connected: bool,
}

/// One frame of `StateService.WatchState`.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Sample(Sample),
    ZoneHit(ZoneHitEvent),
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Sample {
    /// The device's own sample counter. **Not a loss signal for a consumer**:
    /// this stream is decimated, so `seq` skips by design. See `lost_before`.
    pub seq: u64,
    /// The device clock. Correlating it with the host's is the daemon's job.
    pub device_us: u64,
    /// `CLOCK_MONOTONIC` nanoseconds — the join key with statemachined's trace
    /// and vstimd's vblank timestamps.
    pub host_monotonic_ns: u64,
    /// Samples the daemon **never received** since the previous frame on this
    /// socket, accumulated across the ones decimation dropped. Only the daemon
    /// can tell decimation from loss; a consumer breaks its line on this and on
    /// nothing else.
    pub lost_before: u64,
    pub axes: Vec<AxisState>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ZoneHitEvent {
    pub zone: String,
    pub arm_id: u32,
    pub seq: u64,
    pub host_monotonic_ns: u64,
    pub position_cm: f64,
}

/// `StateService.ZeroPosition`.
///
/// Moves the **API origin** — what displacement, distance and the zones are
/// measured from. Never the accumulator published to a camera: vstimd
/// differences that every frame, and a value that jumps backwards jumps the
/// corridor backwards. Teleporting a corridor is a command to vstimd, from
/// whoever runs the experiment.
#[derive(Debug, Clone, Default, serde::Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ZeroRequest {
    /// Which axes. Empty means all of them.
    #[serde(default)]
    pub axes: Vec<String>,
}

/// `StateService.SetPosition`.
///
/// Sets the **API origin** to a specific position. Unlike `ZeroPosition` which
/// sets the origin to the current position, `SetPosition` allows you to
/// specify any position in centimetres. The origin is adjusted so that
/// `position_cm` matches the requested value.
///
/// This is for vstimd corridor synchronization: when vstimd resets the camera
/// position, mousewheeld can sync the wheel position to match.
#[derive(Debug, Clone, Default, serde::Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SetPositionRequest {
    /// Which axes. Empty means all of them.
    #[serde(default)]
    pub axes: Vec<String>,
    /// Target position in centimetres for each axis.
    #[serde(default)]
    pub position_cm: Vec<f64>,
}
