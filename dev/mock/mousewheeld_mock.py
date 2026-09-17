#!/usr/bin/env python3
"""A mock mousewheeld, so the web elements can be built before the daemon is.

**This is not the daemon and must never become it.** The daemon is one Rust
binary (`dev/PLAN.md`, *The daemon*); this is a few hundred lines of standard
library that answers the routes in *The API* with a simulated wheel behind
them, so `web/elements/` can be written, looked at and driven by hand at M0
instead of waiting for M2. It is deleted the day the real daemon serves
`/elements/`.

What it simulates, and each of these is here because a panel has to show it:

- an animal that runs in bouts and rests between them, so velocity is not a
  constant and the charts have something to be read against;
- **two velocities** — one the device reports, quantised to whole counts over a
  scan window, and one the host computes from the sample path. The trace panel
  draws both, and they must not be the same number or it is drawing a lie;
- **sequence gaps**: every few seconds a line is "lost", `seq` skips, and the
  trace draws a break rather than a straight line through a hole;
- **zones**: an armed set is evaluated on every tick, fires on entry, and the
  hit reaches the stream and the wire log;
- **the wire**, both directions, including the commands the API routes send.

Run it:

    python3 dev/mock/mousewheeld_mock.py           # http://127.0.0.1:8082

It serves `/elements/` from `web/elements/` and a development page at `/` that
mounts all five panels. Standard library only, no dependencies, Linux or macOS.
"""

from __future__ import annotations

import base64
import hashlib
import json
import math
import os
import random
import select
import struct
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

HERE = Path(__file__).resolve()
ELEMENTS_DIR = HERE.parent.parent.parent / "web" / "elements"
WEBSOCKET_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"

TICK_HZ = 200.0
COUNTS_PER_REV = 4096
DIAMETER_CM = 15.0

DEFAULT_ZONE_SETS = {
    "goal": {
        "zone_set_version": 3,
        "zones": [
            {
                "name": "goal",
                "shape": "rect",
                "axes": ["wheel"],
                "metric": "displacement",
                "min_cm": [200.0],
                "max_cm": [None],
                "fire": "once",
                "output": {"line": "zone_goal", "action": "pulse", "ms": 10},
            }
        ],
    },
    "corridor": {
        "zone_set_version": 1,
        "zones": [
            {
                "name": "reward_region",
                "shape": "rect",
                "axes": ["wheel"],
                "metric": "displacement",
                "min_cm": [120.0],
                "max_cm": [140.0],
                "wrap_cm": 300.0,
                "fire": "rearm",
                "hysteresis_cm": 2.0,
                "output": {"line": "zone_region", "action": "level"},
            },
            {
                "name": "landmark",
                "shape": "rect",
                "axes": ["wheel"],
                "metric": "displacement",
                "min_cm": [40.0],
                "max_cm": [60.0],
                "wrap_cm": 300.0,
                "fire": "rearm",
                "hysteresis_cm": 2.0,
                "output": {"line": "zone_goal", "action": "pulse", "ms": 5},
            },
        ],
    },
}


def monotonic_ns() -> int:
    return time.monotonic_ns()


