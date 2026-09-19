# SPDX-License-Identifier: AGPL-3.0-or-later
"""The client against a daemon, over real gRPC.

What `test_convert.py` cannot check: that the rpcs are the ones the daemon
serves, that a refusal arrives with its code and its context, and that a stream
decimates.
"""

from __future__ import annotations

import time

import pytest

from mousewheeld import ArmOrigin, NotConnected, Reference, Refused, Sample, ZoneHit


def test_it_answers_who_it_is_without_a_board(rig):
    version = rig.version()
    assert version.daemon
    assert version.api >= 1
    # Advertised, never gated on: the floor is the oldest this daemon will
    # speak to, and the board says what it speaks.
    assert version.device_protocol.floor <= version.device_protocol.speaks


def test_the_simulated_board_is_a_board(rig):
    device = rig.device()
    assert device.connected
    assert device.board == "simulated"
    assert device.capacities.max_zones > 0


def test_the_wheel_turns(rig):
    first = rig.state().axis("wheel")
    assert not rig.state().health.stale
    # The simulator runs; counts are cumulative, so this only ever grows.
    assert rig.state().axis("wheel").counts >= first.counts


def test_an_unknown_axis_is_a_key_error_rather_than_a_default(rig):
    with pytest.raises(KeyError, match="no axis named ball"):
        rig.state().axis("ball")


def test_zeroing_moves_the_api_origin(rig):
    after = rig.zero_position()
    assert abs(after.axis("wheel").position_cm) < 1.0


def test_the_stream_is_decimated_and_says_so(rig):
    """The daemon samples at 500 Hz and is asked for 5, so `seq` steps by about
    a hundred per frame. That gap is decimation and not loss — which is exactly
    why a consumer breaks its line on `lost_before` instead."""
    frames = rig.watch_state(rate_hz=5)
    samples = []
    for frame in frames:
        assert isinstance(frame, (Sample, ZoneHit))
        if isinstance(frame, Sample):
            samples.append(frame)
        if len(samples) == 4:
            break
    frames.close()

    steps = [b.seq - a.seq for a, b in zip(samples, samples[1:])]
    assert all(step > 10 for step in steps), steps
    assert all(sample.lost_before == 0 for sample in samples[1:])


def test_a_stored_set_reads_back_as_types(rig):
    assert "goal" in rig.zone_sets()
    zone = rig.zone_set("goal").zones[0]
    assert zone.min_cm == (Reference("goal_cm"),)
    assert zone.max_cm == (None,)  # an open end, not a zero


def test_the_same_set_reads_back_as_the_file_it_is(rig):
    """Two spellings, both real. The file is the one a person edits, and it has
    to come back as the text on disk — `mousewheel set goal > goal.json` has to
    produce a file the daemon would accept."""
    file = rig.zone_set_file("goal")
    assert file.name == "goal"
    assert '"$goal_cm"' in file.text
    assert rig.check_zone_set_file(file.text).ok


def test_a_file_that_does_not_parse_is_refused_with_where(rig):
    with pytest.raises(Refused) as refusal:
        rig.check_zone_set_file('{"zone_set_version": 1, "zones": [{"nam": "goal"}]}')
    assert refusal.value.error == "zone_set_unreadable"
    assert "unknown field `nam`" in refusal.value.detail
    assert "line 1 column" in refusal.value.detail


def test_a_parameterised_set_names_what_it_needs(rig):
    report = rig.check_zone_set("goal")
    assert report.ok
    assert report.references == ("goal_cm",)
    assert report.counts_per_cm > 0


def test_arming_resolves_the_patch(disarmed):
    armed = disarmed.arm("goal", patch={"goal_cm": 180}, label="trial 42")
    assert armed.zone_set == "goal"
    assert armed.label == "trial 42"
    assert armed.arm_id is not None

    zone = armed.zones[0]
    assert zone.armed
    # Resolved, and converted back from the counts the board holds — so it is
    # near 180 rather than exactly 180.
    assert zone.min_cm[0] == pytest.approx(180, abs=0.1)
    assert zone.max_cm == (None,)


def test_arming_without_the_patch_is_refused_by_name(disarmed):
    """Never a default. A goal distance that silently became 0 cm is a trial
    that looks like it ran."""
    with pytest.raises(Refused) as refusal:
        disarmed.arm("goal")
    assert refusal.value.error == "unresolved_reference"
    assert "$goal_cm" in refusal.value.detail
    assert refusal.value.context == "goal"
    assert not refusal.value.retryable


