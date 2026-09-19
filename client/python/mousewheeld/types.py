# SPDX-License-Identifier: AGPL-3.0-or-later
"""What this client hands back, and what it takes.

**No protobuf message is ever returned from this package, and none is
accepted.** The generated types live in `mousewheeld._proto`, `_convert.py` is
the seam, and everything here is a frozen dataclass or an enum. That is the
family's rule (`contracts/DAEMON_LAYOUT.md`): the interface is authored in
`proto/mousewheeld/v1/`, generated code implements it, and a caller writing an
experiment should never have to learn a generated API to read a number.

It buys three things that matter in a rig script:

* **`None` means absent.** protobuf spells an absent `float` as a message with
  no field set, and an absent enum as zero; both read as a value. Here a
  calibration that has never been measured has `measured_at is None`.
* **Enums are the short names.** `ZoneMetric.DISPLACEMENT`, not
  `ZONE_METRIC_DISPLACEMENT` — the wire spelling exists so that clients in
  different languages agree about the same byte, and a person typing a script
  is not a wire.
* **A quantity spells its unit**, as everywhere else in this family:
  `position_cm`, `velocity_cm_s`, `rate_hz`.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

# --------------------------------------------------------------- vocabulary ---


class ZoneShape(Enum):
    RECT = "rect"


class ZoneMetric(Enum):
    """What a zone is measured against.

    `DISPLACEMENT` is signed position from the origin — where the animal *is*.
    `DISTANCE` is path length, which only grows: how far it has run.
    """

    DISPLACEMENT = "displacement"
    DISTANCE = "distance"


class FireRule(Enum):
    ONCE = "once"
    REARM = "rearm"


class OutputAction(Enum):
    PULSE = "pulse"
    LEVEL = "level"


class ArmOrigin(Enum):
    """Where a zone's centimetres are measured from.

    `CURRENT` zeroes at the moment of arming, which is what a trial wants: the
    goal is 180 cm from *here*. `ABSOLUTE` keeps the device's own origin.
    """

    CURRENT = "current"
    ABSOLUTE = "absolute"


class WireDirection(Enum):
    OUT = "out"
    IN = "in"


class WireLevel(Enum):
    INFO = "info"
    ERROR = "error"


# ------------------------------------------------------------------- device ---


@dataclass(frozen=True)
class DeviceProtocol:
    """What this daemon speaks, the oldest it will speak, and what answered.

    `board` is `None` until a board has greeted. Advertised and never gated on
    (`contracts/INTERACTIONS.md` §11).
    """

    speaks: int
    floor: int
    board: int | None


@dataclass(frozen=True)
class VersionReport:
    daemon: str
    api: int
    device_protocol: DeviceProtocol
    vinput_layout: int


@dataclass(frozen=True)
class Capacities:
    """What the board can hold. A zone set that does not fit is refused here
    rather than half-uploaded."""

    n_axes: int
    max_zones: int
    max_lines: int
    scan_hz: int


@dataclass(frozen=True)
class FlashedZoneSet:
    name: str
    version: int


@dataclass(frozen=True)
class LinkStats:
    connection_count: int
    last_error: str | None


@dataclass(frozen=True)
class DeviceInfo:
    connected: bool
    port: str
    board: str
    firmware_version: str
    protocol_version: int
    uptime_device_us: int
    capacities: Capacities
    flashed_zone_set: FlashedZoneSet | None
    link: LinkStats


@dataclass(frozen=True)
class FirmwareVersions:
    running: str
    available: tuple[str, ...]


@dataclass(frozen=True)
class WireLine:
    """One line of the serial conversation, uninterpreted."""

    host_monotonic_ns: int
    direction: WireDirection
    text: str
    level: WireLevel


# -------------------------------------------------------------------- state ---


@dataclass(frozen=True)
class AxisState:
    """One axis, in counts and in centimetres.

    Both velocities are here because they answer different questions and are
    never averaged together: `velocity_cm_s` is the host's, differenced from the
    sample path; `device_velocity_cm_s` is the board's own, over a window in its
    scan. A disagreement between them is worth seeing.
    """

    name: str
    counts: int
    position_cm: float
    distance_cm: float
    velocity_cm_s: float
    device_velocity_cm_s: float


@dataclass(frozen=True)
class LinkHealth:
    """Whether to believe the numbers beside it.

    `stale` is the one to check in a trial loop: a board that stopped sending
    looks exactly like a board sending zeroes.
    """

    seq_gaps: int
    ring_drops: int
    measured_rate_hz: float
    stale: bool


@dataclass(frozen=True)
class RigState:
    axes: tuple[AxisState, ...]
    health: LinkHealth
    connected: bool

    def axis(self, name: str) -> AxisState:
        """One axis by name, or `KeyError` — never a silent default."""
        for axis in self.axes:
            if axis.name == name:
                return axis
        raise KeyError(f"no axis named {name}; this rig has {[a.name for a in self.axes]}")


@dataclass(frozen=True)
class Sample:
    """One sample of the stream.

    **`lost_before` is the field to break a line on**, and the only one:
    samples the daemon never received since the frame before, accumulated across
    the ones decimation dropped. `seq` skips here by design — the stream is
    decimated — so a consumer that watched `seq` would see a gap in every frame.
    """

    seq: int
    device_us: int
    host_monotonic_ns: int
    lost_before: int
    axes: tuple[AxisState, ...]


@dataclass(frozen=True)
class ZoneHit:
    """A zone fired. Decided on the device, in the scan that saw the count;
    this is the report of it, not the path it took."""

    zone: str
    arm_id: int
    seq: int
    host_monotonic_ns: int
    position_cm: float


#: One frame of `watch_state`. Match on it rather than testing attributes.
StreamFrame = Sample | ZoneHit


# -------------------------------------------------------------- calibration ---


@dataclass(frozen=True)
class AxisCalibration:
    """Counts per centimetre, and the dimensions the arithmetic is checked
    against.

    `measured_at` is `None` when the number came from the diameter rather than
    from rolling the wheel — which is arithmetic, not a measurement.
    """

    name: str
    counts_per_cm: float
    counts_per_rev: int | None = None
    diameter_cm: float | None = None
    invert: bool = False
    measured_at: str | None = None


@dataclass(frozen=True)
class BallCalibration:
    diameter_cm: float
    sensor_angles_deg: tuple[float, ...]


@dataclass(frozen=True)
class Calibration:
    axes: tuple[AxisCalibration, ...]
    ball: BallCalibration | None = None


@dataclass(frozen=True)
class AxisCalibrationPatch:
    """One axis's calibration, with everything unnamed left alone."""

    name: str
    counts_per_cm: float | None = None
    counts_per_rev: int | None = None
    diameter_cm: float | None = None
    invert: bool | None = None


