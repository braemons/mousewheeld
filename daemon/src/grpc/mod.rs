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

use prost::Message;
use tonic::metadata::{MetadataMap, MetadataValue};
use tonic::{Code, Status};

use crate::daemon_state::Daemon;
use crate::model::{ApiError, Refusal};

/// A refusal, as gRPC says it.
///
/// `ApiError` stays the daemon's own vocabulary — `{error, detail, context}`,
/// raised in `zones/`, `device/` and `model/` — and this is the single place it
/// becomes a status. `ApiError` carries a `Refusal` — this daemon's own six
/// categories — and each maps to exactly one code:
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
/// **The whole refusal also travels as itself.** A status code is a category
/// and a message is a sentence; `error` — `no_device`, `zone_overflow` — is
/// the part a client switches on, and `context` names the thing to change. So
/// `mousewheeld.v1.Error` is encoded into the trailing metadata entry
/// `mousewheeld-error-bin`, which is what makes `proto/mousewheeld/v1/error.proto`
/// the one description of a refusal rather than a shape nothing sends.
///
/// The message keeps `detail (context)` for everything that has not been told
/// about the metadata — grpcurl, a log line, a panic in a test.
pub fn status_of(error: ApiError) -> Status {
    let ApiError { refusal, body } = error;
    // Exhaustive, with no catch-all: a category added to `Refusal` is a
    // compile error here rather than an `Internal` nobody chose.
    let code = match refusal {
        Refusal::NoBoardAttached => Code::Unavailable,
        Refusal::WrongMoment => Code::FailedPrecondition,
        Refusal::BadRequest => Code::InvalidArgument,
        Refusal::NoSuchThing => Code::NotFound,
        Refusal::NotBuiltYet => Code::Unimplemented,
        Refusal::TheDaemonBroke => Code::Internal,
    };
    let message = if body.context.is_empty() {
        body.detail.clone()
    } else {
        format!("{} ({})", body.detail, body.context)
    };
    let mut metadata = MetadataMap::new();
    metadata.insert_bin(
        REFUSAL_METADATA_KEY,
        MetadataValue::from_bytes(&crate::convert::error::error_to_wire(body).encode_to_vec()),
    );
    Status::with_metadata(code, message, metadata)
}

/// Where the typed refusal rides. `-bin` is gRPC's own spelling for a metadata
/// value that is bytes rather than ASCII, and it is what lets `context` hold a
/// zone name with a space or a non-ASCII character in it.
pub const REFUSAL_METADATA_KEY: &str = "mousewheeld-error-bin";

/// The five services, on one daemon.
///
/// Named for what it is rather than for the thing it serves: it is not a rig —
/// a rig has four daemons on it — it is this daemon's implementation of every
/// service `proto/mousewheeld/v1/` declares. One handle, five traits.
#[derive(Clone)]
pub struct DaemonServices {
    pub daemon: Arc<Daemon>,
}

impl DaemonServices {
    pub fn new(daemon: Arc<Daemon>) -> Self {
        Self { daemon }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::error::ApiErrorBody;

    /// The refusal a client reads is the one the daemon raised — all three
    /// fields of it, not a sentence to parse.
    #[test]
    fn a_refusal_travels_as_itself_in_the_metadata() {
        let status = status_of(
            ApiError::not_found("no_such_zone_set", "no zone set named corridor")
                .about("corridor".to_string()),
        );

        assert_eq!(status.code(), Code::NotFound);
        // The message is for everything that has not been told about the
        // metadata, so it carries the context too.
        assert_eq!(status.message(), "no zone set named corridor (corridor)");

        let encoded = status
            .metadata()
            .get_bin(REFUSAL_METADATA_KEY)
            .expect("the typed refusal");
        let refusal = crate::wire::Error::decode(&encoded.to_bytes().unwrap()[..]).unwrap();
        assert_eq!(refusal.error, "no_such_zone_set");
        assert_eq!(refusal.detail, "no zone set named corridor");
        assert_eq!(refusal.context, "corridor");
    }

    /// The codes are chosen for what they mean. A caller retries `unavailable`
    /// unchanged and must never retry the others, so this mapping is the one
    /// thing about a refusal a client is entitled to act on without reading it.
    #[test]
    fn every_category_has_its_own_code() {
        let code = |error: ApiError| status_of(error).code();
        assert_eq!(code(ApiError::no_device()), Code::Unavailable);
        assert_eq!(
            code(ApiError::conflict("nothing_armed", "arm a set first")),
            Code::FailedPrecondition
        );
        assert_eq!(
            code(ApiError::refused("bad_zone_set", "it does not compile")),
            Code::InvalidArgument
        );
        assert_eq!(
            code(ApiError::not_found("no_such_axis", "no axis named ball")),
            Code::NotFound
        );
        assert_eq!(
            code(ApiError::internal("store_unwritable", "read-only")),
            Code::Internal
        );
    }

    /// An empty `context` is not the string "()" — a refusal whose sentence is
    /// the whole story says it once.
    #[test]
    fn a_refusal_with_no_context_is_just_the_sentence() {
        let status = status_of(ApiError {
            refusal: Refusal::NoBoardAttached,
            body: ApiErrorBody {
                error: "no_device".into(),
                detail: "no board attached".into(),
                context: String::new(),
            },
        });
        assert_eq!(status.message(), "no board attached");
    }
}
