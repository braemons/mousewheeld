// SPDX-License-Identifier: AGPL-3.0-or-later
//! Zone sets, arming, and what is armed.
//!
//! **This is the seam that earns the whole arrangement.** A zone set is stored
//! as a file somebody edits — `"$goal_cm"` for a reference, `"displacement"`
//! for a metric, `null` for an open bound — and carried as a message a client
//! constructs, where those are a `oneof` arm, an enum and an empty message. One
//! is typed by a person and the other by a generator, and neither has to give
//! way because this file is here.
//!
//! Decoding an enum is fallible on purpose. A `oneof` a newer client set, or an
//! enum value this build has never heard of, arrives as `Unspecified` or as an
//! unknown `i32`; turning that into a refusal that names the field is work with
//! exactly one right place to happen, and this is it. The alternative — reading
//! it as the first variant — is a zone armed on a metric nobody asked for.

use std::collections::BTreeMap;

use crate::model::zone_set as m;
use crate::wire;

/// A refusal, in the words the route will repeat.
pub type Refusal = String;

// ------------------------------------------------------------ outward ---

pub fn zone_set_to_wire(set: m::ZoneSet) -> wire::ZoneSet {
    wire::ZoneSet {
        schema_url: set.schema_url,
        zone_set_version: set.zone_set_version,
        zones: set.zones.into_iter().map(zone_to_wire).collect(),
    }
}

fn zone_to_wire(zone: m::Zone) -> wire::Zone {
    wire::Zone {
        name: zone.name,
        shape: match zone.shape {
            m::ZoneShape::Rect => wire::ZoneShape::Rect,
        } as i32,
        axes: zone.axes,
        metric: metric_to_wire(zone.metric) as i32,
        min_cm: zone.min_cm.into_iter().map(bound_to_wire).collect(),
        max_cm: zone.max_cm.into_iter().map(bound_to_wire).collect(),
        wrap_cm: zone.wrap_cm,
        fire: match zone.fire {
            m::FireRule::Once => wire::FireRule::Once,
            m::FireRule::Rearm => wire::FireRule::Rearm,
        } as i32,
        hysteresis_cm: zone.hysteresis_cm,
        level: zone.level,
        output: Some(wire::ZoneOutput {
            line: zone.output.line,
            action: match zone.output.action {
                m::OutputAction::Pulse => wire::OutputAction::Pulse,
                m::OutputAction::Level => wire::OutputAction::Level,
            } as i32,
            ms: zone.output.ms,
        }),
    }
}

/// The file's three-way position becomes three shapes: a value, a reference, or
/// neither arm set — which is the open bound the file spells `null`.
fn bound_to_wire(bound: m::ZoneBound) -> wire::ZoneBound {
    wire::ZoneBound {
        bound: match bound {
            m::ZoneBound::Open => None,
            m::ZoneBound::Value(value) => Some(wire::zone_bound::Bound::Value(value)),
            m::ZoneBound::Reference(name) => Some(wire::zone_bound::Bound::Reference(name)),
        },
    }
}

fn metric_to_wire(metric: m::ZoneMetric) -> wire::ZoneMetric {
    match metric {
        m::ZoneMetric::Displacement => wire::ZoneMetric::Displacement,
        m::ZoneMetric::Distance => wire::ZoneMetric::Distance,
    }
}

pub fn zone_set_names_to_wire(names: m::ZoneSetNames) -> wire::ZoneSetNames {
    wire::ZoneSetNames {
        zone_sets: names.zone_sets,
    }
}

pub fn validation_report_to_wire(report: m::ValidationReport) -> wire::ValidationReport {
    wire::ValidationReport {
        ok: report.ok,
        problem: report.problem,
        zone_count: report.zone_count as u32,
        counts_per_cm: report.counts_per_cm,
        references: report.references,
    }
}

pub fn armed_zones_to_wire(armed: m::ArmedZones) -> wire::ArmedZones {
    wire::ArmedZones {
        zone_set: armed.zone_set,
        zone_set_version: armed.zone_set_version,
        arm_id: armed.arm_id,
        label: armed.label,
        zones: armed
            .zones
            .into_iter()
            .map(|zone| wire::ZoneStatus {
                name: zone.name,
                armed: zone.armed,
                fired: zone.fired,
                inside: zone.inside,
                fired_at_cm: zone.fired_at_cm,
                min_cm: zone.min_cm.into_iter().map(resolved_to_wire).collect(),
                max_cm: zone.max_cm.into_iter().map(resolved_to_wire).collect(),
                wrap_cm: zone.wrap_cm,
                metric: metric_to_wire(zone.metric) as i32,
            })
            .collect(),
    }
}

