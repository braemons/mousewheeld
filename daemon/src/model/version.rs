//! What this daemon is, and what it speaks.
//!
//! A console shows three daemons' panels beside each other and needs to know
//! what it is showing; a client that wants a field added last month needs to
//! know whether it is there. Both are answered by **advertising** a version,
//! never by gating on one — a version check between daemons turns every upgrade
//! into a coordinated one, on a rig whose whole arrangement is that each daemon
//! runs alone (`contracts/INTERACTIONS.md` §11).
//!
//! Three numbers, and they are not the same number:
//!
//! - **`daemon`** — this release. Changes when anything here does.
//! - **`api`** — the interface contract's major version. What a client cares
//!   about, and deliberately slower-moving than the release.
//! - **`device_protocol`** — the wire to the board: what this daemon sends,
//!   the oldest it will talk to, and what the attached board actually said.
//!
//! The same numbers go in the mDNS TXT record, which is the one place the
//! family's four daemons can agree on despite speaking three transports.

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VersionReport {
    /// This daemon's release, from the build.
    pub daemon: String,
    /// The API contract's major version. Within it, changes are additive:
    /// fields, routes and enum values may appear; nothing is removed, renamed
    /// or given a new meaning.
    pub api: u32,
    pub device_protocol: DeviceProtocol,
    /// The `vinput` segment layout this daemon writes — the one interface in
    /// the family where a mismatch is bytes reinterpreted rather than a field
    /// gone missing, and therefore the one that is checked rather than
    /// advertised. The reader refuses to map a segment whose version is not its
    /// own; this is here so a person can see why before it happens.
    pub vinput_layout: u32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeviceProtocol {
    /// What this daemon sends in `hello`.
    pub speaks: u32,
    /// The oldest it will talk to. Below this a board is refused by name rather
    /// than half-understood.
    pub floor: u32,
    /// What the attached board greeted with, if one has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board: Option<u32>,
}
