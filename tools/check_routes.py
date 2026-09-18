#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Is every route this daemon serves declared in the proto, and vice versa?

`proto/mousewheeld/v1/` is the interface (`contracts/DAEMON_LAYOUT.md`), and an
interface nothing checks is a wish. The generated types keep the *shapes*
honest on their own — a field renamed in the proto stops compiling in the
daemon. What nothing catches is a route: a handler wired into the router with
no rpc above it is a piece of public API that exists and is written down
nowhere, and an rpc whose route was never wired is a promise to a client that
404s.

So this reads both sides and compares them:

  * every `Router` under `daemon/src/api/`, which is the truth about what is
    served — the main table in `mod.rs` and the ones merged into it;
  * the `option (braemons.v1.route)` on every rpc, which is the truth about
    what is promised.

It reads the `.proto` files as text rather than a descriptor set, deliberately.
A custom option is only legible in a descriptor with the protobuf runtime and
the generated extension to hand, and this check has to run in CI on a machine
with nothing installed but `python3` — the same reason `check_outcomes.py` is
regexes over source files.

Routes that are not API are not declared and not expected: the console
elements, the development page, and `/api/openapi.json` while it still exists.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
#: Every file under `api/`, not only the one holding the main router: elements
#: and the proto files build their own `Router` and are merged in, and a route
#: this check could not see is exactly the route it exists to catch.
API_DIR = HERE / "daemon" / "src" / "api"
PROTO_DIR = HERE / "proto" / "mousewheeld" / "v1"

#: Served, and deliberately not rpcs.
#:
#: The elements and the development page are a UI contract, not an interface.
#: `/api/proto` serves the interface *files* — it is how the rpcs are read, so
#: it cannot be one of them without describing itself.
NOT_API = {
    ("GET", "/"),
    ("GET", "/elements/{}"),
    ("GET", "/api/proto"),
    ("GET", "/api/proto/{}"),
}

#: A `.route("path", …)` call runs until the next one, which is what lets the
#: handlers be split over several lines — as the long paths here are.
ROUTE_CALL = re.compile(r'\.route\(\s*"(?P<path>[^"]+)"\s*,', re.DOTALL)
HANDLER_VERB = re.compile(r"\b(get|post|put|patch|delete)\s*\(")

#: Up to the `}` that ends the option, not the first `}` in it: a path like
#: `/api/zone-sets/{name}` contains one, and matching `[^}]*` stops there.
OPTION_BLOCK = re.compile(
    r"option\s*\(braemons\.v1\.route\)\s*=\s*\{(?P<body>.*?)\}\s*;", re.DOTALL
)
RPC_NAME = re.compile(r"\brpc\s+(\w+)\s*\(")


def routes_the_router_serves() -> set[tuple[str, str]]:
    found: set[tuple[str, str]] = set()
    for source in sorted(API_DIR.glob("*.rs")):
        text = source.read_text()
        calls = list(ROUTE_CALL.finditer(text))
        for index, match in enumerate(calls):
            end = calls[index + 1].start() if index + 1 < len(calls) else len(text)
            handlers = text[match.end() : end]
            for verb in HANDLER_VERB.findall(handlers):
                found.add((verb.upper(), normalise(match.group("path"))))
    return found


def routes_the_proto_declares() -> dict[tuple[str, str], str]:
    declared: dict[tuple[str, str], str] = {}
    for proto in sorted(PROTO_DIR.glob("*.proto")):
        text = proto.read_text()
        for option in OPTION_BLOCK.finditer(text):
            body = option.group("body")
            method = field(body, "method")
            path = field(body, "path")
            if not method or not path:
                print(f"{proto.name}: a route option with no method or path")
                continue
            # The nearest `rpc` above this option is the one it belongs to.
            before = text[: option.start()]
            names = RPC_NAME.findall(before)
            declared[(method.upper(), normalise(path))] = names[-1] if names else "?"
    return declared


def field(body: str, name: str) -> str:
    match = re.search(rf'{name}\s*:\s*"([^"]*)"', body)
    return match.group(1) if match else ""


def normalise(path: str) -> str:
    """A path parameter is a path parameter, whatever either side calls it.

    axum writes `{name}` and `{relative_path:path}`; the proto writes `{name}`.
    Comparing the spellings would fail on a rename that changed nothing.
    """
    return re.sub(r"\{[^}]*\}", "{}", path)


def main() -> int:
    served = {r for r in routes_the_router_serves() if r not in NOT_API}
    declared = routes_the_proto_declares()

    undeclared = sorted(served - set(declared))
    unserved = sorted(set(declared) - served)

    for method, path in undeclared:
        print(f"served but not in the proto:   {method:6} {path}")
    for method, path in unserved:
        print(f"in the proto but not served:   {method:6} {path}  ({declared[(method, path)]})")

    if undeclared or unserved:
        print()
        print("every route this daemon serves is part of its public interface,")
        print("and its public interface is proto/mousewheeld/v1/. Add the rpc, or")
        print("add the route to NOT_API in this file and say why.")
        return 1

    print(f"the router and the proto agree on {len(served)} routes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
