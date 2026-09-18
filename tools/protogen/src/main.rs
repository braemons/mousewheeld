// SPDX-License-Identifier: AGPL-3.0-or-later
//! Write `daemon/src/wire/` from `proto/`.
//!
//! Two generators, and they do different halves. `prost-build` writes the
//! Rust types from the schema; `pbjson-build` writes their `serde` impls
//! against **protobuf's JSON mapping**, which is the part that matters here —
//! this family puts JSON on every control plane, so the mapping is the wire
//! format rather than a sideline, and having it generated is what keeps the
//! bytes and the `.proto` from being two descriptions of one thing.
//!
//! The output is committed. That is a deliberate inversion of the usual
//! `OUT_DIR` arrangement: a reviewer sees an interface change in a diff, a
//! checkout builds without protoc, and anybody reading the repository can read
//! the types instead of inferring them from a build directory.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("tools/protogen lives two levels below the repository root")
        .to_path_buf();

    let proto_root = repository.join("proto");
    let out_dir = repository.join("daemon/src/wire");
    std::fs::create_dir_all(&out_dir)?;

    let files: Vec<PathBuf> = ["calibration", "config", "device", "error", "state", "version", "zones"]
        .iter()
        .map(|name| proto_root.join(format!("mousewheeld/v1/{name}.proto")))
        .collect();

    // The descriptor set is what pbjson-build reads, and prost writes it as a
    // side effect of compiling — so the two generators are guaranteed to be
    // looking at the same schema rather than at two parses of it.
    let descriptor_path = out_dir.join("descriptor.bin");

    let mut config = prost_build::Config::new();
    config
        .out_dir(&out_dir)
        .file_descriptor_set_path(&descriptor_path)
        // The well-known types come from `pbjson-types` rather than
        // `prost-types`: only the former carries the JSON mapping, and a
        // `google.protobuf.Struct` that cannot serialise as JSON would be no
        // use to the one route that returns one. `compile_well_known_types`
        // clears prost's own default mapping first, which is otherwise a
        // duplicate of this one and an error.
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        // Keep the enum's name on its variants: `ZoneMetric::ZoneMetricDisplacement`.
        //
        // Verbose in Rust, and only `convert/` ever types it. The alternative is
        // worse than verbose: prost strips the prefix from the Rust variant and
        // pbjson would then strip it from the JSON too, putting `"DISPLACEMENT"`
        // on the wire where protobuf's JSON mapping says `"ZONE_METRIC_DISPLACEMENT"`.
        // A Python client generated from this same file would emit the long form
        // and be refused — which is precisely the class of disagreement authoring
        // the interface in one place is meant to end.
        .retain_enum_prefix()
        .btree_map(["."]);

    config.compile_protos(&files, &[&proto_root])?;

    let descriptor = std::fs::read(&descriptor_path)?;
    pbjson_build::Builder::new()
        .out_dir(&out_dir)
        // **Emit every field, including the ones at their default.**
        //
        // protobuf's JSON mapping omits a field holding its default, so a wheel
        // sitting still would answer `{"name": "wheel"}` — no `position_cm`, no
        // `counts` — and every consumer would read `undefined` at exactly the
        // position the rig starts at. A reading of zero is a reading.
        .emit_fields()
        // Spell an enum value as the `.proto` spells it. pbjson strips the
        // prefix by default, which is friendlier and is *not* what protobuf's
        // JSON mapping says, so a Python or JavaScript client generated from
        // the same file would disagree with this one about the same byte.
        .retain_enum_prefix()
        // Unknown fields are refused, which is pbjson's default and this API's
        // documented behaviour — `contracts/INTERACTIONS.md` §11: requests
        // refuse what they do not understand, responses ignore it. This daemon
        // parses requests.
        .register_descriptors(&descriptor)?
        .build(&[".mousewheeld.v1"])?;

    // Nothing reads it after this, and a binary blob in a source tree is a file
    // no reviewer can check.
    std::fs::remove_file(&descriptor_path)?;

    println!("wrote {}", out_dir.display());
    Ok(())
}
