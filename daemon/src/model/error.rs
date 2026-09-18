//! One refusal shape, for every route.
//!
//! `{error, detail, context}` — a machine-readable code, a sentence, and the
//! thing to change. The third field is the one that earns its keep: a zone set
//! that does not fit names what overflowed, and a UI that can only render
//! "409" throws exactly that away. Both clients read all three.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

pub type ApiResult<T> = Result<T, ApiError>;

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
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn new(status: StatusCode, error: &str, detail: impl Into<String>) -> Self {
        Self {
            status,
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
        Self::new(StatusCode::NOT_FOUND, error, detail)
    }

    /// The request was understood and refused — a zone set that does not
    /// compile, a measurement applied before one was taken.
    pub fn refused(error: &str, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, error, detail)
    }

    /// The daemon is not in a state where this means anything yet.
    pub fn conflict(error: &str, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, error, detail)
    }

    /// No board is attached. Distinct from a refusal: nothing about the request
    /// is wrong, and it will work when the link comes back.
    pub fn no_device() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "no_device",
            "no board is connected — this needs one",
        )
    }

    /// Designed, modelled, and not built. The 2-D ball's routes answer this by
    /// name so that adding it later is filling in rather than redesigning.
    pub fn not_implemented(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_IMPLEMENTED, "not_implemented", detail)
    }

    /// Something the daemon owns went wrong — a store it could not write.
    pub fn internal(error: &str, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error, detail)
    }
}

impl IntoResponse for ApiError {
    /// Through `convert`, like every other answer.
    ///
    /// The two shapes happen to be identical today, which is exactly why this
    /// is worth doing rather than serialising `self.body` directly: the moment
    /// they are not, a refusal would be the one response on this API that had
    /// quietly kept its own spelling.
    fn into_response(self) -> Response {
        let body = crate::convert::error::error_to_wire(self.body);
        (self.status, axum::Json(body)).into_response()
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
