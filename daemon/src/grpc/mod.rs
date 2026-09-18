// SPDX-License-Identifier: AGPL-3.0-or-later
//! The API, as the five services `proto/mousewheeld/v1/` declares.
//!
//! **An rpc with no implementation here is a compile error.** That is the whole
//! of the route checking this repository used to do by reading source code with
//! regexes, and the reason there is no `check_routes.py` any more.
//!
//! One module per service, named for it. Every method is thin: convert the
//! request, call the daemon, convert the answer — the decisions live in
//! `device/`, `zones/` and `model/`, and a method that grows one has put it in
//! the wrong place.
//!
//! **Refusals are gRPC statuses**, not a hand-rolled mapping to HTTP codes.
//! `unavailable` is the standard spelling of "nothing about the request is
//! wrong and it will work when the link comes back"; `failed_precondition` is
//! "not in a state where this means anything"; `invalid_argument` is a request
//! that cannot be satisfied as written. Each carries the same sentence the
//! daemon would have put in `detail`.

pub mod calibration;
pub mod config;
pub mod device;
pub mod state;
pub mod zones;

use std::sync::Arc;

use axum::http::StatusCode;
use tonic::Status;

use crate::daemon_state::Daemon;
use crate::model::ApiError;

/// A refusal, as gRPC says it.
///
/// `ApiError` stays the daemon's own vocabulary — `{error, detail, context}`,
/// raised in `zones/`, `device/` and `model/` — and this is the single place it
/// becomes a status. The codes are chosen for what they mean rather than by
/// transcribing the HTTP status the error used to carry:
///
/// * `unavailable` — nothing about the request is wrong and it will work when
///   the link comes back. This is `no_device`, and it is the one a caller may
///   retry unchanged.
/// * `failed_precondition` — the daemon is not in a state where this means
///   anything: nothing armed, nothing measured.
/// * `invalid_argument` — understood and refused: a zone set that does not
///   compile, a rate above the scan.
/// * `not_found` — no such zone set, no such axis.
/// * `unimplemented` — designed, modelled, and not built. The 2-D ball.
///
/// `context` — the field or set that needs changing — is appended to the
/// message rather than dropped, because it is usually the answer.
pub fn status_of(error: ApiError) -> Status {
    let ApiError { status, body } = error;
    let message = if body.context.is_empty() {
        body.detail
    } else {
        format!("{} ({})", body.detail, body.context)
    };
    match status {
        StatusCode::SERVICE_UNAVAILABLE => Status::unavailable(message),
        StatusCode::CONFLICT => Status::failed_precondition(message),
        StatusCode::UNPROCESSABLE_ENTITY => Status::invalid_argument(message),
        StatusCode::NOT_FOUND => Status::not_found(message),
        StatusCode::NOT_IMPLEMENTED => Status::unimplemented(message),
        _ => Status::internal(message),
    }
}

/// One handle, shared by every service. They are five traits on one daemon.
#[derive(Clone)]
pub struct Rig {
    pub daemon: Arc<Daemon>,
}

impl Rig {
    pub fn new(daemon: Arc<Daemon>) -> Self {
        Self { daemon }
    }
}
