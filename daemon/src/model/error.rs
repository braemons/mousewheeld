//! One refusal shape, for every route.
//!
//! `{error, detail, context}` — a machine-readable code, a sentence, and the
//! thing to change. The third field is the one that earns its keep: a zone set
//! that does not fit names what overflowed, and a UI that can only render a
//! status code throws exactly that away.
//!
//! **This is the daemon's vocabulary, not the wire's.** `grpc::status_of` is
//! the single place it becomes a `tonic::Status`, and the `Refusal` kept here
//! is how a refusal says which *kind* it is.
//!
//! That field used to be an `axum::http::StatusCode`, because this type grew
//! up behind HTTP routes. Nothing here has answered HTTP since the interface
//! became `proto/mousewheeld/v1/`, so the category was spelled as a number
//! from a protocol the daemon no longer speaks — and `grpc/mod.rs` translated
//! it back. Six names now, and they say what they mean.

use serde::Serialize;
use utoipa::ToSchema;

pub type ApiResult<T> = Result<T, ApiError>;

/// Why a call was refused — the category, in this daemon's own words.
///
/// The *case* within a category is `ApiErrorBody::error`, which is what a
/// client switches on. This is the coarser thing a caller may act on without
/// reading the rest: `Unavailable` is the only one worth retrying unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No such zone set, no such axis.
    NoSuchThing,
    /// Understood and refused: a zone set that does not compile, a rate above
    /// the scan.
    BadRequest,
    /// The daemon is not in a state where this means anything: nothing armed,
    /// nothing measured.
    WrongMoment,
    /// Nothing about the request is wrong and it will work when the link comes
    /// back. The one a caller may retry unchanged.
    NoBoardAttached,
    /// Designed, modelled, and not built. The 2-D ball.
    NotBuiltYet,
    /// Something the daemon owns went wrong.
    TheDaemonBroke,
}

/// A refusal, as the API renders it.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiErrorBody {
    /// A stable, machine-readable code — `no_such_zone_set`, `no_device`.
    pub error: String,
    /// One sentence, for a person.
    pub detail: String,
    /// What to change: a zone set's name, the field that did not fit. May be
    /// empty when the sentence is the whole story.
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub refusal: Refusal,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn new(refusal: Refusal, error: &str, detail: impl Into<String>) -> Self {
        Self {
            refusal,
            body: ApiErrorBody {
                error: error.to_string(),
                detail: detail.into(),
                context: String::new(),
            },
        }
    }

    /// Name what to change. Chained: `ApiError::not_found(..).about(name)`.
    pub fn about(mut self, context: impl Into<String>) -> Self {
        self.body.context = context.into();
        self
    }

    pub fn not_found(error: &str, detail: impl Into<String>) -> Self {
        Self::new(Refusal::NoSuchThing, error, detail)
    }

    /// The request was understood and refused — a zone set that does not
    /// compile, a measurement applied before one was taken.
    pub fn refused(error: &str, detail: impl Into<String>) -> Self {
        Self::new(Refusal::BadRequest, error, detail)
    }

    /// The daemon is not in a state where this means anything yet.
    pub fn conflict(error: &str, detail: impl Into<String>) -> Self {
        Self::new(Refusal::WrongMoment, error, detail)
    }

    /// No board is attached. Distinct from a refusal: nothing about the request
    /// is wrong, and it will work when the link comes back.
    pub fn no_device() -> Self {
        Self::new(
            Refusal::NoBoardAttached,
            "no_device",
            "no board is connected — this needs one",
        )
    }

    /// Designed, modelled, and not built. The 2-D ball's routes answer this by
    /// name so that adding it later is filling in rather than redesigning.
    pub fn not_implemented(detail: impl Into<String>) -> Self {
        Self::new(Refusal::NotBuiltYet, "not_implemented", detail)
    }

    /// Something the daemon owns went wrong — a store it could not write.
    pub fn internal(error: &str, detail: impl Into<String>) -> Self {
        Self::new(Refusal::TheDaemonBroke, error, detail)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.body.error, self.body.detail)?;
        if !self.body.context.is_empty() {
            write!(f, " ({})", self.body.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}
