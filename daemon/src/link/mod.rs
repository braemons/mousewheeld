//! The wire: framing, messages, and the arithmetic that turns a board's numbers
//! into the host's.
//!
//! `docs/reference/protocol.md` is the specification, and the firmware is
//! written against that document rather than against this module.
//!
//! What is *not* here is anything that decides. The link parses, checks and
//! maps; what a sample means to a corridor, and what a zone hit means to a
//! trial, belong to the daemon above it and to the consumers above that.

pub mod continuity;
pub mod framing;
pub mod messages;
pub mod serial;
