# SPDX-License-Identifier: AGPL-3.0-or-later
"""`mousewheel` — the rig, from a terminal.

Named without the `d`: the daemon is `mousewheeld` and it installs a binary of
that name on every rig this would also be installed on. Two different programs
answering to one word is a bug report about the wrong one.

**This is a view onto the client, and holds no logic of its own.** Anything it
can work out, `mousewheeld.MousewheeldClient` could have; anything it decides would be a
second opinion about a rig that already has one.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Sequence
from dataclasses import asdict, is_dataclass
from enum import Enum

from . import MousewheeldClient
from .daemon_refusals import DaemonRefusedTheRequest
from .api_types import Sample, ZoneHit


def _plain(value):
    """A dataclass tree as JSON. Enums print as their short name, which is the
    one this client speaks and the one a zone-set file uses."""
    if is_dataclass(value) and not isinstance(value, type):
        return {name: _plain(field) for name, field in asdict(value).items()}
    if isinstance(value, Enum):
        return value.value
    if isinstance(value, (list, tuple)):
        return [_plain(item) for item in value]
    if isinstance(value, dict):
        return {key: _plain(item) for key, item in value.items()}
    return value


def _print(value) -> None:
    print(json.dumps(_plain(value), indent=2))


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="mousewheel",
        description="Talk to a mousewheeld. Everything prints JSON, so it pipes into jq.",
    )
    parser.add_argument(
        "--rig",
        default="localhost",
        help="host, or host:port (default port 8082)",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    commands.add_parser("version", help="the daemon, the API, the device protocol")
    commands.add_parser("device", help="board, firmware, capacities, link")
    commands.add_parser("connect", help="open the link to the board")
    commands.add_parser("state", help="one reading of every axis")
    commands.add_parser("config", help="rates, the shared-memory segment, the port")
    commands.add_parser("lines", help="the output line map")
    commands.add_parser("calibration", help="counts per centimetre, per axis")
    commands.add_parser("armed", help="what is armed, what fired, and where")
    commands.add_parser("sets", help="the names in the zone-set store")
    commands.add_parser("schema", help="the JSON Schema of a zone-set file")

    zero = commands.add_parser("zero", help="move the API origin")
    zero.add_argument("axes", nargs="*", help="which axes; all of them if none named")

    show = commands.add_parser("set", help="one zone set, as the file it is stored as")
    show.add_argument("name")

    check = commands.add_parser("check", help="compile a set against the calibration in force")
    check.add_argument("name", nargs="?", help="a stored set; omit and pass --file for a draft")
    check.add_argument("--file", help="a zone-set file to check without storing it")

    put = commands.add_parser("put", help="store a zone-set file")
    put.add_argument("name")
    put.add_argument("file", help="the file to store, or - for stdin")

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

    commands.add_parser("disarm")
    commands.add_parser("flash", help="write the armed set to the board's flash")

    watch = commands.add_parser("watch", help="follow the state stream, one JSON object per line")
    watch.add_argument("--rate-hz", type=int, default=10)
    watch.add_argument("-n", "--frames", type=int, help="stop after this many")

    commands.add_parser("wire", help="follow the serial conversation, both directions")

    measure = commands.add_parser("measure", help="the guided calibration, one step at a time")
    measure.add_argument("step", choices=["start", "finish", "apply"])
    measure.add_argument("--axis", default="wheel")
    measure.add_argument("--cm", type=float, default=100.0, help="the distance about to be rolled")

    arguments = parser.parse_args(argv)

    try:
        with MousewheeldClient(arguments.rig) as rig:
            return _run(rig, arguments)
    except DaemonRefusedTheRequest as refusal:
        # The refusal names what to change, so print that and not a traceback:
        # a stack trace through generated gRPC code tells a person nothing they
        # can act on.
        print(f"{refusal.error}: {refusal.detail}", file=sys.stderr)
        if refusal.context:
            print(f"  ({refusal.context})", file=sys.stderr)
        return 1
    except (KeyboardInterrupt, BrokenPipeError):
        return 130


def _run(rig: MousewheeldClient, arguments) -> int:
    command = arguments.command
    if command == "version":
        _print(rig.version())
    elif command == "device":
        _print(rig.device())
    elif command == "connect":
        _print(rig.open_link())
    elif command == "state":
        _print(rig.state())
    elif command == "config":
        _print(rig.config())
    elif command == "lines":
        _print(rig.lines())
    elif command == "calibration":
        _print(rig.calibration())
    elif command == "armed":
        _print(rig.armed())
    elif command == "sets":
        _print(rig.zone_sets())
    elif command == "schema":
        _print(rig.zone_set_file_schema())
    elif command == "zero":
        _print(rig.zero_position(tuple(arguments.axes)))
    elif command == "set":
        # The file, printed as it is on disk rather than re-encoded: `mousewheel
        # set goal > goal.json` has to produce a file the daemon would accept.
        print(rig.zone_set_file(arguments.name).text, end="")
    elif command == "check":
        if arguments.file:
            _print(rig.check_zone_set_file(_read(arguments.file), arguments.name or ""))
        elif arguments.name:
            _print(rig.check_zone_set(arguments.name))
        else:
            print("name a stored set, or pass --file", file=sys.stderr)
            return 2
    elif command == "put":
        _print(rig.set_zone_set_file(arguments.name, _read(arguments.file)))
    elif command == "arm":
        _print(
            rig.arm(
                arguments.name,
                patch=_patch(arguments.patch),
                origin=_origin(arguments.absolute),
                label=arguments.label,
            )
        )
    elif command == "disarm":
        _print(rig.disarm())
    elif command == "flash":
        _print(rig.save_to_flash())
    elif command == "watch":
        _watch(rig, arguments)
    elif command == "wire":
        for line in rig.watch_wire():
            print(f"{line.host_monotonic_ns} {line.direction.value:>3} {line.text}", flush=True)
    elif command == "measure":
        _measure(rig, arguments)
    return 0


def _watch(rig: MousewheeldClient, arguments) -> None:
    """One JSON object per line, flushed — so a pipe into `jq` prints as it
    goes rather than when the stream ends."""
    seen = 0
    for frame in rig.watch_state(rate_hz=arguments.rate_hz):
        kind = "sample" if isinstance(frame, Sample) else "zone_hit" if isinstance(frame, ZoneHit) else "?"
        print(json.dumps({"kind": kind, **_plain(frame)}), flush=True)
        seen += 1
        if arguments.frames is not None and seen >= arguments.frames:
            return


def _measure(rig: MousewheeldClient, arguments) -> None:
    if arguments.step == "start":
        _print(rig.start_measuring(arguments.axis, arguments.cm))
    elif arguments.step == "finish":
        _print(rig.finish_measuring())
    else:
        _print(rig.apply_measurement())


def _patch(pairs: list[str]) -> dict[str, float]:
    patch = {}
    for pair in pairs:
        name, _, value = pair.partition("=")
        if not _:
            raise SystemExit(f"--patch takes NAME=CM, not {pair!r}")
        patch[name] = float(value)
    return patch


def _origin(absolute: bool):
    from .api_types import ArmOrigin

    return ArmOrigin.ABSOLUTE if absolute else ArmOrigin.CURRENT


def _read(path: str) -> str:
    return sys.stdin.read() if path == "-" else open(path, encoding="utf-8").read()


if __name__ == "__main__":
    raise SystemExit(main())
