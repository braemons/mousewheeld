// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the generated types actually put on the wire.
//!
//! The JSON these produce is the API — panels, `curl`, MATLAB and the Python
//! client all read it — and protobuf's JSON mapping has habits that are easy to
//! assume wrongly. Every assertion here was written by printing the bytes
//! first, not by reading the specification.

use mousewheeld::wire;

#[test]
fn a_field_at_its_default_is_still_written() {
    // The mapping's own rule is to omit it, which would mean a wheel sitting
    // still answered `{"name":"wheel"}` and every consumer read `undefined` at
    // exactly the position a rig starts at. `emit_fields()` in tools/protogen
    // turns that off; this is the test that says so.
    let axis = wire::AxisState {
        name: "wheel".into(),
        counts: 0,
        position_cm: 0.0,
        distance_cm: 0.0,
        velocity_cm_s: 0.0,
        device_velocity_cm_s: 0.0,
    };
    let json = serde_json::to_value(&axis).unwrap();
    assert_eq!(json["position_cm"], serde_json::json!(0.0));
    assert_eq!(json["counts"], serde_json::json!("0"));
}

#[test]
fn a_units_name_survives_the_mapping() {
    // Without `json_name` in the .proto every one of these would be camelCase,
    // and `position_cm` — the family's rule that a quantity spells its unit —
    // would not survive contact with the wire.
    let axis = wire::AxisState {
        name: "wheel".into(),
        counts: 41822,
        position_cm: 366.73,
        distance_cm: 366.73,
        velocity_cm_s: 12.5,
        device_velocity_cm_s: 12.0,
    };
    let json = serde_json::to_value(&axis).unwrap();
    for field in [
        "position_cm",
        "distance_cm",
        "velocity_cm_s",
        "device_velocity_cm_s",
    ] {
        assert!(json.get(field).is_some(), "{field} is not on the wire");
    }
}

#[test]
fn a_64_bit_integer_is_a_string() {
    // JSON numbers are doubles, so protobuf's mapping quotes anything 64-bit.
    // Every consumer of `counts`, `seq` or a timestamp has to parse it; this is
    // the fact they have to know, asserted rather than remembered.
    let sample = wire::Sample {
        seq: 41822,
        device_us: 8_000_000,
        host_monotonic_ns: 1_234_567_890_123,
        lost_before: 0,
        axes: vec![],
    };
    let json = serde_json::to_value(&sample).unwrap();
    assert_eq!(json["seq"], serde_json::json!("41822"));
    assert_eq!(json["host_monotonic_ns"], serde_json::json!("1234567890123"));
}

#[test]
fn an_enum_keeps_its_full_name() {
    // `ZONE_METRIC_DISPLACEMENT`, not `DISPLACEMENT`: pbjson would strip the
    // prefix by default, and a Python client generated from the same .proto
    // would send the long form and be refused.
    let status = wire::ZoneStatus {
        name: "goal".into(),
        armed: true,
        fired: false,
        inside: false,
        fired_at_cm: None,
        min_cm: vec![wire::ResolvedBound { value: Some(180.0) }],
        max_cm: vec![wire::ResolvedBound { value: None }],
        wrap_cm: None,
        metric: wire::ZoneMetric::ZoneMetricDisplacement as i32,
    };
    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(json["metric"], serde_json::json!("ZONE_METRIC_DISPLACEMENT"));
    // An open bound is an empty object, and a set one carries its value.
    assert_eq!(json["min_cm"][0]["value"], serde_json::json!(180.0));
    assert_eq!(json["max_cm"][0].get("value"), None);
}

#[test]
fn a_zone_bound_names_which_of_the_three_it_is() {
    // The file writes `["$goal_cm", null]`. Here the three cases are three
    // shapes a reader cannot confuse, which is the whole reason the zone set
    // stopped being an opaque document.
    let zone = wire::Zone {
        name: "goal".into(),
        shape: wire::ZoneShape::ZoneShapeRect as i32,
        axes: vec!["wheel".into()],
        metric: wire::ZoneMetric::ZoneMetricDisplacement as i32,
        min_cm: vec![wire::ZoneBound {
            bound: Some(wire::zone_bound::Bound::Reference("goal_cm".into())),
        }],
        max_cm: vec![wire::ZoneBound { bound: None }],
        wrap_cm: None,
        fire: wire::FireRule::FireRuleOnce as i32,
        hysteresis_cm: None,
        level: false,
        output: Some(wire::ZoneOutput {
            line: "zone_goal".into(),
            action: wire::OutputAction::OutputActionPulse as i32,
            ms: Some(10),
        }),
    };
    let json = serde_json::to_value(&zone).unwrap();
    assert_eq!(json["min_cm"][0]["reference"], serde_json::json!("goal_cm"));
    assert_eq!(json["max_cm"][0].as_object().unwrap().len(), 0);
    assert_eq!(json["output"]["action"], serde_json::json!("OUTPUT_ACTION_PULSE"));
}

#[test]
fn a_stream_frame_names_its_arm() {
    // `{"sample": {…}}`, replacing the `{"type": "sample", …}` the panels read
    // today. A reader destructures one key instead of switching on a string.
    let frame = wire::StreamFrame {
        frame: Some(wire::stream_frame::Frame::ZoneHit(wire::ZoneHitEvent {
            zone: "goal".into(),
            arm_id: 7,
            seq: 100,
            host_monotonic_ns: 5,
            position_cm: 30.005,
        })),
    };
    let json = serde_json::to_value(&frame).unwrap();
    assert_eq!(json["zone_hit"]["zone"], serde_json::json!("goal"));
    assert_eq!(json.get("sample"), None);
}

#[test]
fn an_unknown_field_in_a_request_is_refused() {
    // This API refuses what it does not understand on the way in —
    // `contracts/INTERACTIONS.md` §11 — and pbjson does so by default. The
    // assertion is here because it is a default, and a default can be changed
    // by somebody who did not know it was load-bearing.
    let refused: Result<wire::ArmRequest, _> =
        serde_json::from_str(r#"{"zone_set": "goal", "patchh": {}}"#);
    assert!(refused.is_err(), "an unknown field was accepted");

    let accepted: wire::ArmRequest =
        serde_json::from_str(r#"{"zone_set": "goal", "patch": {"goal_cm": 180}}"#).unwrap();
    assert_eq!(accepted.zone_set, "goal");
    assert_eq!(accepted.patch.get("goal_cm"), Some(&180.0));
}

#[test]
fn an_omitted_origin_means_current() {
    // `ARM_ORIGIN_UNSPECIFIED` is zero so that a caller who never thought about
    // the origin gets the one a trial wants, without the request having to
    // carry it.
    let request: wire::ArmRequest = serde_json::from_str(r#"{"zone_set": "goal"}"#).unwrap();
    assert_eq!(request.origin, wire::ArmOrigin::ArmOriginUnspecified as i32);
}
