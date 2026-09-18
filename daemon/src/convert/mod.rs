// SPDX-License-Identifier: AGPL-3.0-or-later
//! The seam between what this daemon thinks in and what it says.
//!
//! **Every conversion between `model` and `wire` is here, and nowhere else.**
//! Not in a handler, not in `model`, not in `device`. A conversion that lives
//! beside the code that needed it is a second, quieter description of the
//! interface, and the point of authoring the interface in one file is that
//! there is no second one.
//!
//! Names are `X_to_wire` and `X_from_wire`, in that direction and nothing else,
//! so the direction of a call is readable rather than looked up. vstimd's
//! `ipc/convert` is the same arrangement with `_from_proto` / `_to_proto`, and
//! reading one should teach you the other.
//!
//! **Why a seam at all, when the two shapes are mostly the same.** Because
//! where they differ, they differ for reasons that are the whole argument:
//!
//! - a zone set on disk says `"$goal_cm"` and a `ZoneBound` on the wire says
//!   `{"reference": "goal_cm"}`, because one is typed by a person and the other
//!   by a client;
//! - `model` holds runtime state — a measurement in progress, a compiled set —
//!   that has no business on a wire;
//! - a wire enum is an `i32` that may hold a number this build has never heard
//!   of, and turning that into a refusal is work with exactly one right place
//!   to happen.
//!
//! An `Unspecified` arm decodes anything unrecognised, so a newer client's
//! value is refused by name rather than silently read as the first variant.

pub mod calibration;
pub mod config;
pub mod device;
pub mod error;
pub mod state;
pub mod zones;