@dataclass(frozen=True)
class MeasurementStarted:
    axis: str
    known_distance_cm: float
    counts_at_start: int
    counts: int


@dataclass(frozen=True)
class MeasurementResult:
    """What rolling the wheel said, against what the rig believes.

    `decoding_suspect` is the one to read: a measurement that comes out a whole
    -number multiple of the configured value is a decoder counting ×1 or ×2
    where the counts-per-revolution assumed ×4. It looks exactly like a wheel of
    the wrong size, and this names it before anybody applies it.
    """

    axis: str
    known_distance_cm: float
    counts: int
    measured_counts_per_cm: float
    configured_counts_per_cm: float
    counts_per_rev: int | None = None
    nominal_counts_per_cm: float | None = None
    implied_circumference_cm: float | None = None
    decoding_suspect: str | None = None


@dataclass(frozen=True)
class MeasurementApplied:
    axis: str
    counts_per_cm: float
    zone_sets_invalidated: bool


# -------------------------------------------------------------------- zones ---


@dataclass(frozen=True)
class Reference:
    """A bound that is filled in at arm time — `$goal_cm` in a zone-set file.

    A separate type rather than a bare string so that a bound is never a number
    that happens to be text: the compiler refuses a reference the arm patch does
    not carry, by name, rather than treating it as zero. A goal distance that
    silently became 0 cm is a trial that looks like it ran.
    """

    name: str


#: An authored bound: a distance, a value supplied at arm, or an open end.
Bound = float | Reference | None


@dataclass(frozen=True)
class ZoneOutput:
    line: str
    action: OutputAction
    ms: int | None = None


@dataclass(frozen=True)
class Zone:
    name: str
    axes: tuple[str, ...]
    metric: ZoneMetric
    output: ZoneOutput
    shape: ZoneShape = ZoneShape.RECT
    min_cm: tuple[Bound, ...] = ()
    max_cm: tuple[Bound, ...] = ()
    wrap_cm: float | None = None
    fire: FireRule = FireRule.ONCE
    hysteresis_cm: float | None = None
    level: bool = False


@dataclass(frozen=True)
class ZoneSet:
    """A set as authored: references intact, centimetres intact.

    The store holds what a person wrote; compilation into the board's integer
    counts happens at arm, against the calibration in force then.
    """

    zones: tuple[Zone, ...]
    zone_set_version: int = 1
    schema_url: str | None = None


@dataclass(frozen=True)
class ZoneStatus:
    """One zone as the board holds it right now.

    The bounds are the **armed** ones: every `$name` resolved, converted back
    from the counts the board is actually comparing against. `None` is an open
    end.
    """

    name: str
    armed: bool
    fired: bool
    inside: bool
    metric: ZoneMetric
    min_cm: tuple[float | None, ...] = ()
    max_cm: tuple[float | None, ...] = ()
    fired_at_cm: float | None = None
    wrap_cm: float | None = None


@dataclass(frozen=True)
class ArmedZones:
    """What is armed. Every field is `None` when nothing is."""

    zones: tuple[ZoneStatus, ...] = ()
    zone_set: str | None = None
    zone_set_version: int | None = None
    arm_id: int | None = None
    label: str | None = None


@dataclass(frozen=True)
class ValidationReport:
    """What the compiler makes of a set, against what is in force right now.

    A set with `references` is not broken, it is parameterised: those are the
    `$name`s an arm patch has to carry.
    """

    ok: bool
    zone_count: int
    counts_per_cm: float
    references: tuple[str, ...] = ()
    problem: str | None = None


# ------------------------------------------------------------------- config ---


@dataclass(frozen=True)
class ConfigView:
    """The settings a session may change, and the facts it may not.

    `shm_open` and `shm_writes` are the pair worth reading: a name in a config
    file and a mapped segment are different things, and the difference is the
    usual reason a corridor does not move.
    """

    rate_hz: int
    display_hz: int
    ring_minutes: int
    shm_name: str
    shm_open: bool
    shm_writes: int
    event_port: int
    port: str
    starves_the_display: bool


@dataclass(frozen=True)
class OutputLine:
    """One trigger line, as the rig config describes it. Read-only over the
    API: wiring is changed where wiring is described."""

    name: str
    index: int
    pin: int
    safe_high: bool


@dataclass(frozen=True)
class ZoneSetFile:
    """A stored set as the text it is on disk.

    The other spelling of a zone set, and both are real: `ZoneSet` above is what
    a script builds, this is what a person edits and copies between rigs. The
    daemon converts between them, once, so that no client has to.
    """

    name: str
    text: str
