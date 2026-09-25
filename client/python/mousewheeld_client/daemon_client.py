# SPDX-License-Identifier: AGPL-3.0-or-later
"""`MousewheeldClient` — one handle on one mousewheeld.

Methods are named for what they ask for, not for their rpcs, and they take and
return the types in `mousewheeld_client.api_types`. The five services are an arrangement
of the interface, not of a caller's day: somebody writing a session wants
`rig.arm(...)` and `rig.state()`, not to know that one lives on `Zones` and the
other on `StateService`.
"""

from __future__ import annotations

from collections.abc import Iterator
from types import TracebackType

import grpc

from mousewheeld_client._proto.mousewheeld.v1 import (
    calibration_pb2,
    calibration_pb2_grpc,
    config_pb2,
    config_pb2_grpc,
    device_pb2,
    device_pb2_grpc,
    state_pb2,
    state_pb2_grpc,
    zones_pb2,
    zones_pb2_grpc,
)

from . import _wire_conversions as convert
from ._grpc_transport import call, stream
from .daemon_refusals import DaemonRefusedTheRequest
from .api_types import (
    ArmedZones,
    ArmOrigin,
    AxisCalibrationPatch,
    BallCalibration,
    Calibration,
    DaemonConfig,
    DeviceInfo,
    FirmwareVersions,
    MeasurementApplied,
    MeasurementResult,
    MeasurementStarted,
    OutputLine,
    RigState,
    StreamFrame,
    ValidationReport,
    VersionReport,
    WireLine,
    ZoneSet,
    ZoneSetFile,
)

#: The port mousewheeld serves on. statemachined 8081, vstimd 8080, triald 8420.
DEFAULT_PORT = 8083


