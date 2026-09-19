// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON Schemas for the **files** this daemon reads, and nothing else.
//!
//! This was the OpenAPI document. It described the whole API, generated from
//! the handlers' annotations, and it is gone: the API is authored in
//! `proto/mousewheeld/v1/` now, the handlers speak the types generated from it,
//! and a second generated description of the same interface is the thing
//! `contracts/DAEMON_LAYOUT.md` exists to prevent. What a client reads is the
//! `.proto`, which the daemon serves by gRPC reflection; what a person reads is
//! `docs/reference/api.md`.
//!
//! What survives is the part that was never about the API. A zone set is also a
//! **file** — hand-edited, copied between rigs, reviewed in a pull request —
//! and a file wants the thing an editor understands: a schema at a URL, named
//! by a `$schema` line, so a typo is marked where it is typed rather than
//! refused on save. That schema describes the stored spelling (`"$goal_cm"`,
//! `"displacement"`), which is deliberately not the wire's, so it cannot come
//! from the proto and is generated from the `model` types that own the file.
//!
//! utoipa remains for exactly this, and for nothing that touches a route.

use utoipa::OpenApi;

use crate::model::zone_set::{
    FireRule, OutputAction, Zone, ZoneBound, ZoneMetric, ZoneOutput, ZoneSet, ZoneShape,
};

/// A components-only document. It has no `paths` on purpose: there are no
/// routes to describe here, only the shapes of files.
#[derive(OpenApi)]
#[openapi(components(schemas(
    FireRule, OutputAction, Zone, ZoneBound, ZoneMetric, ZoneOutput, ZoneSet, ZoneShape,
)))]
pub struct FileSchemas;
