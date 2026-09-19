//! The zone-set store, and the compiler that turns centimetres into counts.
//!
//! The store is a directory of JSON files, one per named set — the same
//! arrangement as statemachined's graph store and vstimd's scene-configs, and
//! for the same reason: a set is edited, reviewed and copied between rigs by
//! people, so it is a file with a name and not a row in a database.
//!
//! The compiler is where a centimetre stops existing. The firmware has no
//! floating point on any path whose result crosses the wire, so a zone reaches
//! the board as an integer interval of counts, compiled against **a named
//! calibration** — which is why changing the calibration marks every compiled
//! set stale and the next arm recompiles. A set armed under one calibration is
//! never silently reinterpreted under another.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::model::calibration::Calibration;
use crate::model::device::Capacities;
use crate::model::line_map::OutputLine;
use crate::model::zone_set::{FireRule, ZoneBound, ZoneMetric, ZoneSet, ZoneShape};
use crate::model::{ApiError, ApiResult};

/// A directory of named zone sets.
pub struct ZoneSetStore {
    root: PathBuf,
}

impl ZoneSetStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path_for(&self, name: &str) -> ApiResult<PathBuf> {
        // A name is a name, not a path: `../../etc/passwd` is a refusal rather
        // than a traversal, and the refusal names the rule it broke.
        let sane = !name.is_empty()
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !sane {
            return Err(ApiError::refused(
                "bad_zone_set_name",
                "a zone-set name is letters, digits, dashes and underscores, up to 64",
            )
            .about(name.to_string()));
        }
        Ok(self.root.join(format!("{name}.json")))
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension()? != "json" {
                    return None;
                }
                Some(path.file_stem()?.to_string_lossy().into_owned())
            })
            .collect();
        names.sort();
        names
    }

    pub fn read(&self, name: &str) -> ApiResult<ZoneSet> {
        Self::parse(name, &self.read_text(name)?)
    }

    /// The file as it is on disk, unparsed.
    ///
    /// What an editor is editing. A file that does not parse still has to
    /// reach the person who has to fix it.
    pub fn read_text(&self, name: &str) -> ApiResult<String> {
        let path = self.path_for(name)?;
        std::fs::read_to_string(&path).map_err(|_| {
            ApiError::not_found("no_such_zone_set", format!("no zone set named {name}"))
                .about(name.to_string())
        })
    }

    /// One deserializer, for the store and for anything typed at the daemon.
    pub fn parse(name: &str, text: &str) -> ApiResult<ZoneSet> {
        serde_json::from_str(text).map_err(|e| {
            // A file somebody edited by hand, refused with the line and column
            // rather than an empty list of zones.
            ApiError::refused("zone_set_unreadable", format!("{name}: {e}")).about(name.to_string())
        })
    }

    /// The canonical text of a set — what `write` puts on disk.
    pub fn to_text(set: &ZoneSet) -> ApiResult<String> {
        serde_json::to_string_pretty(set)
            .map_err(|e| ApiError::internal("zone_set_unserializable", e.to_string()))
    }

    pub fn write(&self, name: &str, set: &ZoneSet) -> ApiResult<()> {
        let path = self.path_for(name)?;
        std::fs::create_dir_all(&self.root)
            .map_err(|e| ApiError::internal("store_unwritable", format!("{}: {e}", self.root.display())))?;
        std::fs::write(&path, Self::to_text(set)?)
            .map_err(|e| ApiError::internal("store_unwritable", format!("{}: {e}", path.display())))
    }

    /// Put a default set in an empty store, so a new rig has something to look
    /// at in the editor rather than an empty dropdown.
    pub fn seed_if_empty(&self) -> std::io::Result<()> {
        if !self.names().is_empty() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.root)?;
        let example = include_str!("example_goal_zone_set.json");
        std::fs::write(self.root.join("goal.json"), example)
    }
}

// ------------------------------------------------------------- compiled ---

/// One zone as the board holds it: integer counts, and nothing to interpret.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledZone {
    pub name: String,
    pub axis_indices: Vec<usize>,
    pub metric: ZoneMetric,
    /// Per axis, in the order of `axis_indices`. `None` is an open bound.
    pub min_counts: Vec<Option<i64>>,
    pub max_counts: Vec<Option<i64>>,
    pub wrap_counts: Option<i64>,
    pub fire: FireRule,
    pub hysteresis_counts: i64,
    pub level: bool,
    pub line_index: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledZoneSet {
    pub name: String,
    pub version: u32,
    pub zones: Vec<CompiledZone>,
    /// The calibration this was compiled against, so a later arm can tell
    /// whether it is still valid without guessing.
    pub counts_per_cm: f64,
}

