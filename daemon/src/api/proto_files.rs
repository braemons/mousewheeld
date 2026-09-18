// SPDX-License-Identifier: AGPL-3.0-or-later
//! The interface, served as itself.
//!
//! **"What do I read for mousewheeld's public API?" now has a URL.** Not a
//! generated document about the API — the API: the same `.proto` files a
//! reviewer reads in the repository, embedded in the binary, so a rig answers
//! the question without anybody knowing which release it runs or where its
//! source is checked out.
//!
//! This replaces `/api/openapi.json`, and it is a better artifact than the one
//! it replaces for the reason `contracts/DAEMON_LAYOUT.md` gives: the OpenAPI
//! document was generated *from* the code, so it could only ever restate what
//! the code happened to do. These files are the thing the code is generated
//! from. A client that wants types runs its own generator over them; a person
//! who wants prose reads `docs/reference/api.md`.
//!
//! Text, deliberately, rather than a `FileDescriptorSet`. A descriptor is what
//! a code generator would rather have, and it is a binary blob nobody can read
//! in a browser or check in a diff — and a generator can produce one from these
//! in a line of `protoc`.

use std::sync::Arc;

use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};

use crate::daemon_state::Daemon;

#[derive(rust_embed::RustEmbed)]
#[folder = "../proto"]
struct Protos;

pub fn routes() -> Router<Arc<Daemon>> {
    Router::new()
        .route("/api/proto", get(list_files))
        .route("/api/proto/{*path}", get(serve_file))
}

/// Every file, so a generator can fetch the set without knowing the names.
async fn list_files() -> Json<serde_json::Value> {
    let mut files: Vec<String> = Protos::iter().map(|name| name.to_string()).collect();
    files.sort();
    Json(serde_json::json!({
        "package": "mousewheeld.v1",
        "files": files,
    }))
}

async fn serve_file(Path(path): Path<String>) -> Response {
    let Some(file) = Protos::get(&path) else {
        return (StatusCode::NOT_FOUND, format!("no proto file {path}")).into_response();
    };
    (
        // `text/plain` so a browser shows it. There is no registered media type
        // for protobuf source, and inventing one would only make it download.
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        file.data,
    )
        .into_response()
}
