//! Zone sets: distances at which a trigger line fires.
//!
//! **Zones are dynamic; the lines they drive are not.** The split is
//! statemachined's, between its graphs and its line map: a zone set names
//! `"line": "zone_goal"` and never a pin, so one set runs on every rig that has
//! a `zone_goal` line and a rewired rig edits one file.
//!
//! **Authored in centimetres, compiled to counts.** The firmware has no
//! floating point on any path whose result crosses the wire, so the host turns
//! every centimetre into an integer interval before the board sees it — against
//! a named calibration, which is why changing the calibration marks compiled
//! sets stale.
//!
//! **A numeric field may be a `"$name"` reference**, resolved from the patch
//! given at arm time. A goal distance drawn per trial is `"min_cm": ["$goal_cm"]`
//! plus `arm {patch: {"goal_cm": 180}}`. Substitution happens here, host-side;
//! the board never sees a `$name`.

use std::collections::BTreeMap;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};
use utoipa::ToSchema;

/// One end of a zone's interval on one axis.
///
/// Three things, and they are not interchangeable: a number, an **open** bound
/// (`null` — "everything past here"), and a **reference** to a per-trial value.
/// A reference that reaches the compiler unresolved is a refusal, never a zero.
#[derive(Debug, Clone, PartialEq)]
pub enum ZoneBound {
    Open,
    Value(f64),
    Reference(String),
}

impl Serialize for ZoneBound {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ZoneBound::Open => serializer.serialize_none(),
            ZoneBound::Value(value) => serializer.serialize_f64(*value),
            ZoneBound::Reference(name) => serializer.serialize_str(&format!("${name}")),
        }
    }
}

impl<'de> Deserialize<'de> for ZoneBound {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Number(f64),
            Text(String),
            Null,
        }
        match Raw::deserialize(deserializer)? {
            Raw::Null => Ok(ZoneBound::Open),
            Raw::Number(value) => Ok(ZoneBound::Value(value)),
            Raw::Text(text) => match text.strip_prefix('$') {
                Some(name) if !name.is_empty() => Ok(ZoneBound::Reference(name.to_string())),
                // A bare string is a typo, not a value: refusing it here is the
                // difference between a zone at 0 cm and an error message.
                _ => Err(de::Error::custom(format!(
                    "a bound is a number, null, or \"$name\" — got {text:?}"
                ))),
            },
        }
    }
}

impl utoipa::PartialSchema for ZoneBound {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use utoipa::openapi::schema::{ObjectBuilder, OneOfBuilder, Type};
        OneOfBuilder::new()
            .item(ObjectBuilder::new().schema_type(Type::Number).description(Some(
                "a distance in centimetres",
            )))
            // The pattern is not decoration: without it the schema accepts
            // `"200"`, which the deserializer below refuses — and a published
            // schema looser than the code it describes is worse than none,
            // because it is believed.
            .item(
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .pattern(Some("^\\$[A-Za-z0-9_]+$"))
                    .description(Some(
                        "\"$name\", resolved from the patch given at arm time",
                    )),
            )
            .item(ObjectBuilder::new().schema_type(Type::Null).description(Some(
                "an open bound: everything past the other end",
            )))
            .description(Some("One end of a zone's interval on one axis."))
            .into()
    }
}
impl utoipa::ToSchema for ZoneBound {}

/// The only shape today, and a discriminator rather than a flag: a future
/// `circle` or `polygon` is a new value, and a shape the firmware does not know
/// is refused whole rather than half-evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ZoneShape {
    Rect,
}

/// Which quantity a zone is compared against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ZoneMetric {
    /// Signed, `counts − origin`. What a corridor position is. The default,
    /// because a corridor is what a wheel usually drives.
    #[default]
    Displacement,
    /// Direction-free odometer, `Σ |Δcounts|`. What "how much did it run" is.
    Distance,
}

/// What happens after a zone fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FireRule {
    /// Disarm after firing.
    Once,
    /// Re-arm once the position has left the zone by `hysteresis_cm`, so
    /// encoder jitter at a boundary is not a pulse train.
    Rearm,
}

/// How the line behaves while the condition holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputAction {
    /// High for at most `ms`.
    Pulse,
    /// High while inside, with the same hysteresis.
    Level,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ZoneOutput {
    /// A **name** from the rig config's line map, never a pin number.
    pub line: String,
    pub action: OutputAction,
    /// Pulse width. Ignored by a `level` output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub name: String,
    pub shape: ZoneShape,
    /// The axes this zone constrains, in the order its bounds are given. One
    /// for a wheel; a 2-D rectangle on a ball is already expressible.
    pub axes: Vec<String>,
    #[serde(default)]
    pub metric: ZoneMetric,
    /// The low end per axis, in the order of `axes`.
    pub min_cm: Vec<ZoneBound>,
    /// The high end per axis. `null` is open.
    pub max_cm: Vec<ZoneBound>,
    /// A circular track: the metric is evaluated modulo this period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_cm: Option<f64>,
    pub fire: FireRule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hysteresis_cm: Option<f64>,
    /// Fire on arming if the animal is already inside. Off by default: a zone
    /// fires on the false→true **edge** of "inside", so arming a zone somebody
    /// is standing in does not fire it.
    #[serde(default)]
    pub level: bool,
    pub output: ZoneOutput,
}