class SimulatedRig:
    """The wheel, the board and the daemon's own bookkeeping, behind one lock."""

    def __init__(self) -> None:
        self.lock = threading.RLock()
        self.started = time.monotonic()
        self.connected = True
        self.counts_per_cm = COUNTS_PER_REV / (math.pi * DIAMETER_CM)
        self.invert = False
        self.measured_at = None

        self.counts = 0.0
        self.origin_counts = 0.0
        self.distance_counts = 0.0
        self.host_velocity_cm_s = 0.0
        self.device_velocity_cm_s = 0.0

        self.seq = 0
        self.seq_gaps = 0
        self.ring_drops = 0
        self.subscribers: list[callable] = []
        self.wire: list[dict] = []
        self.wire_subscribers: list[callable] = []

        self.zone_sets = json.loads(json.dumps(DEFAULT_ZONE_SETS))
        self.armed_name = None
        self.armed_version = None
        self.arm_id = None
        self.zone_state: dict[str, dict] = {}
        self.flashed_zone_set = None

        self.measuring = None
        self.measurement = None

        self.rate_hz = 500
        self.display_hz = 120

        self._history: list[tuple[float, float]] = []
        self._bout_until = 0.0
        self._resting = False
        self._target_speed = 0.0
        self._next_gap_at = time.monotonic() + 6.0

    # ---------------------------------------------------------- the wheel ---

    def tick(self) -> None:
        """One scan: move the animal, evaluate zones, publish a sample."""
        now = time.monotonic()
        with self.lock:
            if now >= self._bout_until:
                self._resting = not self._resting
                self._bout_until = now + (random.uniform(1.5, 3.0) if self._resting else random.uniform(4.0, 8.0))
                self._target_speed = 0.0 if self._resting else random.uniform(12.0, 38.0)

            wobble = 1.0 + 0.12 * math.sin(now * 5.3)
            speed_cm_s = max(0.0, self._target_speed * wobble)
            step_counts = speed_cm_s * self.counts_per_cm / TICK_HZ
            self.counts += step_counts
            self.distance_counts += abs(step_counts)

            # The host's velocity is the slope of the path it received; the
            # device's is whole counts over its own scan window, which is the
            # same motion through a coarser sieve. Two numbers, on purpose.
            self._history.append((now, self.counts))
            while self._history and now - self._history[0][0] > 0.5:
                self._history.pop(0)
            if len(self._history) > 1:
                span = now - self._history[0][0]
                self.host_velocity_cm_s = ((self.counts - self._history[0][1]) / self.counts_per_cm) / span
            window_counts = round(step_counts * (TICK_HZ / 20.0))
            self.device_velocity_cm_s = (window_counts * 20.0) / self.counts_per_cm

            self.seq += 1
            lost = 0
            if now >= self._next_gap_at:
                # Lines the daemon never received. `seq` skips, and — because a
                # decimated stream skips `seq` all the time by design — the
                # sample also carries how many were *lost*, which is the only
                # thing a consumer can tell a hole from.
                lost = 3
                self.seq += lost
                self.seq_gaps += 1
                self._next_gap_at = now + random.uniform(5.0, 11.0)

            hits = self._evaluate_zones()
            sample = self._sample_message()
            sample["lost_before"] = lost

        for hit in hits:
            self._publish({"type": "zone_hit", **hit})
            self.log_wire("in", json.dumps({"zone_hit": hit["zone"], "seq": hit["seq"]}))
        self._publish(sample)

    def _displacement_cm(self) -> float:
        return (self.counts - self.origin_counts) / self.counts_per_cm

    def _distance_cm(self) -> float:
        return self.distance_counts / self.counts_per_cm

    def _evaluate_zones(self) -> list[dict]:
        if self.armed_name is None:
            return []
        hits = []
        for zone in self.zone_sets.get(self.armed_name, {}).get("zones", []):
            state = self.zone_state.setdefault(zone["name"], {"inside": False, "fired": False, "armed": True})
            if not state["armed"]:
                continue
            value = self._distance_cm() if zone.get("metric") == "distance" else self._displacement_cm()
            wrap = zone.get("wrap_cm")
            if wrap:
                value = value % wrap
            low = zone.get("min_cm", [None])[0]
            high = zone.get("max_cm", [None])[0]
            inside = (low is None or value >= low) and (high is None or value <= high)
            if inside and not state["inside"]:
                state["fired"] = True
                hits.append(
                    {
                        "zone": zone["name"],
                        "arm_id": self.arm_id,
                        "seq": self.seq,
                        "host_monotonic_ns": monotonic_ns(),
                        "position_cm": self._displacement_cm(),
                    }
                )
                if zone.get("fire") == "once":
                    state["armed"] = False
            state["inside"] = inside
        return hits

    def _sample_message(self) -> dict:
        return {
            "type": "sample",
            "seq": self.seq,
            "device_us": int((time.monotonic() - self.started) * 1e6),
            "host_monotonic_ns": monotonic_ns(),
            "axes": [
                {
                    "name": "wheel",
                    "counts": int(self.counts),
                    "position_cm": self._displacement_cm(),
                    "distance_cm": self._distance_cm(),
                    "velocity_cm_s": self.host_velocity_cm_s,
                    "device_velocity_cm_s": self.device_velocity_cm_s,
                }
            ],
        }

    # ------------------------------------------------------- publications ---

    def _publish(self, message: dict) -> None:
        for send in list(self.subscribers):
            try:
                send(message)
            except Exception:
                pass

    def log_wire(self, direction: str, text: str, level: str = "info") -> None:
        line = {
            "host_monotonic_ns": monotonic_ns(),
            "direction": direction,
            "text": text,
            "level": level,
        }
        with self.lock:
            self.wire.append(line)
            if len(self.wire) > 4000:
                del self.wire[:2000]
        for send in list(self.wire_subscribers):
            try:
                send(line)
            except Exception:
                pass

    # ---------------------------------------------------------- API views ---

    def device(self) -> dict:
        with self.lock:
            return {
                "connected": self.connected,
                "port": "/dev/ttyACM0",
                "board": "teensy41",
                "firmware_version": "0.1.0+mock",
                "protocol_version": 1,
                "uptime_device_us": int((time.monotonic() - self.started) * 1e6),
                "capacities": {
                    "n_axes": 2,
                    "max_zones": 16,
                    "max_lines": 8,
                    "scan_hz": 5000,
                },
                "flashed_zone_set": self.flashed_zone_set,
                "link": {"connection_count": 1, "last_error": None},
            }

    def state(self) -> dict:
        with self.lock:
            return {
                "axes": [
                    {
                        "name": "wheel",
                        "counts": int(self.counts),
                        "position_cm": self._displacement_cm(),
                        "distance_cm": self._distance_cm(),
                        "velocity_cm_s": self.host_velocity_cm_s,
                        "device_velocity_cm_s": self.device_velocity_cm_s,
                    }
                ],
                "health": {
                    "seq_gaps": self.seq_gaps,
                    "ring_drops": self.ring_drops,
                    "measured_rate_hz": TICK_HZ,
                    "stale": not self.connected,
                },
                "link": {"connected": self.connected},
            }

    def calibration(self) -> dict:
        with self.lock:
            return {
                "axes": [
                    {
                        "name": "wheel",
                        "counts_per_cm": self.counts_per_cm,
                        "counts_per_rev": COUNTS_PER_REV,
                        "diameter_cm": DIAMETER_CM,
                        "invert": self.invert,
                        "measured_at": self.measured_at,
                    }
                ],
                "ball": None,
            }

    def armed(self) -> dict:
        with self.lock:
            if self.armed_name is None:
                return {"zone_set": None, "zones": []}
            return {
                "zone_set": self.armed_name,
                "zone_set_version": self.armed_version,
                "arm_id": self.arm_id,
                "zones": [
                    {
                        "name": name,
                        "armed": state["armed"],
                        "fired": state["fired"],
                        "inside": state["inside"],
                    }
                    for name, state in self.zone_state.items()
                ],
            }


