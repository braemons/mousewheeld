//! The OpenAPI document, derived from the same types the routes use.
//!
//! **This is why the clients are checked against the daemon rather than against
//! a description of it.** The document is generated from the handlers'
//! annotations and the model's `ToSchema` derives — the same structs that are
//! the HTTP bodies and the files on disk — so a field renamed in one place
//! cannot stay right in another. A hand-written specification is a second copy,
//! and a second copy of a schema is a second place for it to be wrong.
//!
//! Served at `/api/openapi.json`, which is where both clients look.

use axum::Json;
use utoipa::OpenApi;

use crate::model::calibration::{
    AxisCalibration, AxisCalibrationPatch, BallCalibration, Calibration, CalibrationPatch,
    MeasurementApplied, MeasurementResult, MeasurementStarted, StartMeasurement,
};
use crate::model::config::{ConfigPatch, ConfigView};
use crate::model::device::{
    Capacities, DeviceInfo, FirmwareVersions, FlashedZoneSet, LinkStats, WireDirection, WireLevel,
    WireLine, WireLog,
};
use crate::model::error::ApiErrorBody;
use crate::model::line_map::{LineMap, OutputLine};
use crate::model::version::{DeviceProtocol, VersionReport};
use crate::model::state::{
    AxisState, LinkHealth, LinkState, RigState, Sample, StreamFrame, ZeroRequest, ZoneHitEvent,
};
use crate::model::zone_set::{
    ArmOrigin, ArmRequest, ArmedZones, FireRule, OutputAction, ValidationReport, Zone, ZoneBound,
    ZoneMetric, ZoneOutput, ZoneSet, ZoneSetNames, ZoneShape, ZoneStatus,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "mousewheeld",
        description = "The locomotion input of a braemons rig: a running wheel, read and published.\n\n\
                       Units are in the names — `position_cm`, `counts_per_cm`, `velocity_cm_s`. \
                       Counts are the device's only unit; everything that leaves here is centimetres. \
                       Unknown fields are refused rather than defaulted.",
        version = env!("CARGO_PKG_VERSION"),
        license(name = "AGPL-3.0-or-later"),
    ),
    paths(
        super::device_routes::read_version,
        super::device_routes::read_device,
        super::device_routes::connect,
        super::device_routes::read_firmware,
        super::device_routes::read_wire_log,
        super::state_routes::read_state,
        super::state_routes::zero_position,
        super::calibration_routes::read_calibration,
        super::calibration_routes::replace_calibration,
        super::calibration_routes::read_ball,
        super::calibration_routes::replace_ball,
        super::calibration_routes::start_measurement,
        super::calibration_routes::finish_measurement,
        super::calibration_routes::apply_measurement,
        super::zone_routes::list_zone_sets,
        super::zone_routes::read_zone_set,
        super::zone_routes::replace_zone_set,
        super::zone_routes::validate_zone_set,
        super::zone_routes::validate_body,
        super::schema::zone_set_schema,
        super::zone_routes::read_armed,
        super::zone_routes::arm,
        super::zone_routes::disarm,
        super::zone_routes::save_to_flash,
        super::config_routes::read_config,
        super::config_routes::patch_config,
        super::config_routes::read_lines,
    ),
    components(schemas(
        ApiErrorBody,
        AxisCalibration, AxisCalibrationPatch, BallCalibration, Calibration, CalibrationPatch,
        MeasurementApplied, MeasurementResult, MeasurementStarted, StartMeasurement,
        ConfigPatch, ConfigView,
        Capacities, DeviceInfo, FirmwareVersions, FlashedZoneSet, LinkStats,
        WireDirection, WireLevel, WireLine, WireLog,
        LineMap, OutputLine,
        DeviceProtocol, VersionReport,
        AxisState, LinkHealth, LinkState, RigState, Sample, StreamFrame, ZeroRequest, ZoneHitEvent,
        ArmOrigin, ArmRequest, ArmedZones, FireRule, OutputAction, ValidationReport, Zone,
        ZoneBound, ZoneMetric, ZoneOutput, ZoneSet, ZoneSetNames, ZoneShape, ZoneStatus,
    )),
    tags(
        (name = "device", description = "The board, and the wire to it"),
        (name = "state", description = "Where the wheel is"),
        (name = "calibration", description = "Counts per centimetre, and how it was measured"),
        (name = "zones", description = "Distances at which a line fires"),
        (name = "config", description = "Rates, and the line map"),
    ),
)]
pub struct ApiDoc;

/// The two WebSockets are not in the document.
///
/// OpenAPI describes request/response over HTTP and has no vocabulary for a
/// socket that pushes frames; a fabricated `GET` returning a `Sample` would
/// describe something no client can call. They are specified where they are
/// implemented, and `StreamFrame` — what they carry — is in the components
/// above, so a client can still generate the type it decodes.
pub async fn openapi_document() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