/// Turn one authored set into what the board evaluates.
///
/// Every refusal here names what to change, because this is the last point at
/// which a person can fix it: past this, a wrong zone is a TTL that fires in
/// the wrong place with nothing on screen to say so.
pub fn compile(
    name: &str,
    set: &ZoneSet,
    patch: &BTreeMap<String, f64>,
    calibration: &Calibration,
    lines: &[OutputLine],
    capacities: &Capacities,
) -> ApiResult<CompiledZoneSet> {
    let resolved = set
        .substitute(patch)
        .map_err(|problem| ApiError::refused("unresolved_reference", problem).about(name.to_string()))?;

    if resolved.zones.is_empty() {
        return Err(ApiError::refused("empty_zone_set", "a zone set needs at least one zone")
            .about(name.to_string()));
    }
    if resolved.zones.len() > usize::from(capacities.max_zones) {
        return Err(ApiError::refused(
            "zone_set_too_large",
            format!(
                "{} zones, and the board holds {}",
                resolved.zones.len(),
                capacities.max_zones
            ),
        )
        .about(name.to_string()));
    }

    let mut zones = Vec::with_capacity(resolved.zones.len());
    for zone in &resolved.zones {
        let about = format!("{name}.{}", zone.name);
        if zone.shape != ZoneShape::Rect {
            return Err(ApiError::refused("bad_shape", "rect is the only shape today").about(about));
        }
        if zone.axes.is_empty() {
            return Err(ApiError::refused("no_axis", "a zone names at least one axis").about(about));
        }
        if zone.min_cm.len() != zone.axes.len() || zone.max_cm.len() != zone.axes.len() {
            return Err(ApiError::refused(
                "bound_count_mismatch",
                format!(
                    "{} axes but {} min and {} max bounds",
                    zone.axes.len(),
                    zone.min_cm.len(),
                    zone.max_cm.len()
                ),
            )
            .about(about));
        }

        let line = lines.iter().find(|line| line.name == zone.output.line).ok_or_else(|| {
            ApiError::refused(
                "no_such_line",
                format!(
                    "no output line named {} on this rig — the line map is the rig config",
                    zone.output.line
                ),
            )
            .about(about.clone())
        })?;

        let mut axis_indices = Vec::new();
        let mut min_counts = Vec::new();
        let mut max_counts = Vec::new();
        for (position, axis_name) in zone.axes.iter().enumerate() {
            let index = calibration
                .axes
                .iter()
                .position(|axis| &axis.name == axis_name)
                .ok_or_else(|| {
                    ApiError::refused("no_such_axis", format!("no axis named {axis_name}"))
                        .about(about.clone())
                })?;
            let axis = &calibration.axes[index];
            let low = bound_to_counts(&zone.min_cm[position], axis, &about)?;
            let high = bound_to_counts(&zone.max_cm[position], axis, &about)?;
            if let (Some(low), Some(high)) = (low, high) {
                if low > high {
                    return Err(ApiError::refused(
                        "inverted_interval",
                        "the low bound is above the high one",
                    )
                    .about(about.clone()));
                }
            }
            axis_indices.push(index);
            min_counts.push(low);
            max_counts.push(high);
        }

        let counts_per_cm = calibration
            .axis(&zone.axes[0])
            .map(|axis| axis.counts_per_cm)
            .unwrap_or(1.0);

        // Hysteresis is what keeps encoder jitter at a boundary from being a
        // pulse train, so a rearming zone without one is a refusal rather than
        // a default somebody has to discover.
        if zone.fire == FireRule::Rearm && zone.hysteresis_cm.unwrap_or(0.0) <= 0.0 {
            return Err(ApiError::refused(
                "no_hysteresis",
                "a rearming zone needs hysteresis_cm above zero, or jitter at its edge is a pulse train",
            )
            .about(about));
        }

        zones.push(CompiledZone {
            name: zone.name.clone(),
            axis_indices,
            metric: zone.metric,
            min_counts,
            max_counts,
            wrap_counts: zone.wrap_cm.map(|cm| (cm * counts_per_cm).round() as i64),
            fire: zone.fire,
            hysteresis_counts: (zone.hysteresis_cm.unwrap_or(0.0) * counts_per_cm).round() as i64,
            level: zone.level,
            line_index: line.index,
        });
    }

    Ok(CompiledZoneSet {
        name: name.to_string(),
        version: resolved.zone_set_version,
        zones,
        counts_per_cm: calibration.axes.first().map(|axis| axis.counts_per_cm).unwrap_or(1.0),
    })
}

