# SPDX-License-Identifier: AGPL-3.0-or-later
"""The seam, held to the interface — with no daemon and no network.

These are the tests the guiding principle asks for: the API is defined in one
place and strongly typed, so what this client makes of it can be checked
without a rig. Everything here is `_wire_conversions` against the generated types.
"""

from __future__ import annotations

import pytest

from mousewheeld_client import _wire_conversions as convert
from mousewheeld_client.api_types import (
    ArmOrigin,
    AxisCalibrationPatch,
    FireRule,
    OutputAction,
    ZoneBoundReference,
    Sample,
    Zone,
    ZoneHit,
    ZoneMetric,
    ZoneOutput,
    ZoneSet,
    ZoneShape,
)
from mousewheeld_client._proto.mousewheeld.v1 import calibration_pb2, state_pb2, zones_pb2


def a_zone_set() -> ZoneSet:
    return ZoneSet(
        zone_set_version=1,
        zones=(
            Zone(
                name="goal",
                axes=("wheel",),
                metric=ZoneMetric.DISPLACEMENT,
                shape=ZoneShape.RECT,
                min_cm=(ZoneBoundReference("goal_cm"),),
                max_cm=(None,),
                fire=FireRule.ONCE,
                output=ZoneOutput(line="zone_goal", action=OutputAction.PULSE, ms=10),
            ),
            Zone(
                name="region",
                axes=("wheel",),
                metric=ZoneMetric.DISTANCE,
                min_cm=(10.0,),
                max_cm=(30.0,),
                wrap_cm=360.0,
                fire=FireRule.REARM,
                hysteresis_cm=0.5,
                level=True,
                output=ZoneOutput(line="zone_region", action=OutputAction.LEVEL),
            ),
        ),
    )


def test_a_zone_set_survives_the_round_trip_unchanged():
    """Every field, both directions. A converter that drops one is a zone that
    quietly fires somewhere else."""
    original = a_zone_set()
    assert convert.zone_set_from_wire(convert.zone_set_to_wire(original)) == original


def test_the_three_kinds_of_bound_are_three_different_things():
    """A distance, a value supplied at arm, and an open end. protobuf has no
    nullable double inside a repeated field, so all three are the same shape on
    the wire and only the `oneof` arm tells them apart."""
    for bound in (30.0, ZoneBoundReference("goal_cm"), None):
        assert convert.bound_from_wire(convert.bound_to_wire(bound)) == bound

    assert convert.bound_from_wire(zones_pb2.ZoneBound()) is None
    assert convert.bound_from_wire(zones_pb2.ZoneBound(value=0.0)) == 0.0


def test_a_bound_of_zero_is_not_an_open_end():
    """The case the `oneof` exists for: a zone that starts at the origin.
    Testing the value would make `0 cm` and `no bound` the same zone."""
    wire = convert.bound_to_wire(0.0)
    assert wire.WhichOneof("bound") == "value"
    assert convert.bound_from_wire(wire) == 0.0


def test_an_enum_this_build_does_not_know_is_refused_rather_than_defaulted():
    """A newer daemon's value must not read as the first one in the table.

    A zone whose metric silently became `DISPLACEMENT` because this client had
    never heard of `DISTANCE` is a trial measured against the wrong thing, and
    nothing about it looks wrong.
    """
    wire = convert.zone_set_to_wire(a_zone_set())
    wire.zones[0].metric = 99  # ty: ignore[invalid-assignment]  (the point: a value no enum has)
    with pytest.raises(ValueError, match="does not know a zone metric 99"):
        convert.zone_set_from_wire(wire)


def test_an_unset_enum_is_refused_too():
    """Zero is `*_UNSPECIFIED`, which is also what proto3 gives a field nobody
    set. A daemon that forgot to set one is a bug here, not a default."""
    wire = convert.zone_set_to_wire(a_zone_set())
    wire.zones[0].fire = 0  # ty: ignore[invalid-assignment]  (the point: unspecified)
    with pytest.raises(ValueError, match="a fire rule 0"):
        convert.zone_set_from_wire(wire)


def test_absent_is_none_and_not_zero():
    """`counts_per_rev` unset and `counts_per_rev = 0` are the same bytes for a
    proto3 scalar; they are only distinguishable because the field is
    `optional`, and this is the test that says so."""
    state = calibration_pb2.CalibrationState()
    axis = state.axes.add()
    axis.name = "wheel"
    axis.counts_per_cm = 86.92

    read = convert.calibration_from_wire(state)
    assert read.axes[0].counts_per_rev is None
    assert read.axes[0].measured_at is None
    assert read.ball is None

    axis.counts_per_rev = 0
    assert convert.calibration_from_wire(state).axes[0].counts_per_rev == 0


def test_a_calibration_patch_carries_only_what_was_named():
    """An unset field means "leave it alone". A patch that sent every field
    would overwrite a measurement with a default the caller never typed."""
    patch = convert.calibration_patch_to_wire((AxisCalibrationPatch(name="wheel", invert=True),))
    axis = patch.axes[0]
    assert axis.name == "wheel"
    assert axis.invert is True
    assert not axis.HasField("counts_per_cm")
    assert not axis.HasField("diameter_cm")


def test_a_stream_frame_becomes_the_arm_it_is():
    sample = state_pb2.StreamFrame()
    sample.sample.seq = 7
    sample.sample.lost_before = 3
    assert convert.stream_frame_from_wire(sample) == Sample(
        seq=7, device_us=0, host_monotonic_ns=0, lost_before=3, axes=()
    )

    hit = state_pb2.StreamFrame()
    hit.zone_hit.zone = "goal"
    hit.zone_hit.position_cm = 180.0
    assert convert.stream_frame_from_wire(hit) == ZoneHit(
        zone="goal", arm_id=0, seq=0, host_monotonic_ns=0, position_cm=180.0
    )


def test_a_frame_with_no_arm_is_refused():
    """A frame from a daemon newer than this client. Skipping it silently would
    hand a trial a shorter path than the one the animal ran."""
    with pytest.raises(ValueError, match="stream frame of kind None"):
        convert.stream_frame_from_wire(state_pb2.StreamFrame())


def test_an_arm_request_says_what_it_was_asked():
    request = convert.arm_request_to_wire(
        "goal", {"goal_cm": 180}, ArmOrigin.ABSOLUTE, "trial 42"
    )
    assert request.zone_set == "goal"
    assert request.patch["goal_cm"] == 180.0
    assert request.origin == zones_pb2.ARM_ORIGIN_ABSOLUTE
    assert request.label == "trial 42"


def test_an_arm_request_with_no_label_sends_no_label():
    """`label` is `optional`: absent and empty are different, and a trial with
    no label should not be stored as one labelled ""."""
    request = convert.arm_request_to_wire("goal", None, ArmOrigin.CURRENT, None)
    assert not request.HasField("label")
    assert len(request.patch) == 0