/// A named set, as it is stored and as it is uploaded.
///
/// Declaration order resolves two zones firing in one scan: both still fire,
/// and the order is the order of their `zone_hit` messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ZoneSet {
    /// A URL for this set's JSON Schema, carried so an editor can validate the
    /// file as somebody types it.
    ///
    /// It exists **because** unknown fields are refused: without a field for it
    /// the one line that makes a hand-edited file checkable would itself be a
    /// refusal. Accepted, preserved on write, and ignored by everything else.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
    /// Bumped by whoever edits the set. The board reports the version it holds,
    /// so "is the board running what I think it is" is one comparison.
    pub zone_set_version: u32,
    pub zones: Vec<Zone>,
}

impl ZoneSet {
    /// Every `$name` this set expects at arm time.
    pub fn references(&self) -> Vec<String> {
        let mut names = Vec::new();
        for zone in &self.zones {
            for bound in zone.min_cm.iter().chain(&zone.max_cm) {
                if let ZoneBound::Reference(name) = bound {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
            }
        }
        names
    }

    /// Resolve `$name` bounds from a per-trial patch.
    ///
    /// Host-side and before compilation, so the board never sees a reference.
    /// A name the patch does not carry is a refusal naming the name — never a
    /// default, because a goal distance that silently became zero is a trial
    /// that looks like it ran.
    pub fn substitute(&self, patch: &BTreeMap<String, f64>) -> Result<ZoneSet, String> {
        let mut resolved = self.clone();
        for zone in &mut resolved.zones {
            for bound in zone.min_cm.iter_mut().chain(zone.max_cm.iter_mut()) {
                if let ZoneBound::Reference(name) = bound {
                    match patch.get(name.as_str()) {
                        Some(value) => *bound = ZoneBound::Value(*value),
                        None => return Err(format!("${name} is not in the arm patch")),
                    }
                }
            }
        }
        Ok(resolved)
    }
}

// -------------------------------------------------------------- arming ---

/// `Zones.Arm`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ArmRequest {
    /// A name from the store. Never a slot: the daemon holds the sets, and a
    /// name it does not have is a configuration error the caller can report.
    pub zone_set: String,
    /// Per-trial values for this set's `$name` bounds.
    #[serde(default)]
    pub patch: BTreeMap<String, f64>,
    #[serde(default)]
    pub origin: ArmOrigin,
    /// Opaque text stored with the arm — triald writes `trial 42`. The daemon
    /// stays trial-blind and hands it back unread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Where the metric is measured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArmOrigin {
    /// Displacement and distance are zero at the arm point, which is what a
    /// trial wants.
    #[default]
    Current,
    /// Keep the device origin.
    Absolute,
}

/// What a zone is doing right now.
///
/// The bounds are **as armed**, in centimetres: the `$name`s resolved from the
/// trial's patch and the counts the board is actually comparing against,
/// converted back. Without them a UI can only draw the set as *stored*, which
/// for a parameterised set is a picture of a zone that is not the one running —
/// and "is the reward region where I think it is" is the question the picture
/// exists to answer.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ZoneStatus {
    pub name: String,
    pub armed: bool,
    pub fired: bool,
    pub inside: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fired_at_cm: Option<f64>,
    /// Per axis, in the order the zone names them. `null` is an open bound.
    pub min_cm: Vec<Option<f64>>,
    pub max_cm: Vec<Option<f64>>,
    /// The period the metric is taken modulo, if the zone wraps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_cm: Option<f64>,
    pub metric: ZoneMetric,
}

/// `Zones.ReadArmed` — what is armed, what fired, when.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ArmedZones {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_set_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arm_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub zones: Vec<ZoneStatus>,
}

/// `Zones.ValidateZoneSet` — compile without uploading.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ValidationReport {
    pub ok: bool,
    /// The first thing wrong, in the words the compiler would use. `None` when
    /// it compiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    pub zone_count: usize,
    /// The calibration it was compiled against, so a report is readable a day
    /// later without guessing which one was in force.
    pub counts_per_cm: f64,
    /// The `$name`s this set still needs at arm time.
    pub references: Vec<String>,
}

/// The names in the store.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ZoneSetNames {
    pub zone_sets: Vec<String>,
}