fn resolved_to_wire(value: Option<f64>) -> wire::ResolvedBound {
    wire::ResolvedBound { value }
}

// ------------------------------------------------------------- inward ---

pub fn zone_set_from_wire(set: wire::ZoneSet) -> Result<m::ZoneSet, Refusal> {
    Ok(m::ZoneSet {
        schema_url: set.schema_url,
        zone_set_version: set.zone_set_version,
        zones: set
            .zones
            .into_iter()
            .map(zone_from_wire)
            .collect::<Result<_, _>>()?,
    })
}

fn zone_from_wire(zone: wire::Zone) -> Result<m::Zone, Refusal> {
    let name = zone.name.clone();
    let output = zone
        .output
        .ok_or_else(|| format!("zone {name:?} has no output"))?;
    Ok(m::Zone {
        shape: match wire::ZoneShape::try_from(zone.shape) {
            Ok(wire::ZoneShape::Rect) => m::ZoneShape::Rect,
            // Unspecified is the omitted case and the unknown case both, and
            // neither may become `Rect` by default: a shape the firmware does
            // not know is refused whole rather than half-evaluated.
            _ => return Err(format!("zone {name:?} has no shape this daemon knows")),
        },
        metric: match wire::ZoneMetric::try_from(zone.metric) {
            // Omitted means displacement, which is what a corridor is.
            Ok(wire::ZoneMetric::Unspecified) => m::ZoneMetric::Displacement,
            Ok(wire::ZoneMetric::Displacement) => m::ZoneMetric::Displacement,
            Ok(wire::ZoneMetric::Distance) => m::ZoneMetric::Distance,
            Err(_) => return Err(format!("zone {name:?} has a metric this daemon does not know")),
        },
        fire: match wire::FireRule::try_from(zone.fire) {
            Ok(wire::FireRule::Once) => m::FireRule::Once,
            Ok(wire::FireRule::Rearm) => m::FireRule::Rearm,
            _ => return Err(format!("zone {name:?} must say fire: once or rearm")),
        },
        min_cm: zone.min_cm.into_iter().map(bound_from_wire).collect(),
        max_cm: zone.max_cm.into_iter().map(bound_from_wire).collect(),
        axes: zone.axes,
        wrap_cm: zone.wrap_cm,
        hysteresis_cm: zone.hysteresis_cm,
        level: zone.level,
        output: m::ZoneOutput {
            action: match wire::OutputAction::try_from(output.action) {
                Ok(wire::OutputAction::Pulse) => m::OutputAction::Pulse,
                Ok(wire::OutputAction::Level) => m::OutputAction::Level,
                _ => return Err(format!("zone {name:?} must say action: pulse or level")),
            },
            line: output.line,
            ms: output.ms,
        },
        name,
    })
}

fn bound_from_wire(bound: wire::ZoneBound) -> m::ZoneBound {
    match bound.bound {
        None => m::ZoneBound::Open,
        Some(wire::zone_bound::Bound::Value(value)) => m::ZoneBound::Value(value),
        Some(wire::zone_bound::Bound::Reference(name)) => m::ZoneBound::Reference(name),
    }
}

pub fn arm_request_from_wire(request: wire::ArmRequest) -> Result<m::ArmRequest, Refusal> {
    Ok(m::ArmRequest {
        zone_set: request.zone_set,
        patch: request.patch.into_iter().collect::<BTreeMap<_, _>>(),
        origin: match wire::ArmOrigin::try_from(request.origin) {
            // Omitted means current, which is what a trial wants and what a
            // caller who never thought about the origin should get.
            Ok(wire::ArmOrigin::Unspecified) => m::ArmOrigin::Current,
            Ok(wire::ArmOrigin::Current) => m::ArmOrigin::Current,
            Ok(wire::ArmOrigin::Absolute) => m::ArmOrigin::Absolute,
            Err(_) => return Err("origin must be current or absolute".to_string()),
        },
        label: request.label,
    })
}
