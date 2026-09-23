// SPDX-License-Identifier: AGPL-3.0-or-later
//! Write `daemon/src/wire/` from `proto/`: the API, and under `link/` the
//! board's link.
//!
//! `prost-build` writes the Rust types and `tonic-prost-build` writes the
//! `service` blocks as traits the daemon implements. There is no JSON
//! generator any more: the wire is protobuf's own encoding over HTTP/2, so the
//! five mapping settings that used to be pinned here — and the test file that
//! printed bytes to hold them — are gone with it.
//!
//! The descriptor set is written beside them and **kept**: server reflection
//! serves it, which is how a client discovers this daemon's services without
//! having the `.proto` to hand.
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
    // One argument, and only `make check-proto` passes it: generate somewhere
    // else so the committed tree can be compared against a fresh one. Asking
    // git whether the tree changed cannot answer for a *new* file — an
    // untracked one has no diff — and a staleness check that goes quiet the
    // first time a message is added is worse than none.
    let out_dir = match std::env::args().nth(1) {
        Some(elsewhere) => PathBuf::from(elsewhere),
        None => repository.join("daemon/src/wire"),
    };
    std::fs::create_dir_all(&out_dir)?;

    let files: Vec<PathBuf> = ["calibration", "config", "device", "error", "state", "version", "zones"]
        .iter()
        .map(|name| proto_root.join(format!("mousewheeld/v1/{name}.proto")))
        .collect();

    // The `service` blocks, as traits the daemon implements. An rpc with no
    // implementation is a compile error, which is the job a route checker used
    // to do by reading source code with regexes.
    let service_dir = out_dir.join("service");
    std::fs::create_dir_all(&service_dir)?;
    let mut tonic = tonic_prost_build::Config::new();
    tonic
        .out_dir(&service_dir)
        .extern_path(".google.protobuf", "::prost_types")
        // Point at the messages prost already generated rather than making a
        // second set: without this the trait asks for `service::DeviceInfo`
        // and the daemon has a `wire::DeviceInfo`, which are different types
        // with the same fields.
        .extern_path(".mousewheeld.v1", "crate::wire")
        .compile_well_known_types()
        .btree_map(["."]);
    tonic_prost_build::configure()
        .build_server(true)
        // No Rust client: this daemon calls nobody over gRPC. The Python client
        // is generated in its own tree, from the same files.
        .build_client(false)
        .out_dir(&service_dir)
        .file_descriptor_set_path(out_dir.join("descriptor_for_reflection.bin"))
        .compile_with_config(tonic, &files, &[proto_root.clone()])?;

    let mut config = prost_build::Config::new();
    config
        .out_dir(&out_dir)
        // The well-known types come from `pbjson-types` rather than
        // `prost-types`: only the former carries the JSON mapping, and a
        // `google.protobuf.Struct` that cannot serialise as JSON would be no
        // use to the one route that returns one. `compile_well_known_types`
        // clears prost's own default mapping first, which is otherwise a
        // duplicate of this one and an error.
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        // The prefix is dropped from the Rust variant now — `ZoneMetric::Displacement`
        // rather than `ZoneMetric::ZoneMetricDisplacement`. It was kept only
        // because pbjson would otherwise have put the short form on the JSON
        // wire where the mapping says the long one. There is no JSON wire.
        .btree_map(["."]);

    config.compile_protos(&files, &[&proto_root])?;

    // The board's link: a separate package, so no message is shared with the
    // API by accident, and no JSON mapping or well-known types — a
    // microcontroller sees counts and nothing else. The firmware generates the
    // same file with nanopb (`make firmware-proto`).
    let link_dir = out_dir.join("link");
    std::fs::create_dir_all(&link_dir)?;
    prost_build::Config::new()
        .out_dir(&link_dir)
        .compile_protos(
            &[proto_root.join("mousewheeld/link/v1/link.proto")],
            &[&proto_root],
        )?;

    println!("wrote {}", out_dir.display());
    Ok(())
}