fn bound_to_counts(
    bound: &ZoneBound,
    axis: &crate::model::calibration::AxisCalibration,
    about: &str,
) -> ApiResult<Option<i64>> {
    match bound {
        ZoneBound::Open => Ok(None),
        // One place converts a centimetre into counts, and it is the axis'
        // own method — including the sign, so an inverted encoder is inverted
        // in exactly one place.
        ZoneBound::Value(cm) => Ok(Some(axis.cm_to_counts(*cm).round() as i64)),
        // Caught earlier by `substitute`; refused again here rather than
        // compiled to a zero, because a goal distance that silently became 0 cm
        // is a trial that looks like it ran.
        ZoneBound::Reference(name) => Err(ApiError::refused(
            "unresolved_reference",
            format!("${name} reached the compiler — give it in the arm patch"),
        )
        .about(about.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::calibration::AxisCalibration;
    use crate::model::zone_set::{OutputAction, Zone, ZoneOutput};

    fn calibration() -> Calibration {
        Calibration {
            axes: vec![AxisCalibration {
                name: "wheel".into(),
                counts_per_cm: 100.0,
                counts_per_rev: Some(4096),
                diameter_cm: Some(13.0),
                invert: false,
                measured_at: None,
            }],
            ball: None,
        }
    }

    fn lines() -> Vec<OutputLine> {
        vec![OutputLine { name: "zone_goal".into(), index: 0, pin: 5, safe_high: false }]
    }

    fn capacities() -> Capacities {
        Capacities { n_axes: 2, max_zones: 16, max_lines: 8, scan_hz: 5000 }
    }

    fn zone(min: ZoneBound, max: ZoneBound) -> Zone {
        Zone {
            name: "goal".into(),
            shape: ZoneShape::Rect,
            axes: vec!["wheel".into()],
            metric: ZoneMetric::Displacement,
            min_cm: vec![min],
            max_cm: vec![max],
            wrap_cm: None,
            fire: FireRule::Once,
            hysteresis_cm: None,
            level: false,
            output: ZoneOutput { line: "zone_goal".into(), action: OutputAction::Pulse, ms: Some(10) },
        }
    }

    fn set(zone: Zone) -> ZoneSet {
        ZoneSet { schema_url: None, zone_set_version: 1, zones: vec![zone] }
    }

    #[test]
    fn centimetres_become_counts_against_the_calibration() {
        let compiled = compile(
            "goal",
            &set(zone(ZoneBound::Value(200.0), ZoneBound::Open)),
            &BTreeMap::new(),
            &calibration(),
            &lines(),
            &capacities(),
        )
        .unwrap();
        assert_eq!(compiled.zones[0].min_counts, vec![Some(20_000)]);
        assert_eq!(compiled.zones[0].max_counts, vec![None], "null stays open, never zero");
    }

    #[test]
    fn a_reference_is_resolved_from_the_patch_and_refused_without_one() {
        let authored = set(zone(ZoneBound::Reference("goal_cm".into()), ZoneBound::Open));
        let patch = BTreeMap::from([("goal_cm".to_string(), 180.0)]);
        let compiled = compile("goal", &authored, &patch, &calibration(), &lines(), &capacities()).unwrap();
        assert_eq!(compiled.zones[0].min_counts, vec![Some(18_000)]);

        let refused = compile("goal", &authored, &BTreeMap::new(), &calibration(), &lines(), &capacities())
            .unwrap_err();
        assert_eq!(refused.body.error, "unresolved_reference");
        assert!(refused.body.detail.contains("goal_cm"), "{}", refused.body.detail);
    }

    #[test]
    fn a_line_this_rig_does_not_have_is_refused_by_name() {
        let mut one = zone(ZoneBound::Value(1.0), ZoneBound::Open);
        one.output.line = "zone_elsewhere".into();
        let refused = compile("goal", &set(one), &BTreeMap::new(), &calibration(), &lines(), &capacities())
            .unwrap_err();
        assert_eq!(refused.body.error, "no_such_line");
        assert!(refused.body.detail.contains("zone_elsewhere"));
    }

    #[test]
    fn a_rearming_zone_without_hysteresis_is_refused() {
        let mut one = zone(ZoneBound::Value(10.0), ZoneBound::Value(20.0));
        one.fire = FireRule::Rearm;
        let refused = compile("goal", &set(one), &BTreeMap::new(), &calibration(), &lines(), &capacities())
            .unwrap_err();
        assert_eq!(refused.body.error, "no_hysteresis");
    }

    #[test]
    fn an_inverted_interval_is_refused_rather_than_silently_empty() {
        let refused = compile(
            "goal",
            &set(zone(ZoneBound::Value(50.0), ZoneBound::Value(10.0))),
            &BTreeMap::new(),
            &calibration(),
            &lines(),
            &capacities(),
        )
        .unwrap_err();
        assert_eq!(refused.body.error, "inverted_interval");
    }

    #[test]
    fn a_name_that_is_a_path_is_refused() {
        let store = ZoneSetStore::new(PathBuf::from("/tmp/mousewheeld-test"));
        let refused = store.read("../../etc/passwd").unwrap_err();
        assert_eq!(refused.body.error, "bad_zone_set_name");
    }
}
