//! The store, the compiler's verdict, and arming.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;

use crate::api::ApiJson;
use crate::daemon_state::Daemon;
use crate::model::zone_set::{
    ArmRequest, ArmedZones, ValidationReport, ZoneSet, ZoneSetNames,
};
use crate::model::device::DeviceInfo;
use crate::model::{ApiError, ApiResult};
use crate::zones::compile;

#[utoipa::path(get, path = "/api/zone-sets", tag = "zones",
    responses((status = 200, body = ZoneSetNames)))]
pub async fn list_zone_sets(State(daemon): State<Arc<Daemon>>) -> Json<ZoneSetNames> {
    Json(ZoneSetNames {
        zone_sets: daemon.store.names(),
    })
}

#[utoipa::path(get, path = "/api/zone-sets/{name}", tag = "zones",
    params(("name" = String, Path, description = "the set's name in the store")),
    responses((status = 200, body = ZoneSet), (status = 404, description = "no such set")))]
pub async fn read_zone_set(
    State(daemon): State<Arc<Daemon>>,
    Path(name): Path<String>,
) -> ApiResult<Json<ZoneSet>> {
    Ok(Json(daemon.store.read(&name)?))
}

/// Write a set to the store. Stored as authored — `$name` references intact,
/// centimetres intact — because the store is what a person edits and copies
/// between rigs. Compilation happens at arm.
#[utoipa::path(put, path = "/api/zone-sets/{name}", tag = "zones",
    request_body = ZoneSet,
    params(("name" = String, Path, description = "the set's name in the store")),
    responses((status = 200, body = ZoneSet), (status = 422, description = "refused")))]
pub async fn replace_zone_set(
    State(daemon): State<Arc<Daemon>>,
    Path(name): Path<String>,
    ApiJson(set): ApiJson<ZoneSet>,
) -> ApiResult<Json<ZoneSet>> {
    daemon.store.write(&name, &set)?;
    Ok(Json(set))
}

/// Compile a set that is **not in the store**, so an editor can say what is
/// wrong with what somebody is typing.
///
/// The counterpart below validates what is *stored* — "is that set still good
/// after the calibration changed". This one validates a draft, and the
/// difference matters: validating an edit used to mean saving it first, which
/// put a set nobody had approved into the store on the way to finding out it
/// was wrong.
///
/// The body goes through the same extractor every other request does, so a
/// misspelled field is refused by the real deserializer with the real message —
/// there is no second, looser description of a zone set anywhere in this path.
#[utoipa::path(post, path = "/api/zone-sets/validate", tag = "zones",
    request_body = ZoneSet,
    responses((status = 200, body = ValidationReport), (status = 422, description = "not a zone set")))]
pub async fn validate_body(
    State(daemon): State<Arc<Daemon>>,
    ApiJson(set): ApiJson<ZoneSet>,
) -> Json<ValidationReport> {
    Json(report_on(&daemon, "draft", &set))
}

/// Compile against the current calibration and the board's capacities, and
/// upload nothing.
///
/// This is what lets a zone set be wrong in an editor rather than at arm time,
/// which is the difference between a message on screen and a TTL that fires in
/// the wrong place in the middle of a session.
#[utoipa::path(post, path = "/api/zone-sets/{name}/validate", tag = "zones",
    params(("name" = String, Path, description = "the set's name in the store")),
    responses((status = 200, body = ValidationReport)))]
pub async fn validate_zone_set(
    State(daemon): State<Arc<Daemon>>,
    Path(name): Path<String>,
) -> ApiResult<Json<ValidationReport>> {
    let set = daemon.store.read(&name)?;
    Ok(Json(report_on(&daemon, &name, &set)))
}

/// What the compiler makes of one set, against what is in force right now.
fn report_on(daemon: &Arc<Daemon>, name: &str, set: &ZoneSet) -> ValidationReport {
    let calibration = daemon.calibration();
    let lines = daemon.config.lock().unwrap().lines.clone();
    let counts_per_cm = calibration.axes.first().map(|axis| axis.counts_per_cm).unwrap_or(1.0);

    // Compiled with the references standing in for numbers: a set that needs a
    // patch is not broken, it is parameterised, and the report names what it
    // needs rather than refusing it.
    let references = set.references();
    let patch = references.iter().map(|name| (name.clone(), 0.0)).collect();
    let problem = compile(name, set, &patch, &calibration, &lines, &daemon.device.capacities()).err();

    ValidationReport {
        ok: problem.is_none(),
        problem: problem.map(|error| error.to_string()),
        zone_count: set.zones.len(),
        counts_per_cm,
        references,
    }
}

#[utoipa::path(get, path = "/api/zones", tag = "zones",
    responses((status = 200, body = ArmedZones)))]
pub async fn read_armed(State(daemon): State<Arc<Daemon>>) -> Json<ArmedZones> {
    let (armed, zones) = daemon.device.armed_zones();
    Json(ArmedZones {
        zone_set: armed.as_ref().map(|armed| armed.zone_set.clone()),
        zone_set_version: armed.as_ref().map(|armed| armed.version),
        arm_id: armed.as_ref().map(|armed| armed.arm_id),
        label: armed.and_then(|armed| armed.label),
        zones,
    })
}

/// Resolve the patch, compile, upload, wait for `armed`.
#[utoipa::path(post, path = "/api/zones/arm", tag = "zones",
    request_body = ArmRequest,
    responses(
        (status = 200, body = ArmedZones),
        (status = 404, description = "no set by that name"),
        (status = 422, description = "the set did not compile"),
        (status = 503, description = "no board"),
    ))]
pub async fn arm(
    State(daemon): State<Arc<Daemon>>,
    ApiJson(request): ApiJson<ArmRequest>,
) -> ApiResult<Json<ArmedZones>> {
    if !daemon.device.connected() {
        return Err(ApiError::no_device());
    }
    let set = daemon.store.read(&request.zone_set)?;
    let calibration = daemon.calibration();
    let lines = daemon.config.lock().unwrap().lines.clone();
    let compiled = compile(
        &request.zone_set,
        &set,
        &request.patch,
        &calibration,
        &lines,
        &daemon.device.capacities(),
    )?;
    daemon.device.arm(compiled, request.origin, request.label);
    Ok(Json(read_armed(State(daemon)).await.0))
}

#[utoipa::path(post, path = "/api/zones/disarm", tag = "zones",
    responses((status = 200, body = ArmedZones)))]
pub async fn disarm(State(daemon): State<Arc<Daemon>>) -> Json<ArmedZones> {
    daemon.device.disarm();
    read_armed(State(daemon)).await
}

/// Write the armed set, and its armed state, to the board's flash.
#[utoipa::path(post, path = "/api/zones/save", tag = "zones",
    responses((status = 200, body = DeviceInfo), (status = 409, description = "nothing armed")))]
pub async fn save_to_flash(State(daemon): State<Arc<Daemon>>) -> ApiResult<Json<DeviceInfo>> {
    daemon
        .device
        .save_to_flash()
        .ok_or_else(|| ApiError::conflict("nothing_armed", "arm a set before writing it to flash"))?;
    let port = daemon.config.lock().unwrap().device.port.clone();
    Ok(Json(daemon.device.info(port)))
}
