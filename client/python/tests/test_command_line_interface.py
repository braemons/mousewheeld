# SPDX-License-Identifier: AGPL-3.0-or-later
"""`mousewheelctl` keeps the family's rules for a `<name>ctl`.

`contracts/DAEMON_LAYOUT.md` lists them. These are the ones that need no
daemon: which rig, the version, and how a failure reads to a script.
"""

from __future__ import annotations

import json
import socket

import pytest

from mousewheeld_client.command_line_interface import ExitStatus, build_parser, main


def a_port_nobody_listens_on() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def test_the_rig_is_the_flag_then_the_environment_then_localhost(monkeypatch):
    monkeypatch.delenv("BRAEMONS_RIG", raising=False)
    assert build_parser().parse_args(["state"]).rig == "localhost"
    monkeypatch.setenv("BRAEMONS_RIG", "rig-a.local")
    assert build_parser().parse_args(["state"]).rig == "rig-a.local"
    assert build_parser().parse_args(["--rig", "rig-b", "state"]).rig == "rig-b"


def test_the_version_is_the_commands_own(capsys):
    with pytest.raises(SystemExit) as exit:
        main(["--version"])
    assert exit.value.code == 0
    assert capsys.readouterr().out.startswith("mousewheelctl ")


def test_nothing_answering_is_unavailable_as_json_on_stderr(capsys):
    port = a_port_nobody_listens_on()
    status = main(["--rig", f"127.0.0.1:{port}", "--timeout", "0.2", "state"])
    assert status == ExitStatus.UNAVAILABLE == 3
    failure = json.loads(capsys.readouterr().err)
    assert failure["error"] == "unavailable"
    assert failure["detail"]


def test_a_document_from_stdin_needs_a_name():
    with pytest.raises(SystemExit) as exit:
        main(["sets", "put", "-"])
    assert exit.value.code == ExitStatus.USAGE == 2


def test_a_calibration_change_names_only_what_it_sets():
    arguments = build_parser().parse_args(["calibration", "set", "wheel", "--no-invert"])
    assert (arguments.axis, arguments.invert, arguments.counts_per_cm) == ("wheel", False, None)
    assert build_parser().parse_args(["calibration"]).action is None


def test_a_position_takes_centimetres_and_optionally_axes():
    arguments = build_parser().parse_args(["position", "120.5", "--axes", "wheel"])
    assert (arguments.position_cm, arguments.axes) == ([120.5], ["wheel"])


# -- against a daemon ------------------------------------------------------------


def mousewheelctl(wheel, capsys, *arguments: str):
    status = main(["--rig", wheel.target, *arguments])
    captured = capsys.readouterr()
    return status, captured.out, captured.err


def test_a_rate_is_patched_and_put_back(wheel, capsys):
    before = wheel.config()
    status, out, _ = mousewheelctl(
        wheel, capsys, "rig-config", "set", "--ring-minutes", str(before.ring_minutes + 1)
    )
    assert status == ExitStatus.OK
    assert json.loads(out)["ring_minutes"] == before.ring_minutes + 1
    wheel.set_config(ring_minutes=before.ring_minutes)


def test_one_axis_is_recalibrated_and_put_back(wheel, capsys):
    axis = wheel.calibration().axes[0]
    status, out, _ = mousewheelctl(
        wheel, capsys, "calibration", "set", axis.name,
        "--counts-per-cm", str(axis.counts_per_cm * 2),
    )  # fmt: skip
    assert status == ExitStatus.OK
    changed = {one["name"]: one for one in json.loads(out)["axes"]}[axis.name]
    assert changed["counts_per_cm"] == axis.counts_per_cm * 2
    mousewheelctl(
        wheel, capsys, "calibration", "set", axis.name, "--counts-per-cm", str(axis.counts_per_cm)
    )


def test_the_ball_is_refused_as_unimplemented(wheel, capsys):
    status, _, err = mousewheelctl(wheel, capsys, "ball")
    assert status == ExitStatus.REFUSED
    assert json.loads(err)["status"] == "unimplemented"


def test_firmware_and_the_wire_log_print(wheel, capsys):
    assert mousewheelctl(wheel, capsys, "firmware")[0] == ExitStatus.OK
    status, out, _ = mousewheelctl(wheel, capsys, "wire-log")
    assert status == ExitStatus.OK
    assert all(json.loads(line) for line in out.splitlines())


def test_the_position_is_set_and_the_state_comes_back(wheel, capsys):
    # Not compared to 42: the simulated wheel keeps rolling between the rpc and
    # the assertion, and how far depends on the machine running the test.
    status, out, _ = mousewheelctl(wheel, capsys, "position", "42")
    assert status == ExitStatus.OK
    assert json.loads(out)["axes"]
