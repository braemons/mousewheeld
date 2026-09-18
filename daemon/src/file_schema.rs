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
//! It is **extracted from the generated document**, not written again. There is
//! one description of a zone set *file* in this daemon, it lives on the `model`
//! type that reads and writes it, and this is a view of that with the
//! references rewritten from `#/components/schemas/` to `#/$defs/`.
//!
//! Note which description. The wire's `ZoneSet` is in the proto and differs on
//! purpose — `{"reference": "goal_cm"}` where the file says `"$goal_cm"`. This
//! schema is the file's, because an editor has the file open.

use serde_json::{json, Map, Value};

use crate::file_schema_types::FileSchemas;
use utoipa::OpenApi;

/// A standalone JSON Schema for one component, with everything it references.
///
/// 2020-12, which is the dialect OpenAPI 3.1 already speaks — so this is a
/// re-rooting, not a translation.
pub fn standalone_schema(root: &str, id: &str, title: &str) -> Value {
    let document = serde_json::to_value(FileSchemas::openapi()).unwrap_or_else(|_| json!({}));
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

/// What a zone set file may contain.
///
/// Printed by `mousewheeld schema` and committed to
/// `docs/reference/zone-set.schema.json`. A subcommand rather than a route:
/// this is a build artifact for an editor and for CI, neither of which should
/// have to start a daemon to get it.
pub fn zone_set_schema() -> Value {
    standalone_schema(
        "ZoneSet",
        "https://braemons.org/mousewheeld/zone-set.schema.json",
        "mousewheeld zone set",
    )
}

/// The same schema as a protobuf `Struct`, for the rpc that serves it.
///
/// A hand-written walk because there is no longer a JSON mapping to borrow
/// one from: `prost-types` models arbitrary JSON and `serde_json` produces it,
/// and nothing in between knows about both. Twenty lines, and the only place
/// in this daemon where arbitrary JSON crosses the wire.
pub fn as_protobuf_struct(value: &Value) -> prost_types::Struct {
    match structured(value).kind {
        Some(prost_types::value::Kind::StructValue(structure)) => structure,
        // A JSON Schema is an object at the top by definition; anything else is
        // this function being called on the wrong thing.
        _ => prost_types::Struct::default(),
    }
}

fn structured(value: &Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match value {
        Value::Null => Kind::NullValue(0),
        Value::Bool(flag) => Kind::BoolValue(*flag),
        // Every JSON number becomes a double, which is what protobuf's own
        // `Value` offers and what JSON numbers are anyway.
        Value::Number(number) => Kind::NumberValue(number.as_f64().unwrap_or(0.0)),
        Value::String(text) => Kind::StringValue(text.clone()),
        Value::Array(items) => Kind::ListValue(prost_types::ListValue {
            values: items.iter().map(structured).collect(),
        }),
        Value::Object(fields) => Kind::StructValue(prost_types::Struct {
            fields: fields
                .iter()
                .map(|(name, field)| (name.clone(), structured(field)))
                .collect(),
        }),
    };
    prost_types::Value { kind: Some(kind) }
}