RIG = SimulatedRig()


def wheel_thread() -> None:
    period = 1.0 / TICK_HZ
    next_tick = time.monotonic()
    while True:
        next_tick += period
        delay = next_tick - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        else:
            next_tick = time.monotonic()
        RIG.tick()


def wire_chatter_thread() -> None:
    """The wire as it would look: samples at rate, and the odd status line."""
    while True:
        time.sleep(0.05)
        state = RIG.state()["axes"][0]
        RIG.log_wire(
            "in",
            json.dumps({"sample": {"seq": RIG.seq, "c": [state["counts"]], "t_us": int(time.monotonic() * 1e6)}}),
        )
        if random.random() < 0.02:
            RIG.log_wire("out", json.dumps({"ping": {"message_id": random.randint(1, 9999)}}))
            RIG.log_wire("in", json.dumps({"pong": {"message_id": random.randint(1, 9999)}}))


DEV_PAGE = """<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>mousewheeld elements — mock</title>
    <style>
      body { font-family: system-ui, sans-serif; margin: 0; padding: 1.5rem;
             background: #eef1f4; color: #16202a; }
      h1 { font-size: 1.1rem; margin: 0 0 0.3rem; }
      p.lede { color: #667a8a; margin: 0 0 1.2rem; max-width: 60rem; }
      .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(30rem, 1fr));
              gap: 1rem; align-items: start; }
    </style>
    <script type="module" src="/elements/mousewheeld.js"></script>
  </head>
  <body>
    <h1>mousewheeld — the five panels, against the mock</h1>
    <p class="lede">
      This page stands in for the console: it does nothing but mount the elements the daemon
      serves, which is the whole point of the contract. The wheel behind them is simulated.
    </p>
    <div class="grid">
      <mousewheeld-device base=""></mousewheeld-device>
      <mousewheeld-trace base=""></mousewheeld-trace>
      <mousewheeld-calibration base=""></mousewheeld-calibration>
      <mousewheeld-zones base=""></mousewheeld-zones>
      <mousewheeld-monitor base=""></mousewheeld-monitor>
    </div>
  </body>
</html>
"""


class MockHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server_version = "mousewheeld-mock/0"

    def log_message(self, fmt, *args):  # quieter than the default
        pass

    # ------------------------------------------------------------ plumbing ---

    def _send_json(self, payload, status=200):
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _refuse(self, status, error, detail, context=""):
        self._send_json({"error": error, "detail": detail, "context": context}, status=status)

    def _body(self):
        length = int(self.headers.get("Content-Length") or 0)
        if length == 0:
            return {}
        try:
            return json.loads(self.rfile.read(length) or b"{}")
        except json.JSONDecodeError:
            return {}

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Content-Length", "0")
        self.end_headers()

    # ------------------------------------------------------------- routes ---

    def do_GET(self):
        url = urlparse(self.path)
        path = url.path

        if self.headers.get("Upgrade", "").lower() == "websocket":
            return self._serve_websocket(path, parse_qs(url.query))
        if path == "/":
            return self._send_html(DEV_PAGE)
        if path.startswith("/elements/"):
            return self._serve_element(path[len("/elements/"):])

        if path == "/api/device":
            return self._send_json(RIG.device())
        if path == "/api/device/firmware":
            return self._send_json({"running": "0.1.0+mock", "available": []})
        if path == "/api/device/monitor":
            with RIG.lock:
                return self._send_json({"lines": RIG.wire[-400:]})
        if path == "/api/state":
            return self._send_json(RIG.state())
        if path == "/api/calibration":
            return self._send_json(RIG.calibration())
        if path == "/api/calibration/ball":
            return self._refuse(501, "not_implemented", "ball calibration arrives with the hardware")
        if path == "/api/config":
            with RIG.lock:
                return self._send_json(
                    {
                        "rate_hz": RIG.rate_hz,
                        "display_hz": RIG.display_hz,
                        "shm_name": "/vstimd_wheel",
                        "ring_minutes": 30,
                        "event_port": 5557,
                    }
                )
        if path == "/api/lines":
            return self._send_json(
                {"lines": [{"name": "zone_goal", "index": 0, "pin": 5}, {"name": "zone_region", "index": 1, "pin": 6}]}
            )
        if path == "/api/zone-sets":
            with RIG.lock:
                return self._send_json({"zone_sets": sorted(RIG.zone_sets)})
        if path.startswith("/api/zone-sets/"):
            name = path[len("/api/zone-sets/"):]
            with RIG.lock:
                if name not in RIG.zone_sets:
                    return self._refuse(404, "no_such_zone_set", f"no zone set named {name}", name)
                return self._send_json(RIG.zone_sets[name])
        if path == "/api/zones":
            return self._send_json(RIG.armed())
        return self._refuse(404, "no_such_route", f"the mock has no {path}")

    def do_POST(self):
        path = urlparse(self.path).path
        body = self._body()

        if path == "/api/device/connect":
            with RIG.lock:
                RIG.connected = True
            RIG.log_wire("out", json.dumps({"hello": {"protocol_version": 1}}))
            RIG.log_wire("in", json.dumps({"hello_ack": {"board": "teensy41", "n_axes": 2}}))
            return self._send_json(RIG.device())

        if path == "/api/position/zero":
            with RIG.lock:
                RIG.origin_counts = RIG.counts
            RIG.log_wire("out", json.dumps({"zero": {}}))
            return self._send_json(RIG.state())

        if path == "/api/calibration/measure/start":
            with RIG.lock:
                RIG.measuring = {
                    "axis": body.get("axis", "wheel"),
                    "known_distance_cm": float(body.get("known_distance_cm", 100.0)),
                    "counts_at_start": RIG.counts,
                }
                return self._send_json({"counts": int(RIG.counts), **RIG.measuring})

        if path == "/api/calibration/measure/finish":
            with RIG.lock:
                if RIG.measuring is None:
                    return self._refuse(409, "not_measuring", "no measurement is in progress")
                counts = abs(RIG.counts - RIG.measuring["counts_at_start"])
                known = RIG.measuring["known_distance_cm"]
                RIG.measurement = {
                    "axis": RIG.measuring["axis"],
                    "known_distance_cm": known,
                    "counts": int(counts),
                    "measured_counts_per_cm": counts / known if known else 0.0,
                    "configured_counts_per_cm": RIG.counts_per_cm,
                    "counts_per_rev": COUNTS_PER_REV,
                }
                RIG.measuring = None
                return self._send_json(RIG.measurement)

        if path == "/api/calibration/measure/apply":
            with RIG.lock:
                if RIG.measurement is None:
                    return self._refuse(409, "nothing_measured", "finish a measurement before applying one")
                RIG.counts_per_cm = RIG.measurement["measured_counts_per_cm"]
                RIG.measured_at = time.strftime("%Y-%m-%d %H:%M")
                RIG.measurement = None
                # A calibration change marks every compiled set stale.
                RIG.armed_name = None
                return self._send_json({"counts_per_cm": RIG.counts_per_cm})

        if path.startswith("/api/zone-sets/") and path.endswith("/validate"):
            name = path[len("/api/zone-sets/"): -len("/validate")]
            with RIG.lock:
                zone_set = RIG.zone_sets.get(name)
                if zone_set is None:
                    return self._refuse(404, "no_such_zone_set", f"no zone set named {name}", name)
                problem = validate_zone_set(zone_set)
                return self._send_json(
                    {
                        "ok": problem is None,
                        "problem": problem,
                        "zone_count": len(zone_set.get("zones", [])),
                        "counts_per_cm": RIG.counts_per_cm,
                    }
                )

        if path == "/api/zones/arm":
            name = body.get("zone_set")
            with RIG.lock:
                if name not in RIG.zone_sets:
                    return self._refuse(404, "no_such_zone_set", f"no zone set named {name}", f"{name}")
                resolved = substitute(RIG.zone_sets[name], body.get("patch") or {})
                problem = validate_zone_set(resolved)
                if problem is not None:
                    return self._refuse(422, "zone_set_refused", problem, name)
                RIG.armed_name = name
                RIG.armed_version = resolved.get("zone_set_version")
                RIG.arm_id = random.randint(1000, 9999)
                RIG.zone_state = {
                    zone["name"]: {"inside": False, "fired": False, "armed": True}
                    for zone in resolved.get("zones", [])
                }
                if body.get("origin", "current") == "current":
                    RIG.origin_counts = RIG.counts
                    RIG.distance_counts = 0.0
            RIG.log_wire("out", json.dumps({"zones_begin": {"zone_set_version": RIG.armed_version}}))
            RIG.log_wire("out", json.dumps({"arm": {"arm_id": RIG.arm_id, "origin": body.get("origin", "current")}}))
            RIG.log_wire("in", json.dumps({"armed": {"arm_id": RIG.arm_id}}))
            return self._send_json(RIG.armed())

        if path == "/api/zones/disarm":
            with RIG.lock:
                RIG.armed_name = None
                RIG.zone_state = {}
            RIG.log_wire("out", json.dumps({"disarm": {}}))
            return self._send_json(RIG.armed())

        if path == "/api/zones/save":
            with RIG.lock:
                if RIG.armed_name is None:
                    return self._refuse(409, "nothing_armed", "arm a set before writing it to flash")
                RIG.flashed_zone_set = {"name": RIG.armed_name, "version": RIG.armed_version}
            RIG.log_wire("out", json.dumps({"save": {}}))
            return self._send_json(RIG.device())

        return self._refuse(404, "no_such_route", f"the mock has no {path}")

    def do_PUT(self):
        path = urlparse(self.path).path
        body = self._body()

        if path == "/api/calibration":
            with RIG.lock:
                for axis in body.get("axes", []):
                    if "counts_per_cm" in axis:
                        RIG.counts_per_cm = float(axis["counts_per_cm"])
                    if "invert" in axis:
                        RIG.invert = bool(axis["invert"])
                return self._send_json(RIG.calibration())

        if path == "/api/calibration/ball":
            return self._refuse(501, "not_implemented", "ball calibration arrives with the hardware")

        if path.startswith("/api/zone-sets/"):
            name = path[len("/api/zone-sets/"):]
            problem = validate_zone_set(body)
            if problem is not None:
                return self._refuse(422, "zone_set_refused", problem, name)
            with RIG.lock:
                RIG.zone_sets[name] = body
                return self._send_json(body)

        return self._refuse(404, "no_such_route", f"the mock has no {path}")

    def do_PATCH(self):
        path = urlparse(self.path).path
        body = self._body()
        if path == "/api/config":
            with RIG.lock:
                RIG.rate_hz = int(body.get("rate_hz", RIG.rate_hz))
                RIG.display_hz = int(body.get("display_hz", RIG.display_hz))
                return self._send_json({"rate_hz": RIG.rate_hz, "display_hz": RIG.display_hz})
        return self._refuse(404, "no_such_route", f"the mock has no {path}")

    # ------------------------------------------------------- static files ---

    def _send_html(self, text):
        body = text.encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _serve_element(self, relative):
        target = (ELEMENTS_DIR / relative).resolve()
        if not str(target).startswith(str(ELEMENTS_DIR.resolve())) or not target.is_file():
            return self._refuse(404, "no_such_element", f"no element file {relative}")
        body = target.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "text/javascript; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        # The console is served from somewhere else entirely, and `no-cache`
        # because a browser holding yesterday's panel list against today's
        # daemon is exactly the failure the contract's headers prevent.
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Cache-Control", "no-cache, must-revalidate")
        self.end_headers()
        self.wfile.write(body)

    # ---------------------------------------------------------- websocket ---

    def _serve_websocket(self, path, query):
        key = self.headers.get("Sec-WebSocket-Key")
        if key is None:
            return self._refuse(400, "bad_upgrade", "no Sec-WebSocket-Key")
        accept = base64.b64encode(hashlib.sha1((key + WEBSOCKET_GUID).encode()).digest()).decode()
        self.send_response(101)
        self.send_header("Upgrade", "websocket")
        self.send_header("Connection", "Upgrade")
        self.send_header("Sec-WebSocket-Accept", accept)
        self.end_headers()

        queue: list[dict] = []
        gate = threading.Event()

        def enqueue(message):
            queue.append(message)
            gate.set()

        if path == "/api/stream":
            rate_hz = float(query.get("rate_hz", ["30"])[0])
            sink = decimate_to(enqueue, rate_hz)
            subscribers = RIG.subscribers
        elif path == "/api/device/monitor/stream":
            sink = enqueue
            subscribers = RIG.wire_subscribers
        else:
            return
        subscribers.append(sink)

        try:
            while True:
                gate.wait(timeout=0.2)
                gate.clear()
                while queue:
                    self._send_frame(json.dumps(queue.pop(0)))
                if self._client_went_away():
                    break
        except (BrokenPipeError, ConnectionResetError, OSError):
            pass
        finally:
            if sink in subscribers:
                subscribers.remove(sink)

    def _send_frame(self, text):
        payload = text.encode()
        header = bytearray([0x81])
        length = len(payload)
        if length < 126:
            header.append(length)
        elif length < 65536:
            header.append(126)
            header += struct.pack(">H", length)
        else:
            header.append(127)
            header += struct.pack(">Q", length)
        self.wfile.write(bytes(header) + payload)
        self.wfile.flush()

    def _client_went_away(self):
        readable, _, _ = select.select([self.connection], [], [], 0)
        if not readable:
            return False
        try:
            return self.connection.recv(1024) == b""
        except OSError:
            return True


