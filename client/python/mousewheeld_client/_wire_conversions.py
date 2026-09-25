# SPDX-License-Identifier: AGPL-3.0-or-later
"""The seam. Every protobuf message this client sees is made or read here.

The names are `x_from_wire` and `x_to_wire`, in that direction and nothing else,
so the direction of a call is readable rather than looked up — the same rule
`daemon/src/convert/` keeps on the other side, and `ipc/convert` in vstimd
before it.

**Enums are converted exhaustively, by table.** A wire value this build does not
know raises rather than becoming a default: a zone whose metric silently read as
`DISPLACEMENT` because a newer daemon sent `DISTANCE` is a trial measured
against the wrong thing. The tables are the only place the long wire spelling
appears in this package.
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar

from google.protobuf.message import Message

from mousewheeld_client._proto.mousewheeld.v1 import calibration_pb2, config_pb2, device_pb2, state_pb2, version_pb2, zones_pb2

from .api_types import (
    ArmedZones,
    ArmOrigin,
    AxisCalibration,
    AxisCalibrationPatch,
    AxisState,
    BallCalibration,
    ZoneBound,
    Calibration,
    BoardCapacities,
    DaemonConfig,
    DeviceInfo,
    DeviceProtocol,
    FireRule,
    FirmwareVersions,
    FlashedZoneSet,
    LinkHealth,
    LinkStatistics,
    MeasurementApplied,
    MeasurementResult,
    MeasurementStarted,
    OutputAction,
    OutputLine,
    ZoneBoundReference,
    RigState,
    Sample,
    SetPositionRequest,
    StreamFrame,
    ValidationReport,
    VersionReport,
    WireDirection,
    WireLevel,
    WireLine,
    Zone,
    ZoneHit,
    ZoneMetric,
    ZoneOutput,
    ZoneSet,
    ZoneSetFile,
    ZoneShape,
    ZoneStatus,
)

# ------------------------------------------------------------------- enums ---
#
# One table per enum, written out rather than derived from the name, because a
# value whose spelling does not follow the pattern should be a line in a table
# and not a surprise at three in the morning.

_METRICS = {
    zones_pb2.ZONE_METRIC_DISPLACEMENT: ZoneMetric.DISPLACEMENT,
    zones_pb2.ZONE_METRIC_DISTANCE: ZoneMetric.DISTANCE,
}
_SHAPES = {zones_pb2.ZONE_SHAPE_RECT: ZoneShape.RECT}
_FIRE_RULES = {
    zones_pb2.FIRE_RULE_ONCE: FireRule.ONCE,
    zones_pb2.FIRE_RULE_REARM: FireRule.REARM,
}
_ACTIONS = {
    zones_pb2.OUTPUT_ACTION_PULSE: OutputAction.PULSE,
    zones_pb2.OUTPUT_ACTION_LEVEL: OutputAction.LEVEL,
}
_ORIGINS = {
    zones_pb2.ARM_ORIGIN_CURRENT: ArmOrigin.CURRENT,
    zones_pb2.ARM_ORIGIN_ABSOLUTE: ArmOrigin.ABSOLUTE,
}
_DIRECTIONS = {
    device_pb2.WIRE_DIRECTION_OUT: WireDirection.OUT,
    device_pb2.WIRE_DIRECTION_IN: WireDirection.IN,
}
_LEVELS = {
    device_pb2.WIRE_LEVEL_INFO: WireLevel.INFO,
    device_pb2.WIRE_LEVEL_ERROR: WireLevel.ERROR,
}

_E = TypeVar("_E")


def _decode(table: Mapping[Any, _E], value: Any, what: str) -> _E:
    """A wire enum, or a refusal naming what could not be read.

    Zero is `*_UNSPECIFIED` in every one of these, which proto3 also gives a
    field nobody set — so it is refused like any other unknown, and a daemon
    that forgot to set one is a bug here rather than a default in a trial.
    """
    try:
        return table[value]
    except KeyError:
        raise ValueError(
            f"this client does not know {what} {value}; the daemon is newer than it is"
        ) from None


def _encode(table: Mapping[Any, _E], value: _E) -> Any:
    for wire, known in table.items():
        if known is value:
            return wire
    raise ValueError(f"no wire value for {value!r}")


def _maybe(message: Message, field: str):
    """An `optional` field, or `None`. `HasField` is the only honest test: a
    float at its default and an absent float are the same bytes otherwise."""
    return getattr(message, field) if message.HasField(field) else None


# ------------------------------------------------------------------ device ---


def version_from_wire(message: version_pb2.VersionReport) -> VersionReport:
    return VersionReport(
        daemon=message.daemon,
        api=message.api,
        device_protocol=DeviceProtocol(
            speaks=message.device_protocol.speaks,
            floor=message.device_protocol.floor,
            board=_maybe(message.device_protocol, "board"),
        ),
        vinput_layout=message.vinput_layout,
    )


def device_info_from_wire(message: device_pb2.DeviceInfo) -> DeviceInfo:
    flashed = message.flashed_zone_set if message.HasField("flashed_zone_set") else None
    return DeviceInfo(
        connected=message.connected,
        port=message.port,
        board=message.board,
        firmware_version=message.firmware_version,
        protocol_version=message.protocol_version,
        uptime_device_us=message.uptime_device_us,
        capacities=BoardCapacities(
            n_axes=message.capacities.n_axes,
            max_zones=message.capacities.max_zones,
            max_lines=message.capacities.max_lines,
            scan_hz=message.capacities.scan_hz,
        ),
        flashed_zone_set=(
            None if flashed is None else FlashedZoneSet(name=flashed.name, version=flashed.version)
        ),
        link=LinkStatistics(
            connection_count=message.link.connection_count,
            last_error=_maybe(message.link, "last_error"),
        ),
    )


def firmware_from_wire(message: device_pb2.FirmwareVersions) -> FirmwareVersions:
    return FirmwareVersions(running=message.running, available=tuple(message.available))


def wire_line_from_wire(message: device_pb2.WireLine) -> WireLine:
    return WireLine(
        host_monotonic_ns=message.host_monotonic_ns,
        direction=_decode(_DIRECTIONS, message.direction, "a wire direction"),
        text=message.text,
        level=_decode(_LEVELS, message.level, "a wire level"),
    )


# ------------------------------------------------------------------- state ---


def axis_state_from_wire(message: state_pb2.AxisState) -> AxisState:
    return AxisState(
        name=message.name,
        counts=message.counts,
        position_cm=message.position_cm,
        distance_cm=message.distance_cm,
        velocity_cm_s=message.velocity_cm_s,
        device_velocity_cm_s=message.device_velocity_cm_s,
    )


def rig_state_from_wire(message: state_pb2.RigState) -> RigState:
    return RigState(
        axes=tuple(axis_state_from_wire(axis) for axis in message.axes),
        health=LinkHealth(
            seq_gaps=message.health.seq_gaps,
            ring_drops=message.health.ring_drops,
            measured_rate_hz=message.health.measured_rate_hz,
            stale=message.health.stale,
        ),
        connected=message.link.connected,
    )


def set_position_request_to_wire(request: SetPositionRequest) -> state_pb2.SetPositionRequest:
    """Convert Python type to wire protobuf for SetPosition."""
    return state_pb2.SetPositionRequest(
        axes=list(request.axes),
        position_cm=list(request.position_cm)
    )


def stream_frame_from_wire(message: state_pb2.StreamFrame) -> StreamFrame:
    """One frame, as the arm it actually is.

    A frame from a newer daemon carrying an arm this build has never heard of
    raises rather than being half-read: `WhichOneof` returning a name nothing
    here handles is a frame nobody can act on.
    """
    arm = message.WhichOneof("frame")
    if arm == "sample":
        return Sample(
            seq=message.sample.seq,
            device_us=message.sample.device_us,
            host_monotonic_ns=message.sample.host_monotonic_ns,
            lost_before=message.sample.lost_before,
            axes=tuple(axis_state_from_wire(axis) for axis in message.sample.axes),
        )
    if arm == "zone_hit":
        return ZoneHit(
            zone=message.zone_hit.zone,
            arm_id=message.zone_hit.arm_id,
            seq=message.zone_hit.seq,
            host_monotonic_ns=message.zone_hit.host_monotonic_ns,
            position_cm=message.zone_hit.position_cm,
        )
    raise ValueError(f"this client does not know a stream frame of kind {arm!r}")


# ------------------------------------------------------------- calibration ---


def calibration_from_wire(message: calibration_pb2.CalibrationState) -> Calibration:
    return Calibration(
        axes=tuple(
            AxisCalibration(
                name=axis.name,
                counts_per_cm=axis.counts_per_cm,
                counts_per_rev=_maybe(axis, "counts_per_rev"),
                diameter_cm=_maybe(axis, "diameter_cm"),
                invert=axis.invert,
                measured_at=_maybe(axis, "measured_at"),
            )
            for axis in message.axes
        ),
        ball=ball_from_wire(message.ball) if message.HasField("ball") else None,
    )


def ball_from_wire(message: calibration_pb2.BallCalibration) -> BallCalibration:
    return BallCalibration(
        diameter_cm=message.diameter_cm,
        sensor_angles_deg=tuple(message.sensor_angles_deg),
    )


def ball_to_wire(ball: BallCalibration) -> calibration_pb2.BallCalibration:
    return calibration_pb2.BallCalibration(
        diameter_cm=ball.diameter_cm,
        sensor_angles_deg=list(ball.sensor_angles_deg),
    )


def calibration_patch_to_wire(
    patches: tuple[AxisCalibrationPatch, ...],
) -> calibration_pb2.CalibrationPatch:
    """Only what was named. An unset field on the wire is "leave it alone", and
    a patch that sent every field would overwrite a measurement with a
    default."""
    message = calibration_pb2.CalibrationPatch()
    for patch in patches:
        axis = message.axes.add()
        axis.name = patch.name
        for name in ("counts_per_cm", "counts_per_rev", "diameter_cm", "invert"):
            value = getattr(patch, name)
            if value is not None:
                setattr(axis, name, value)
    return message


def measurement_started_from_wire(
    message: calibration_pb2.MeasurementStarted,
) -> MeasurementStarted:
    return MeasurementStarted(
        axis=message.axis,
        known_distance_cm=message.known_distance_cm,
        counts_at_start=message.counts_at_start,
        counts=message.counts,
    )


def measurement_result_from_wire(message: calibration_pb2.MeasurementResult) -> MeasurementResult:
    return MeasurementResult(
        axis=message.axis,
        known_distance_cm=message.known_distance_cm,
        counts=message.counts,
        measured_counts_per_cm=message.measured_counts_per_cm,
        configured_counts_per_cm=message.configured_counts_per_cm,
        counts_per_rev=_maybe(message, "counts_per_rev"),
        nominal_counts_per_cm=_maybe(message, "nominal_counts_per_cm"),
        implied_circumference_cm=_maybe(message, "implied_circumference_cm"),
        decoding_suspect=_maybe(message, "decoding_suspect"),
    )


def measurement_applied_from_wire(
    message: calibration_pb2.MeasurementApplied,
) -> MeasurementApplied:
    return MeasurementApplied(
        axis=message.axis,
        counts_per_cm=message.counts_per_cm,
        zone_sets_invalidated=message.zone_sets_invalidated,
    )


# ------------------------------------------------------------------- zones ---


def bound_from_wire(message: zones_pb2.ZoneBound) -> ZoneBound:
    arm = message.WhichOneof("bound")
    if arm == "value":
        return message.value
    if arm == "reference":
        return ZoneBoundReference(message.reference)
    return None  # an open end: protobuf has no nullable double in a repeated field


def bound_to_wire(bound: ZoneBound) -> zones_pb2.ZoneBound:
    if bound is None:
        return zones_pb2.ZoneBound()
    if isinstance(bound, ZoneBoundReference):
        return zones_pb2.ZoneBound(reference=bound.name)
    return zones_pb2.ZoneBound(value=float(bound))


def zone_set_from_wire(message: zones_pb2.ZoneSet) -> ZoneSet:
    return ZoneSet(
        schema_url=_maybe(message, "schema_url"),
        zone_set_version=message.zone_set_version,
        zones=tuple(
            Zone(
                name=zone.name,
                shape=_decode(_SHAPES, zone.shape, "a zone shape"),
                axes=tuple(zone.axes),
                metric=_decode(_METRICS, zone.metric, "a zone metric"),
                min_cm=tuple(bound_from_wire(bound) for bound in zone.min_cm),
                max_cm=tuple(bound_from_wire(bound) for bound in zone.max_cm),
                wrap_cm=_maybe(zone, "wrap_cm"),
                fire=_decode(_FIRE_RULES, zone.fire, "a fire rule"),
                hysteresis_cm=_maybe(zone, "hysteresis_cm"),
                level=zone.level,
                output=ZoneOutput(
                    line=zone.output.line,
                    action=_decode(_ACTIONS, zone.output.action, "an output action"),
                    ms=_maybe(zone.output, "ms"),
                ),
            )
            for zone in message.zones
        ),
    )


def zone_set_to_wire(zone_set: ZoneSet) -> zones_pb2.ZoneSet:
    message = zones_pb2.ZoneSet(zone_set_version=zone_set.zone_set_version)
    if zone_set.schema_url is not None:
        message.schema_url = zone_set.schema_url
    for zone in zone_set.zones:
        wire = message.zones.add()
        wire.name = zone.name
        wire.shape = _encode(_SHAPES, zone.shape)
        wire.axes.extend(zone.axes)
        wire.metric = _encode(_METRICS, zone.metric)
        wire.min_cm.extend(bound_to_wire(bound) for bound in zone.min_cm)
        wire.max_cm.extend(bound_to_wire(bound) for bound in zone.max_cm)
        if zone.wrap_cm is not None:
            wire.wrap_cm = zone.wrap_cm
        wire.fire = _encode(_FIRE_RULES, zone.fire)
        if zone.hysteresis_cm is not None:
            wire.hysteresis_cm = zone.hysteresis_cm
        wire.level = zone.level
        wire.output.line = zone.output.line
        wire.output.action = _encode(_ACTIONS, zone.output.action)
        if zone.output.ms is not None:
            wire.output.ms = zone.output.ms
    return message


def zone_set_file_from_wire(message: zones_pb2.ZoneSetFile) -> ZoneSetFile:
    return ZoneSetFile(name=message.name, text=message.text)


def armed_zones_from_wire(message: zones_pb2.ArmedZones) -> ArmedZones:
    return ArmedZones(
        zone_set=_maybe(message, "zone_set"),
        zone_set_version=_maybe(message, "zone_set_version"),
        arm_id=_maybe(message, "arm_id"),
        label=_maybe(message, "label"),
        zones=tuple(
            ZoneStatus(
                name=zone.name,
                armed=zone.armed,
                fired=zone.fired,
                inside=zone.inside,
                fired_at_cm=_maybe(zone, "fired_at_cm"),
                min_cm=tuple(_maybe(bound, "value") for bound in zone.min_cm),
                max_cm=tuple(_maybe(bound, "value") for bound in zone.max_cm),
                wrap_cm=_maybe(zone, "wrap_cm"),
                metric=_decode(_METRICS, zone.metric, "a zone metric"),
            )
            for zone in message.zones
        ),
    )


def arm_request_to_wire(
    zone_set: str,
    patch: dict[str, float] | None,
    origin: ArmOrigin,
    label: str | None,
) -> zones_pb2.ArmRequest:
    message = zones_pb2.ArmRequest(
        zone_set=zone_set,
        origin=_encode(_ORIGINS, origin),
    )
    if patch:
        message.patch.update({name: float(value) for name, value in patch.items()})
    if label is not None:
        message.label = label
    return message


def validation_report_from_wire(message: zones_pb2.ValidationReport) -> ValidationReport:
    return ValidationReport(
        ok=message.ok,
        problem=_maybe(message, "problem"),
        zone_count=message.zone_count,
        counts_per_cm=message.counts_per_cm,
        references=tuple(message.references),
    )


# ------------------------------------------------------------------ config ---


def config_from_wire(message: config_pb2.ConfigView) -> DaemonConfig:
    return DaemonConfig(
        rate_hz=message.rate_hz,
        display_hz=message.display_hz,
        ring_minutes=message.ring_minutes,
        shm_name=message.shm_name,
        shm_open=message.shm_open,
        shm_writes=message.shm_writes,
        event_port=message.event_port,
        port=message.port,
        starves_the_display=message.starves_the_display,
    )


def config_patch_to_wire(
    rate_hz: int | None, display_hz: int | None, ring_minutes: int | None
) -> config_pb2.ConfigPatch:
    message = config_pb2.ConfigPatch()
    if rate_hz is not None:
        message.rate_hz = rate_hz
    if display_hz is not None:
        message.display_hz = display_hz
    if ring_minutes is not None:
        message.ring_minutes = ring_minutes
    return message


def lines_from_wire(message: config_pb2.LineMap) -> tuple[OutputLine, ...]:
    return tuple(
        OutputLine(name=line.name, index=line.index, pin=line.pin, safe_high=line.safe_high)
        for line in message.lines
    )