def test_arming_absolute_keeps_the_devices_origin(disarmed):
    armed = disarmed.arm("goal", patch={"goal_cm": 180}, origin=ArmOrigin.ABSOLUTE)
    assert armed.zones[0].armed


def test_disarming_leaves_nothing_armed(disarmed):
    disarmed.arm("goal", patch={"goal_cm": 180})
    after = disarmed.disarm()
    assert after.zones == ()
    assert after.zone_set is None
    assert after.arm_id is None


def test_a_missing_set_is_not_found_and_says_which(rig):
    with pytest.raises(Refused) as refusal:
        rig.zone_set("corridor")
    assert refusal.value.status == "not_found"
    assert refusal.value.error == "no_such_zone_set"
    assert refusal.value.context == "corridor"


def test_a_name_that_is_a_path_is_refused_as_a_name(rig):
    """`../../etc/passwd` is a refusal naming the rule it broke, not a
    traversal."""
    with pytest.raises(Refused) as refusal:
        rig.zone_set("../../etc/passwd")
    assert refusal.value.error == "bad_zone_set_name"


def test_the_two_dimensional_ball_is_unimplemented_rather_than_absent(rig):
    """Modelled and not built. An rpc that answers `unimplemented` is a promise
    with a date on it; one that does not exist is a redesign."""
    with pytest.raises(Refused) as refusal:
        rig.ball()
    assert refusal.value.status == "unimplemented"


def test_applying_a_measurement_nobody_took_is_refused(rig):
    with pytest.raises(Refused) as refusal:
        rig.apply_measurement()
    assert refusal.value.status == "failed_precondition"
    assert refusal.value.error == "nothing_measured"


def test_finishing_a_measurement_nothing_moved_for_is_refused(rig):
    """A wheel that did not turn is not a calibration of zero counts per
    centimetre; it is a measurement that did not happen."""
    rig.start_measuring("wheel", 100.0)
    with pytest.raises(Refused) as refusal:
        rig.finish_measuring()
    assert refusal.value.error == "too_few_counts"


def test_a_measurement_reports_against_what_is_configured(rig):
    started = rig.start_measuring("wheel", 100.0)
    assert started.axis == "wheel"
    assert started.known_distance_cm == 100.0

    # Let the simulated wheel turn: a measurement is a distance actually
    # rolled, and the daemon refuses one that covered nothing.
    deadline = time.monotonic() + 5
    while rig.state().axis("wheel").counts - started.counts_at_start < 1000:
        if time.monotonic() > deadline:
            pytest.fail("the simulated wheel never turned")
        time.sleep(0.05)

    result = rig.finish_measuring()
    assert result.axis == "wheel"
    assert result.configured_counts_per_cm > 0
    assert result.measured_counts_per_cm > 0
    # Not applied: applying invalidates every compiled set and disarms, and
    # this suite's other tests are entitled to the calibration they started
    # with. That it is three presses and not one is the point of the design.


def test_the_config_says_whether_the_segment_is_open(rig):
    config = rig.config()
    assert config.rate_hz > 0
    # The pair worth reading: a name in a config file and a mapped segment are
    # different things.
    assert config.shm_name
    assert config.shm_open


def test_a_config_patch_changes_only_what_it_names(rig):
    before = rig.config()
    after = rig.set_config(ring_minutes=before.ring_minutes + 1)
    assert after.ring_minutes == before.ring_minutes + 1
    assert after.rate_hz == before.rate_hz
    rig.set_config(ring_minutes=before.ring_minutes)


def test_the_lines_come_from_the_rig_config(rig):
    lines = rig.lines()
    assert lines
    assert any(line.name == "zone_goal" for line in lines)


def test_the_wire_log_holds_the_conversation(rig):
    lines = rig.wire_log()
    assert any("hello" in line.text for line in lines)


def test_a_daemon_that_is_not_there_is_not_connected():
    """The one refusal a script may legitimately wait on, and it has its own
    class so that waiting on it does not mean catching everything."""
    from mousewheeld import Rig

    with Rig("127.0.0.1:1") as absent, pytest.raises(NotConnected) as refusal:
        absent.state()
    assert refusal.value.retryable