def decimate_to(enqueue, rate_hz):
    """Wrap a subscriber so it only sees `rate_hz` samples a second.

    The daemon decimates for a browser rather than sending it 500 Hz; zone hits
    are never decimated, because dropping one is dropping the event.

    **Decimation is not loss, and the stream has to keep them apart.** A client
    cannot do it: `seq` skips either way. So what this drops carries no report,
    and what it *never received* is accumulated across the dropped samples onto
    the next one it does deliver, as `lost_before`. A consumer breaks its line
    on that and on nothing else.
    """
    state = {"last": 0.0, "pending_lost": 0}
    period = 1.0 / max(rate_hz, 1.0)

    def decimated(message):
        if message.get("type") == "sample":
            now = time.monotonic()
            if now - state["last"] < period:
                state["pending_lost"] += message.get("lost_before", 0)
                return
            state["last"] = now
            message = dict(message)
            message["lost_before"] = message.get("lost_before", 0) + state["pending_lost"]
            state["pending_lost"] = 0
        enqueue(message)

    return decimated


def substitute(zone_set, patch):
    """Resolve `"$name"` references from a per-trial patch, host-side."""
    text = json.dumps(zone_set)
    for name, value in patch.items():
        text = text.replace(f'"${name}"', json.dumps(value))
    return json.loads(text)


