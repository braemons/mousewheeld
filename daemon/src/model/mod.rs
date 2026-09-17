//! The public surface, as types.
//!
//! **The wire shape is the file shape.** Every type here is at once the HTTP
//! body, the thing in the store or the rig config, and the runtime value —
//! there are no DTOs, which is vstimd's rule and holds for the same reason: a
//! second copy of a schema is a second place for it to be wrong.
//!
//! Three conventions, all of them load-bearing:
//!
//! - **A quantity with a unit spells it in the name** — `position_cm`,
//!   `counts_per_cm`, `velocity_cm_s`, `rate_hz`. The same name travels through
//!   the proto, the config, both clients and the console, so it is spelled once.
//! - **Unknown fields are refused** (`deny_unknown_fields`), so a typo in a zone
//!   set is an error rather than a zone with a default.
//! - **Counts are the device's; centimetres are everybody else's.** The firmware
//!   knows counts and nothing else; every number that leaves this daemon is in
//!   centimetres, because the consumers — a corridor, a trial record, a
//!   distance-triggered line — all need the same unit and none of them can read
//!   this daemon's calibration.

pub mod calibration;
pub mod config;
pub mod device;
pub mod error;
pub mod line_map;
pub mod state;
pub mod zone_set;

pub use error::{ApiError, ApiResult};
