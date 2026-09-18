//! `/elements/` — the console panels, served by the daemon that owns the wheel.
//!
//! The contract: a console served from somewhere else imports
//! `/elements/mousewheeld.js` by URL and gets this daemon's own version of its
//! panels. That is what stops a console bundling a copy of them and drifting
//! the first time a field changes.
//!
//! Two headers carry the whole arrangement:
//!
//! - **CORS is open** (applied to the whole router): the importing page is not
//!   on this origin, by design.
//! - **`no-cache, must-revalidate`**: a browser holding yesterday's panels
//!   against today's daemon is exactly the failure the contract prevents. A
//!   panel is cheap to fetch and expensive to be wrong about.
//!
//! Embedded in the binary with `rust-embed`, so deployment is one file and a
//! rig cannot end up serving panels from a directory somebody moved. The
//! development page at `/` is a stand-in for a console: it mounts every panel
//! and holds nothing else.

use std::sync::Arc;

use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::daemon_state::Daemon;

#[derive(rust_embed::RustEmbed)]
#[folder = "../client/web/elements"]
struct Elements;

pub fn routes() -> Router<Arc<Daemon>> {
    Router::new()
        .route("/", get(development_page))
        .route("/elements/{*path}", get(serve_element))
}

async fn serve_element(Path(path): Path<String>) -> Response {
    let Some(file) = Elements::get(&path) else {
        return (StatusCode::NOT_FOUND, format!("no element file {path}")).into_response();
    };
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    (
        [
            (header::CONTENT_TYPE, mime.as_ref()),
            (header::CACHE_CONTROL, "no-cache, must-revalidate"),
        ],
        file.data,
    )
        .into_response()
}

/// A page that mounts every panel, for a bench with no console on it.
async fn development_page() -> Html<&'static str> {
    Html(include_str!("development_page.html"))
}