class MousewheeldClient:
    """One handle on the daemon that owns one wheel.

    Named for what it is — a client — rather than for the rig: a rig has four
    daemons on it, and a script that talks to two of them should be able to say
    which is which. Every client in this family is spelled the same way, after
    the daemon it speaks to.

    ::

        with MousewheeldClient("rig.local") as wheel:
            wheel.open_link()
            wheel.arm("goal", patch={"goal_cm": 180})
            for frame in wheel.watch_state(rate_hz=50):
                ...

    A bare host name gets the default port, because every rig in this family
    serves on the same one and writing it out is a chance to get it wrong.
    Plaintext: this is a rig network, and a daemon that required a certificate
    to answer "is the wheel turning" would be answered by nobody.
    """

    def __init__(self, target: str = "localhost", *, channel: grpc.Channel | None = None) -> None:
        self.target = target if ":" in target else f"{target}:{DEFAULT_PORT}"
        self._own_channel = channel is None
        self._channel = channel or grpc.insecure_channel(self.target)
        self._device = device_pb2_grpc.DeviceStub(self._channel)
        self._state = state_pb2_grpc.StateServiceStub(self._channel)
        self._calibration = calibration_pb2_grpc.CalibrationStub(self._channel)
        self._zones = zones_pb2_grpc.ZonesStub(self._channel)
        self._config = config_pb2_grpc.ConfigStub(self._channel)

    def __enter__(self) -> MousewheeldClient:
        return self

    def __exit__(
        self,
        kind: type[BaseException] | None,
        value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

    def close(self) -> None:
        """Close the channel, unless it was handed in — a channel this did not
        open is not this object's to close."""
        if self._own_channel:
            self._channel.close()

    def wait_until_ready(self, timeout_s: float = 5.0) -> None:
        """Block until the daemon answers, or raise.

        For a script that starts a daemon and then talks to it: a channel is
        lazy, so without this the first rpc is the thing that discovers nothing
        is listening.
        """
        try:
            grpc.channel_ready_future(self._channel).result(timeout=timeout_s)
        except grpc.FutureTimeoutError:
            raise TimeoutError(f"{self.target} did not answer within {timeout_s} s") from None

    # --------------------------------------------------------------- device ---

    def version(self) -> VersionReport:
        """Three versions that are not the same number, and the `vinput` layout.

        Answers without a board.
        """
        return convert.version_from_wire(
            call(lambda: self._device.ReadVersion(device_pb2.ReadVersionRequest()))
        )

    def device(self) -> DeviceInfo:
        return convert.device_info_from_wire(
            call(lambda: self._device.ReadDevice(device_pb2.ReadDeviceRequest()))
        )

    def open_link(self) -> DeviceInfo:
        """Open the serial port named in the rig config and greet the board.

        Idempotent: a connected daemon answers with what is already there.
        Raises `DaemonOrBoardIsUnavailable` when the port is not openable.
        """
        return convert.device_info_from_wire(
            call(lambda: self._device.OpenLink(device_pb2.OpenLinkRequest()))
        )

    def firmware(self) -> FirmwareVersions:
        return convert.firmware_from_wire(
            call(lambda: self._device.ReadFirmware(device_pb2.ReadFirmwareRequest()))
        )

    def wire_log(self) -> tuple[WireLine, ...]:
        """The wire's recent past: the whole conversation, and the last hundred
        samples. Samples are capped rather than the conversation, because at
        500 Hz they are the only thing that would come back."""
        answer = call(lambda: self._device.ReadWireLog(device_pb2.ReadWireLogRequest()))
        return tuple(convert.wire_line_from_wire(line) for line in answer.lines)

    def watch_wire(self) -> Iterator[WireLine]:
        """The wire as it happens, both directions, uninterpreted."""
        frames = stream(lambda: self._device.WatchWire(device_pb2.WatchWireRequest()))
        return (convert.wire_line_from_wire(line) for line in frames)

    # ---------------------------------------------------------------- state ---

    def state(self) -> RigState:
        return convert.rig_state_from_wire(
            call(lambda: self._state.ReadState(state_pb2.ReadStateRequest()))
        )

    def watch_state(self, rate_hz: int = 50) -> Iterator[StreamFrame]:
        """The decimated stream, as `Sample` and `ZoneHit` frames.

        `rate_hz` is what you ask for, not what the device samples at: the
        daemon decimates and says what it lost. Break a line on `lost_before`
        and on nothing else — `seq` skips here by design.

        The iterator is open until it is exhausted or garbage-collected; close
        it (or leave its `for` loop) to end the call.
        """
        frames = stream(
            lambda: self._state.WatchState(state_pb2.WatchStateRequest(rate_hz=rate_hz))
        )
        return (convert.stream_frame_from_wire(frame) for frame in frames)

    def zero_position(self, axes: tuple[str, ...] = ()) -> RigState:
        """Move the API origin — every axis, or the ones named.

        Never the accumulator vstimd differences every frame: a value that jumps
        backwards jumps the corridor backwards. Teleporting a corridor is a
        command to vstimd, from whoever runs the experiment.
        """
        return convert.rig_state_from_wire(
            call(lambda: self._state.ZeroPosition(state_pb2.ZeroRequest(axes=list(axes))))
        )

    def set_position(self, position_cm: tuple[float, ...], axes: tuple[str, ...] = ()) -> RigState:
        """Set the API origin to a specific position for vstimd sync.

        Unlike `zero_position` which sets the origin to the current position,
        `set_position` allows you to specify any position in centimetres.
        The origin is adjusted so that `position_cm` matches the requested value.

        This is for corridor synchronization: when vstimd resets the camera
        position, mousewheeld can sync the wheel position to match.
        """
        return convert.rig_state_from_wire(
            call(lambda: self._state.SetPosition(
                state_pb2.SetPositionRequest(
                    axes=list(axes),
                    position_cm=list(position_cm)
                )
            ))
        )

    # ---------------------------------------------------------- calibration ---

    def calibration(self) -> Calibration:
        return convert.calibration_from_wire(
            call(
                lambda: self._calibration.ReadCalibration(calibration_pb2.ReadCalibrationRequest())
            )
        )

    def set_calibration(self, *axes: AxisCalibrationPatch) -> Calibration:
        """Change what was named, on the axes named, and leave the rest alone."""
        patch = convert.calibration_patch_to_wire(axes)
        return convert.calibration_from_wire(
            call(lambda: self._calibration.ReplaceCalibration(patch))
        )

    def ball(self) -> BallCalibration:
        """The 2-D ball. `DaemonRefusedTheRequest` with `unimplemented` until the hardware
        exists — modelled so that adding it is filling in rather than
        redesigning."""
        return convert.ball_from_wire(
            call(lambda: self._calibration.ReadBall(calibration_pb2.ReadBallRequest()))
        )

    def set_ball(self, ball: BallCalibration) -> BallCalibration:
        return convert.ball_from_wire(
            call(lambda: self._calibration.ReplaceBall(convert.ball_to_wire(ball)))
        )

    def start_measuring(self, axis: str, known_distance_cm: float) -> MeasurementStarted:
        """Begin a measurement: name an axis and the distance about to be
        rolled, by hand, along the surface the animal runs on."""
        return convert.measurement_started_from_wire(
            call(
                lambda: self._calibration.StartMeasuring(
                    calibration_pb2.StartMeasurement(
                        axis=axis, known_distance_cm=known_distance_cm
                    )
                )
            )
        )

    def finish_measuring(self) -> MeasurementResult:
        """Measured against configured — and read `decoding_suspect` before
        applying it."""
        return convert.measurement_result_from_wire(
            call(
                lambda: self._calibration.FinishMeasuring(calibration_pb2.FinishMeasuringRequest())
            )
        )

    def apply_measurement(self) -> MeasurementApplied:
        """Make the measurement the rig's truth.

        A separate decision from taking it, and deliberately: applying
        invalidates every compiled zone set and disarms.
        """
        return convert.measurement_applied_from_wire(
            call(
                lambda: self._calibration.ApplyMeasurement(
                    calibration_pb2.ApplyMeasurementRequest()
                )
            )
        )

    # ---------------------------------------------------------------- zones ---

    def zone_sets(self) -> tuple[str, ...]:
        answer = call(lambda: self._zones.ListZoneSets(zones_pb2.ListZoneSetsRequest()))
        return tuple(answer.zone_sets)

    def zone_set(self, name: str) -> ZoneSet:
        """One stored set as types — references intact, centimetres intact."""
        return convert.zone_set_from_wire(
            call(lambda: self._zones.ReadZoneSet(zones_pb2.ZoneSetName(name=name)))
        )

    def set_zone_set(self, name: str, zone_set: ZoneSet) -> ZoneSet:
        return convert.zone_set_from_wire(
            call(
                lambda: self._zones.ReplaceZoneSet(
                    zones_pb2.WriteZoneSet(name=name, zone_set=convert.zone_set_to_wire(zone_set))
                )
            )
        )

    def zone_set_file(self, name: str) -> ZoneSetFile:
        """The same set as the **file** it is stored as.

        The other spelling, for the case where the text is the point: copying a
        set between rigs, diffing one, or writing one somebody hand-edited.
        """
        return convert.zone_set_file_from_wire(
            call(lambda: self._zones.ReadZoneSetFile(zones_pb2.ZoneSetName(name=name)))
        )

    def set_zone_set_file(self, name: str, text: str) -> ZoneSetFile:
        """Parse, compile and store a file, and answer with what is on disk."""
        return convert.zone_set_file_from_wire(
            call(
                lambda: self._zones.WriteZoneSetFile(zones_pb2.ZoneSetFile(name=name, text=text))
            )
        )

    def check_zone_set_file(self, text: str, name: str = "") -> ValidationReport:
        """Parse and compile a file that is not stored — for CI, or an editor.

        The daemon's own deserializer, so a file that does not parse comes back
        refused with a line and a column.
        """
        return convert.validation_report_from_wire(
            call(
                lambda: self._zones.ValidateZoneSetFile(
                    zones_pb2.ZoneSetFile(name=name, text=text)
                )
            )
        )

    def check_zone_set(self, name: str) -> ValidationReport:
        """Compile the set as it is **stored** — "is that one still good after
        the calibration changed"."""
        return convert.validation_report_from_wire(
            call(lambda: self._zones.ValidateZoneSet(zones_pb2.ZoneSetName(name=name)))
        )

    def check_draft(self, zone_set: ZoneSet) -> ValidationReport:
        """Compile a set that is not stored, saving nothing."""
        return convert.validation_report_from_wire(
            call(lambda: self._zones.ValidateDraft(convert.zone_set_to_wire(zone_set)))
        )

    def zone_set_file_schema(self) -> dict:
        """The JSON Schema of a zone-set **file**, as the running daemon
        believes it. Plain JSON — this one is a document, not a type."""
        from google.protobuf.json_format import MessageToDict

        answer = call(
            lambda: self._zones.ReadZoneSetSchema(zones_pb2.ReadZoneSetSchemaRequest())
        )
        return MessageToDict(answer)

    def armed(self) -> ArmedZones:
        """What is armed, what has fired, and each zone's bounds as armed."""
        return convert.armed_zones_from_wire(
            call(lambda: self._zones.ReadArmed(zones_pb2.ReadArmedRequest()))
        )

    def arm(
        self,
        zone_set: str,
        *,
        patch: dict[str, float] | None = None,
        origin: ArmOrigin = ArmOrigin.CURRENT,
        label: str | None = None,
    ) -> ArmedZones:
        """Resolve the patch, compile, upload, and wait for the board to say
        `armed`.

        `patch` fills in the `$name`s the set was authored with —
        `{"goal_cm": 180}`. A `$name` the patch does not carry is a refusal
        naming it, never a zero.

        Returning before the board confirmed would let a trial run believing a
        line is armed that is not, so a timeout here is a refusal.
        """
        return convert.armed_zones_from_wire(
            call(
                lambda: self._zones.Arm(
                    convert.arm_request_to_wire(zone_set, patch, origin, label)
                )
            )
        )

    def disarm(self) -> ArmedZones:
        return convert.armed_zones_from_wire(
            call(lambda: self._zones.Disarm(zones_pb2.DisarmRequest()))
        )

    def save_to_flash(self) -> DeviceInfo:
        """Write the armed set, and its armed state, to the board's flash, so a
        standalone rig comes up with its zones. The store on the host stays the
        source."""
        return convert.device_info_from_wire(
            call(lambda: self._zones.SaveToFlash(zones_pb2.SaveToFlashRequest()))
        )

    # --------------------------------------------------------------- config ---

    def config(self) -> DaemonConfig:
        return convert.config_from_wire(
            call(lambda: self._config.ReadConfig(config_pb2.ReadConfigRequest()))
        )

    def set_config(
        self,
        *,
        rate_hz: int | None = None,
        display_hz: int | None = None,
        ring_minutes: int | None = None,
    ) -> DaemonConfig:
        """Change what was named. `rate_hz` should be above `display_hz`, or
        some frames see no new sample and the next sees two — the daemon says so
        in `starves_the_display` rather than refusing."""
        patch = convert.config_patch_to_wire(rate_hz, display_hz, ring_minutes)
        return convert.config_from_wire(call(lambda: self._config.PatchConfig(patch)))

    def lines(self) -> tuple[OutputLine, ...]:
        """The output line map, from the rig config. Not writable over the API:
        wiring is changed where wiring is described."""
        return convert.lines_from_wire(
            call(lambda: self._config.ReadLines(config_pb2.ReadLinesRequest()))
        )


__all__ = ["DEFAULT_PORT", "DaemonRefusedTheRequest", "MousewheeldClient"]