def validate_zone_set(zone_set):
    """The refusals the compiler owes an editor, in the words it would use."""
    if not isinstance(zone_set, dict):
        return "a zone set is an object"
    zones = zone_set.get("zones")
    if not isinstance(zones, list) or not zones:
        return "a zone set needs at least one zone"
    if len(zones) > 16:
        return f"{len(zones)} zones, and the board holds 16"
    for zone in zones:
        if zone.get("shape") != "rect":
            return f"{zone.get('name', '?')}: bad_shape {zone.get('shape')!r} — rect is the only shape today"
        if not zone.get("axes"):
            return f"{zone.get('name', '?')}: names no axis"
        for value in (zone.get("min_cm") or []) + (zone.get("max_cm") or []):
            if isinstance(value, str):
                return f"{zone.get('name', '?')}: unresolved {value} — give it in the arm patch"
        if zone.get("fire") not in ("once", "rearm"):
            return f"{zone.get('name', '?')}: fire is once or rearm"
    return None


def main():
    port = int(os.environ.get("MOUSEWHEELD_MOCK_PORT", "8082"))
    threading.Thread(target=wheel_thread, daemon=True).start()
    threading.Thread(target=wire_chatter_thread, daemon=True).start()
    server = ThreadingHTTPServer(("127.0.0.1", port), MockHandler)
    print(f"mock mousewheeld on http://127.0.0.1:{port}  (elements from {ELEMENTS_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
