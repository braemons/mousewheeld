//! The board, as the API describes it.
//!
//! The first question anybody asks this daemon is whether a board is attached,
//! and the second is which firmware is on it. Both are here, with the capacities
//! that decide whether a zone set will fit — reported by the board at `hello`,
//! because a set that does not fit must be refused at upload naming what
//! overflowed, not discovered on a rig at three in the morning.

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeviceInfo {
    pub connected: bool,
    /// The serial port this daemon owns, from the rig config.
    pub port: String,
    /// `teensy41`, `esp32`, or `simulated`.
    pub board: String,
    pub firmware_version: String,
    /// The wire protocol version the board greeted with.
    pub protocol_version: u32,
    /// The board's clock since it booted — a reset shows up here before it
    /// shows up anywhere else.
    pub uptime_device_us: u64,
    pub capacities: Capacities,
    /// The set written to the board's flash, if any: a standalone rig comes up
    /// with its zones, and the board reports which copy it holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flashed_zone_set: Option<FlashedZoneSet>,
    pub link: LinkStats,
}

/// Compile-time limits, reported at `hello`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Capacities {
    pub n_axes: u8,
    pub max_zones: u16,
    pub max_lines: u8,
    /// The scan the firmware runs its counters and zones at.
    pub scan_hz: u32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlashedZoneSet {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LinkStats {
    /// How many times this daemon has (re)connected to a board. A number that
    /// climbs on its own is a cable.
    pub connection_count: u64,
    /// The last thing the board refused, kept because it is usually the answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// One line of the wire, either direction, uninterpreted.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WireLine {
    pub host_monotonic_ns: u64,
    pub direction: WireDirection,
    pub text: String,
    pub level: WireLevel,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WireDirection {
    /// daemon → board
    Out,
    /// board → daemon
    In,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WireLevel {
    Info,
    /// A line the board refused, or one that failed its CRC. Part of the schema
    /// and rendered by the monitor panel; nothing constructs it until there is
    /// a serial link that can fail.
    #[allow(dead_code)]
    Error,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WireLog {
    pub lines: Vec<WireLine>,
}

/// `GET /api/device/firmware`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FirmwareVersions {
    /// What the attached board is running.
    pub running: String,
    /// What this daemon ships and could flash. Empty until the firmware exists.
    pub available: Vec<String>,
}
