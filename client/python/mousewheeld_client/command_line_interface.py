# SPDX-License-Identifier: AGPL-3.0-or-later
"""`mousewheelctl` — one rig's wheel, from a terminal.

Named without the `d`: the daemon is `mousewheeld` and it installs a binary of
that name on every rig this would also be installed on. Two different programs
answering to one word is a bug report about the wrong one.

**This is a view onto the client, and holds no logic of its own.** Anything it
can work out, `MousewheeldClient` could have; anything it decides would be a
second opinion about a rig that already has one.

It follows the family's rules for a `<name>ctl` (`contracts/DAEMON_LAYOUT.md`):
`--rig`, then `$BRAEMONS_RIG`, then localhost; JSON on stdout, one compact
object per line for a stream; a failure as one JSON object on stderr, with an
exit status a script can switch on.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections.abc import Sequence
from dataclasses import asdict, is_dataclass
from enum import Enum, IntEnum
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

from .api_types import ArmOrigin, AxisCalibrationPatch, BallCalibration, Sample, ZoneHit
from .daemon_client import DEFAULT_PORT, MousewheeldClient
from .daemon_refusals import DaemonRefusedTheRequest

RIG_ENVIRONMENT_VARIABLE = "BRAEMONS_RIG"


class ExitStatus(IntEnum):
    """The same numbers from every `<name>ctl` in the family."""

    OK = 0
    FAILURE = 1
    USAGE = 2
    UNAVAILABLE = 3
    TIMED_OUT = 4
    REFUSED = 5
    NOT_FOUND = 6
    INTERRUPTED = 130


_EXIT_STATUS_FOR_REFUSAL = {
    "unavailable": ExitStatus.UNAVAILABLE,
    "deadline_exceeded": ExitStatus.TIMED_OUT,
    "not_found": ExitStatus.NOT_FOUND,
}


def _plain(value):
    """A dataclass tree as JSON. Enums print as their short name, which is the
    one this client speaks and the one a zone-set file uses."""
    if is_dataclass(value) and not isinstance(value, type):
        return {name: _plain(field) for name, field in asdict(value).items()}
    if isinstance(value, Enum):
        return value.value
    if isinstance(value, list | tuple):
        return [_plain(item) for item in value]
    if isinstance(value, dict):
        return {key: _plain(item) for key, item in value.items()}
    return value


def _print(value) -> None:
    print(json.dumps(_plain(value), indent=2))


def _print_line(value) -> None:
    """One compact object per line, flushed — so a pipe into `jq` prints as it
    goes rather than when the stream ends."""
    print(json.dumps(_plain(value)), flush=True)


def _fail(error: str, detail: str, exit_status: ExitStatus, **more) -> int:
    print(json.dumps({"error": error, "detail": detail, **more}), file=sys.stderr)
    return exit_status


def _client_version() -> str:
    try:
        return version("mousewheeld-client")
    except PackageNotFoundError:
        return "unknown"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="mousewheelctl",
        description="Talk to a mousewheeld. Everything prints JSON, so it pipes into jq.",
    )
    parser.add_argument(
        "-V", "--version", action="version", version=f"mousewheelctl {_client_version()}"
    )
    parser.add_argument(
        "--rig",
        default=os.environ.get(RIG_ENVIRONMENT_VARIABLE) or "localhost",
        help=f"host, or host:port (default ${RIG_ENVIRONMENT_VARIABLE}, then localhost; "
        f"port {DEFAULT_PORT})",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=5.0,
        metavar="SECONDS",
        help="how long to wait for the daemon to answer (default %(default)s)",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    commands.add_parser("state", help="one reading of every axis")
    watch = commands.add_parser("watch", help="follow the state stream, one object per line")
    watch.add_argument("--rate-hz", type=int, default=10)
    watch.add_argument("-n", "--frames", type=int, help="stop after this many")
    watch.add_argument("--summary", action="store_true", help="one line of text per frame")

    commands.add_parser("version", help="the daemon, the API, the device protocol")
    commands.add_parser("device", help="board, firmware, capacities, link")
    commands.add_parser("firmware", help="what the board is running, and what it should be")
    commands.add_parser("connect", help="open the link to the board")
    rig_config = commands.add_parser(
        "rig-config", help="rates, the shared-memory segment, the port"
    )
    rig_config_actions = rig_config.add_subparsers(dest="action")
    rig_config_set = rig_config_actions.add_parser(
        "set", help="change a rate until the daemon restarts; /etc is never written"
    )
    rig_config_set.add_argument("--rate-hz", type=int)
    rig_config_set.add_argument("--display-hz", type=int)
    rig_config_set.add_argument("--ring-minutes", type=int)
    commands.add_parser("lines", help="the output line map")

    calibration = commands.add_parser("calibration", help="counts per centimetre, per axis")
    calibration_actions = calibration.add_subparsers(dest="action")
    calibration_set = calibration_actions.add_parser(
        "set", help="change what is named on one axis, and leave the rest alone"
    )
    calibration_set.add_argument("axis")
    calibration_set.add_argument("--counts-per-cm", type=float)
    calibration_set.add_argument("--counts-per-rev", type=int)
    calibration_set.add_argument("--diameter-cm", type=float)
    calibration_set.add_argument("--invert", action=argparse.BooleanOptionalAction)
    ball = commands.add_parser("ball", help="the 2-D ball's calibration")
    ball_actions = ball.add_subparsers(dest="action")
    ball_set = ball_actions.add_parser("set", help="replace it")
    ball_set.add_argument("--diameter-cm", type=float, required=True)
    ball_set.add_argument(
        "--sensor-angles-deg", type=float, nargs="+", required=True, metavar="DEGREES"
    )

    commands.add_parser("wire", help="follow the serial conversation, both directions")
    commands.add_parser("wire-log", help="the serial conversation's recent past, then exit")

    inject = commands.add_parser("inject", help="motion nobody ran, to test the rig downstream")
    inject.add_argument("speed_cm_s", type=float, help="signed; positive is forward")
    inject.add_argument("--ms", type=int, default=2000, help="how long; 0 ends a running injection")
    inject.add_argument("--axis", default="", help="the first if none named")
    zero = commands.add_parser("zero", help="move the API origin")
    zero.add_argument("axes", nargs="*", help="which axes; all of them if none named")
    position = commands.add_parser(
        "position", help="move the API origin so that the wheel reads these centimetres"
    )
    position.add_argument("position_cm", type=float, nargs="+", metavar="CM")
    position.add_argument(
        "--axes", nargs="+", default=[], metavar="AXIS", help="which axes; all if none named"
    )

    sets = commands.add_parser("sets", help="the zone-set store")
    set_actions = sets.add_subparsers(dest="action", required=True)
    set_actions.add_parser("list", help="the names in the store")
    set_get = set_actions.add_parser("get", help="one set, as the file it is stored as")
    set_get.add_argument("name")
    set_put = set_actions.add_parser("put", help="store a zone-set file")
    set_put.add_argument("file", help="the file to store, or - for stdin")
    set_put.add_argument("--name", help="default: the file's stem")
    set_check = set_actions.add_parser("check", help="compile against the calibration in force")
    set_check.add_argument("name", nargs="?", help="a stored set; omit and pass --file for a draft")
    set_check.add_argument("--file", help="a zone-set file to check without storing it")
    set_actions.add_parser("schema", help="the JSON Schema of a zone-set file")

    arm = commands.add_parser("arm", help="arm a stored set")
    arm.add_argument("name")
    arm.add_argument(
        "--patch",
        action="append",
        default=[],
        metavar="NAME=CM",
        help="fill in a $name, e.g. --patch goal_cm=180. Repeatable",
    )
    arm.add_argument("--absolute", action="store_true", help="the device's origin, not this moment's")
    arm.add_argument("--label", help="opaque text, stored and handed back unread")
    commands.add_parser("armed", help="what is armed, what fired, and where")
    commands.add_parser("disarm", help="arm nothing")
    commands.add_parser("flash", help="write the armed set to the board's flash")

    measure = commands.add_parser("measure", help="the guided calibration, one step at a time")
    measure.add_argument("step", choices=["start", "finish", "apply"])
    measure.add_argument("--axis", default="wheel")
    measure.add_argument("--cm", type=float, default=100.0, help="the distance about to be rolled")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    arguments = parser.parse_args(argv)
    if arguments.command == "sets" and arguments.action == "put":
        if arguments.file == "-" and not arguments.name:
            parser.error("put - reads stdin and needs --name")
    if arguments.command == "sets" and arguments.action == "check":
        if not (arguments.name or arguments.file):
            parser.error("check needs a stored set's name, or --file")

    try:
        with MousewheeldClient(arguments.rig) as rig:
            # Before anything else, so that "nothing is listening" is one clear
            # sentence rather than whatever gRPC says about a connection.
            rig.wait_until_ready(timeout_s=arguments.timeout)
            return _run(rig, arguments)
    except DaemonRefusedTheRequest as refusal:
        # The refusal names what to change: its kind for a script, its sentence
        # for a person, and never a traceback through generated gRPC code.
        return _fail(
            refusal.error,
            refusal.detail,
            _EXIT_STATUS_FOR_REFUSAL.get(refusal.status, ExitStatus.REFUSED),
            context=refusal.context,
            status=refusal.status,
        )
    except TimeoutError as problem:
        return _fail("unavailable", str(problem), ExitStatus.UNAVAILABLE)
    except OSError as problem:
        return _fail("unreadable", str(problem), ExitStatus.FAILURE)
    except ValueError as problem:
        return _fail("usage", str(problem), ExitStatus.USAGE)
    except (KeyboardInterrupt, BrokenPipeError):
        return ExitStatus.INTERRUPTED


def _run(rig: MousewheeldClient, arguments) -> int:
    match arguments.command:
        case "state":
            _print(rig.state())
        case "watch":
            _watch(rig, arguments)
        case "version":
            _print(rig.version())
        case "device":
            _print(rig.device())
        case "firmware":
            _print(rig.firmware())
        case "connect":
            _print(rig.open_link())
        case "rig-config" if arguments.action == "set":
            _print(
                rig.set_config(
                    rate_hz=arguments.rate_hz,
                    display_hz=arguments.display_hz,
                    ring_minutes=arguments.ring_minutes,
                )
            )
        case "rig-config":
            _print(rig.config())
        case "lines":
            _print(rig.lines())
        case "calibration" if arguments.action == "set":
            _print(
                rig.set_calibration(
                    AxisCalibrationPatch(
                        name=arguments.axis,
                        counts_per_cm=arguments.counts_per_cm,
                        counts_per_rev=arguments.counts_per_rev,
                        diameter_cm=arguments.diameter_cm,
                        invert=arguments.invert,
                    )
                )
            )
        case "calibration":
            _print(rig.calibration())
        case "ball" if arguments.action == "set":
            _print(
                rig.set_ball(
                    BallCalibration(
                        diameter_cm=arguments.diameter_cm,
                        sensor_angles_deg=tuple(arguments.sensor_angles_deg),
                    )
                )
            )
        case "ball":
            _print(rig.ball())
        case "wire":
            for line in rig.watch_wire():
                _print_line(line)
        case "wire-log":
            for line in rig.wire_log():
                _print_line(line)
        case "inject":
            _print(rig.inject_motion(arguments.speed_cm_s, arguments.ms, arguments.axis))
        case "zero":
            _print(rig.zero_position(tuple(arguments.axes)))
        case "position":
            _print(rig.set_position(tuple(arguments.position_cm), tuple(arguments.axes)))
        case "sets":
            return _sets(rig, arguments)
        case "arm":
            _print(
                rig.arm(
                    arguments.name,
                    patch=_patch(arguments.patch),
                    origin=ArmOrigin.ABSOLUTE if arguments.absolute else ArmOrigin.CURRENT,
                    label=arguments.label,
                )
            )
        case "armed":
            _print(rig.armed())
        case "disarm":
            _print(rig.disarm())
        case "flash":
            _print(rig.save_to_flash())
        case "measure":
            _measure(rig, arguments)
    return ExitStatus.OK


def _sets(rig: MousewheeldClient, arguments) -> int:
    match arguments.action:
        case "list":
            _print(rig.zone_sets())
        case "get":
            # The file, printed as it is on disk rather than re-encoded:
            # `sets get goal > goal.json` has to produce a file the daemon
            # would accept back.
            print(rig.zone_set_file(arguments.name).text, end="")
        case "put":
            name = arguments.name or Path(arguments.file).stem
            _print(rig.set_zone_set_file(name, _read(arguments.file)))
        case "check":
            if arguments.file:
                report = rig.check_zone_set_file(_read(arguments.file), arguments.name or "")
            else:
                report = rig.check_zone_set(arguments.name)
            _print(report)
            return ExitStatus.OK if report.ok else ExitStatus.REFUSED
        case "schema":
            _print(rig.zone_set_file_schema())
    return ExitStatus.OK


def _watch(rig: MousewheeldClient, arguments) -> None:
    seen = 0
    for frame in rig.watch_state(rate_hz=arguments.rate_hz):
        if arguments.summary:
            print(_summary(frame), flush=True)
        else:
            kind = "sample" if isinstance(frame, Sample) else "zone_hit"
            print(json.dumps({"kind": kind, **_plain(frame)}), flush=True)
        seen += 1
        if arguments.frames is not None and seen >= arguments.frames:
            return


def _summary(frame) -> str:
    if isinstance(frame, ZoneHit):
        return f"zone hit  {frame.zone} at {frame.position_cm:.2f} cm"
    return "  ".join(f"{axis.name} {axis.position_cm:9.2f} cm" for axis in frame.axes)


def _measure(rig: MousewheeldClient, arguments) -> None:
    match arguments.step:
        case "start":
            _print(rig.start_measuring(arguments.axis, arguments.cm))
        case "finish":
            _print(rig.finish_measuring())
        case "apply":
            _print(rig.apply_measurement())


def _patch(pairs: list[str]) -> dict[str, float]:
    patch = {}
    for pair in pairs:
        name, separator, value = pair.partition("=")
        if not separator:
            raise ValueError(f"--patch takes NAME=CM, not {pair!r}")
        patch[name] = float(value)
    return patch


def _read(path: str) -> str:
    return sys.stdin.read() if path == "-" else Path(path).read_text(encoding="utf-8")


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
