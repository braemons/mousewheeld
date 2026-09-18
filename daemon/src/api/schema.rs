//! The zone set's JSON Schema, on its own.
//!
//! **Why separately, when the whole API document already contains it.** A zone
//! set is not only a request body: it is a *file*, in a store, edited by hand,
//! copied between rigs and reviewed in a pull request. What that file wants is
//! the thing an editor understands — a schema at a URL, named by a `$schema`
//! line — so a typo is marked where it is typed rather than refused on save.
//! Handing an editor a 2000-line OpenAPI document and asking it to find
//! `components/schemas/ZoneSet` is not that.
//!
//! It is **extracted from the same generated document**, not written again.
//! There is one description of a zone set in this daemon, it lives on the Rust
//! type, and this is a view of it with the references rewritten from
//! `#/components/schemas/` to `#/$defs/`.

use axum::Json;
use serde_json::{json, Map, Value};

use super::openapi::ApiDoc;
use utoipa::OpenApi;

/// A standalone JSON Schema for one component, with everything it references.
///
/// 2020-12, which is the dialect OpenAPI 3.1 already speaks — so this is a
/// re-rooting, not a translation.
pub fn standalone_schema(root: &str, id: &str, title: &str) -> Value {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap_or_else(|_| json!({}));
    let schemas = document
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    // Walk the references, so the file carries exactly what it needs and no
    // more: a schema that drags in the whole API is one nobody opens.
    let mut wanted = vec![root.to_string()];
    let mut collected = Map::new();
    while let Some(name) = wanted.pop() {
        if collected.contains_key(&name) {
            continue;
        }
        let Some(schema) = schemas.get(&name) else { continue };
        let rewritten: Value = serde_json::from_str(
            &serde_json::to_string(schema)
                .unwrap_or_default()
                .replace("#/components/schemas/", "#/$defs/"),
        )
        .unwrap_or_else(|_| json!({}));
        for reference in references_in(&rewritten) {
            wanted.push(reference);
        }
        collected.insert(name, rewritten);
    }

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": id,
        "title": title,
        "$ref": format!("#/$defs/{root}"),
        "$defs": collected,
    })
}

fn references_in(value: &Value) -> Vec<String> {
    match value {
        Value::Object(members) => members
            .iter()
            .flat_map(|(key, member)| match (key.as_str(), member.as_str()) {
                ("$ref", Some(reference)) => reference
                    .strip_prefix("#/$defs/")
                    .map(|name| vec![name.to_string()])
                    .unwrap_or_default(),
                _ => references_in(member),
            })
            .collect(),
        Value::Array(items) => items.iter().flat_map(references_in).collect(),
        _ => Vec::new(),
    }
}

/// `GET /api/zone-sets/schema` — what a zone set may contain.
#[utoipa::path(
    get, path = "/api/zone-sets/schema", tag = "zones",
    responses((status = 200, description = "JSON Schema 2020-12 for a zone set")),
)]
pub async fn zone_set_schema() -> Json<Value> {
    Json(standalone_schema(
        "ZoneSet",
        "https://braemons.org/mousewheeld/zone-set.schema.json",
        "mousewheeld zone set",
    ))
}
