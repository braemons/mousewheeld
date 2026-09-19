# SPDX-License-Identifier: AGPL-3.0-or-later
"""mousewheeld — Python client for the locomotion daemon.

Speaks gRPC to a daemon that owns a wheel, and hands back types::

    from mousewheeld import MousewheeldClient

    with MousewheeldClient("rig.local") as rig:
        rig.open_link()
        rig.arm("goal", patch={"goal_cm": 180})
        for frame in rig.watch_state(rate_hz=50):
            print(frame.axes[0].position_cm)

**No protobuf type is exported from this package, and none is accepted.** The
generated code is private, in `mousewheeld._proto`; `mousewheeld.api_types` is
the public vocabulary and `mousewheeld._wire_conversions` is the seam between
them. A caller
writing an experiment should never have to learn a generated API to read a
number — and this package can keep a name the day the interface adds a field.

The interface those types come from is `proto/mousewheeld/v1/` in the daemon's
repository, authored by hand — types *and* rpcs. `docs/reference/api.md` says
what each rpc is for.
"""

# Extend the package search path so that `from mousewheeld.v1 import ...` in the
# generated stubs resolves to `_proto/mousewheeld/v1/` without shadowing this
# package's own namespace. The same trick vstimd's client uses, for the same
# reason: protoc writes imports rooted at the proto path, and the proto path
# starts with this package's own name.
import os as _os

__path__ = list(__path__) + [_os.path.join(_os.path.dirname(__file__), "_proto", "mousewheeld")]

from .daemon_refusals import DaemonOrBoardIsUnavailable, DaemonRefusedTheRequest  # noqa: E402
from .daemon_client import DEFAULT_PORT, MousewheeldClient  # noqa: E402
from .api_types import (  # noqa: E402
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

__all__ = [
    "DEFAULT_PORT",
    "ArmOrigin",
    "ArmedZones",
    "AxisCalibration",
    "AxisCalibrationPatch",
    "AxisState",
    "BallCalibration",
    "ZoneBound",
    "Calibration",
    "BoardCapacities",
    "DaemonConfig",
    "DeviceInfo",
    "DeviceProtocol",
    "FireRule",
    "FirmwareVersions",
    "FlashedZoneSet",
    "LinkHealth",
    "LinkStatistics",
    "MeasurementApplied",
    "MeasurementResult",
    "MeasurementStarted",
    "DaemonOrBoardIsUnavailable",
    "OutputAction",
    "OutputLine",
    "ZoneBoundReference",
    "DaemonRefusedTheRequest",
    "MousewheeldClient",
    "RigState",
    "Sample",
    "StreamFrame",
    "ValidationReport",
    "VersionReport",
    "WireDirection",
    "WireLevel",
    "WireLine",
    "Zone",
    "ZoneHit",
    "ZoneMetric",
    "ZoneOutput",
    "ZoneSet",
    "ZoneSetFile",
    "ZoneShape",
    "ZoneStatus",
]
