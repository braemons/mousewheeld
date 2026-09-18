// SPDX-License-Identifier: AGPL-3.0-or-later
//! mousewheeld as a library, so its own tests can reach inside it.
//!
//! The binary is `main.rs` and is thin. Everything else lives here, which is
//! what lets `daemon/tests/` construct a wire type, hand it to a route, and
//! destructure the answer — without a serial port, a socket or a board. vstimd
//! splits the same way and for the same reason.
//!
//! The module order is the daemon's own shape: the board at the bottom, the
//! interface at the top, and `convert/` the seam between the two.

pub mod api;
pub mod daemon_state;
pub mod device;
pub mod link;
pub mod model;
pub mod publish;
pub mod wire;
pub mod zones;
