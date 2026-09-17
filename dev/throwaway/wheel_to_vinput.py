#!/usr/bin/env python3
"""The preliminary ESP32 sketch → a vinput segment vstimd reads every frame.

**This is a throwaway, and it is written to be deleted at M2.** It exists
because vstimd's whole consumer half is built and has never been driven by real
hardware: the `vinput` crate, `InputRegistry`, the staleness policy,
`LinearNav3D` and `DeviceDrivenTransform` all work against tests and against
nothing else. A day of this answers the questions no test can — does the
corridor actually move smoothly, what is encoder-to-photon on this bench, does
pulling the USB plug fail the way the design says — and it yields M3's latency
number long before M3.

Nothing here is a design. It polls, because `rotary-encoder/src/main.cpp` only
prints a line when a byte arrives; it unwraps a 16-bit counter host-side,
because that sketch has no 64-bit accumulator; it does its own calibration
arithmetic, because there is no daemon yet to own it. Every one of those is a
thing the real firmware and daemon exist to stop doing, and the numbers this
prints are the argument for building them.

What it *does* honour, because these are the contract and not the prototype:

- **it publishes centimetres**, so the rig-config axis is `scale = 1.0`
  (`dev/PLAN.md`, *Calibration*, and vstimd's `INPUT_LATENCY.md` §6);
- **the axis is cumulative and never reset** — a running total, so a frame that
  misses a sample folds it into the next one instead of losing it;
- **it writes every pass, moving or not**, because a write is the heartbeat and
  a silent producer is a stopped stimulus after `stale_after_ms`.

Usage
-----
    # measure the calibration first -- see below
    uv run --with pyserial python wheel_to_vinput.py --port /dev/ttyUSB0 \\
        --measure 100

    # a real board
    uv run --with pyserial --with vstimd-client \\
        python wheel_to_vinput.py --port /dev/ttyUSB0 --counts-per-cm 86.92

    # no hardware: a synthetic wheel, to exercise the whole path
    uv run --with vstimd-client python wheel_to_vinput.py --simulate

Calibrating
-----------
`--measure <cm>` counts between two presses of Enter while you roll the wheel a
known distance, and prints the `counts_per_cm` that follows. **Measure it; do
not compute it from the wheel's nominal diameter.** What the corridor needs is
how far the animal's feet travelled, and that is the circumference of the
surface it runs on -- inside the rim, with a tread on it, with the feet a few
millimetres off the drum. `counts_per_rev / (pi * diameter_cm)` is the number to
sanity-check the measurement against, not the number to use.

Roll the wheel by hand along its running surface against a tape for a metre or
so, in the direction the animal runs, in one continuous motion. A metre averages
away where you started and stopped; a single revolution does not. `--measure`
also reports the implied circumference and diameter when `--counts-per-rev` is
given, which is what catches a decoder counting x1 or x2 instead of x4 -- an
error that looks exactly like a wheel four times too small.

vstimd's side, in `rig-config.toml`:

    [[input.device]]
    name           = "treadmill"
    shm            = "/vstimd_wheel"
    stale_after_ms = 100

      [[input.device.axis]]
      name     = "distance"
      semantic = "cumulative"
      scale    = 1.0
"""

from __future__ import annotations

import argparse
import math
import select
import statistics
import sys
import time
from dataclasses import dataclass, field

from vstimd.shm import InputDevice, Semantic

COUNTER_BITS = 16
COUNTER_SPAN = 1 << COUNTER_BITS


def unwrap(previous: int, current: int) -> int:
    """Signed change between two readings of a wrapping counter.

    The sketch publishes `uint16_t` with `circleValues`, so 65530 → 6 is +12 and
    not −65524. Correct for any movement of less than half a turn of the
    counter between two polls — half a span is 32768 counts, which at 4096
    counts per revolution is eight revolutions in one poll. A wheel that fast
    has other problems.
    """
    return (current - previous + COUNTER_SPAN // 2) % COUNTER_SPAN - COUNTER_SPAN // 2


@dataclass
class PollStats:
    """What this prototype is actually for: the numbers, not the motion."""

    round_trip_ms: list[float] = field(default_factory=list)
    polls: int = 0
    parse_failures: int = 0
    started: float = field(default_factory=time.perf_counter)

    def record(self, round_trip_s: float) -> None:
        self.polls += 1
        self.round_trip_ms.append(round_trip_s * 1e3)
        # Bounded: this runs for hours and the summary only wants a distribution.
        if len(self.round_trip_ms) > 20_000:
            del self.round_trip_ms[:10_000]

    def summary(self) -> str:
        if not self.round_trip_ms:
            return "no polls completed"
        s = sorted(self.round_trip_ms)
        elapsed = time.perf_counter() - self.started
        p = lambda q: s[min(len(s) - 1, int(q * len(s)))]  # noqa: E731
        return (
            f"{self.polls} polls in {elapsed:.1f}s ({self.polls / max(elapsed, 1e-9):.0f} Hz), "
            f"round trip median {statistics.median(s):.2f} ms, "
            f"p95 {p(0.95):.2f} ms, max {max(s):.2f} ms, "
            f"{self.parse_failures} unparseable lines"
        )


def parse_sample(line: str) -> int | None:
    """`DATA:4:<t_ms>,<dt_ms>,<position>,<step>` → the position, or None.

    Only the position is used. The sketch's `step` is a 16-bit difference that
    is wrong across a wrap, and its `t_ms` is a device clock this prototype does
    not try to correlate — the real daemon does (`dev/PLAN.md`, *The clock*).
    """
    if not line.startswith("DATA:"):
        return None
    fields = line.split(":", 2)
    if len(fields) != 3:
        return None
    parts = fields[2].split(",")
    if len(parts) < 3:
        return None
    try:
        return int(parts[2])
    except ValueError:
        return None


class SerialWheel:
    """The board: poke it with a byte, read back one line."""

    def __init__(self, port: str, baud: int) -> None:
        try:
            import serial  # noqa: PLC0415 — optional, only the real-hardware path
        except ImportError as e:
            raise SystemExit("--port needs pyserial: uv run --with pyserial …") from e
        # A timeout well under one poll period, so a board that stopped talking
        # shows up as a stalled rate rather than as a hung process.
        self._port = serial.Serial(port, baud, timeout=0.05)
        self._port.reset_input_buffer()

    def poll(self) -> int | None:
        self._port.write(b"\n")
        line = self._port.readline().decode("ascii", "replace").strip()
        return parse_sample(line) if line else None

    def close(self) -> None:
        self._port.close()


class SimulatedWheel:
    """A wheel with no board behind it, wrapping exactly as the sketch does."""

    def __init__(self, counts_per_cm: float, speed_cm_s: float) -> None:
        self._counts_per_s = counts_per_cm * speed_cm_s
        self._started = time.perf_counter()

    def poll(self) -> int:
        travelled = (time.perf_counter() - self._started) * self._counts_per_s
        # A slow sinusoidal component, so the reader sees direction changes and
        # a stationary phase rather than a ramp that hides both.
        wobble = 200.0 * math.sin((time.perf_counter() - self._started) * 0.7)
        return int(travelled + wobble) % COUNTER_SPAN

    def close(self) -> None:
        pass


def enter_pressed() -> bool:
    """True if a line is waiting on stdin — so the poll loop never blocks on it."""
    if not select.select([sys.stdin], [], [], 0)[0]:
        return False
    sys.stdin.readline()
    return True


def measure(wheel: SerialWheel | SimulatedWheel, args: argparse.Namespace) -> int:
    """Count between two presses of Enter while the wheel is rolled a known distance.

    The guided procedure `mousewheeld/dev/PLAN.md` describes as
    `calibration/measure/{start,finish,apply}`, with the person at the terminal
    standing in for the API and nothing persisted: this prototype has no store
    to apply a calibration to.
    """
    known_cm = args.measure
    period = 1.0 / args.rate_hz
    print(
        f"\nRoll the wheel {known_cm:g} cm along its running surface, in the "
        f"direction the animal runs.\nPress Enter to start, then Enter again "
        f"when you get there (Ctrl-C to abort).",
        file=sys.stderr,
    )
    while not enter_pressed():
        time.sleep(0.05)

    total = 0
    previous: int | None = None
    started = time.perf_counter()
    print("counting…", file=sys.stderr)
    while True:
        time.sleep(period)
        position = wheel.poll()
        if position is not None:
            if previous is not None:
                total += unwrap(previous, position)
            previous = position
        if enter_pressed():
            break
        if time.perf_counter() - started > 0.25:
            started = time.perf_counter()
            print(f"\r  {total:+8d} counts", end="", file=sys.stderr)

    counts = abs(total)
    print(f"\r  {counts} counts over {known_cm:g} cm", file=sys.stderr)
    if counts < 100:
        print(
            "  that is too few counts to calibrate anything — is the encoder "
            "wired, and did the wheel actually turn?",
            file=sys.stderr,
        )
        return 1

    per_cm = counts / known_cm
    print(f"\n  counts_per_cm = {per_cm:.3f}", file=sys.stderr)
    if args.counts_per_rev:
        circumference = args.counts_per_rev / per_cm
        print(
            f"  implied circumference {circumference:.2f} cm "
            f"(diameter {circumference / math.pi:.2f} cm) at "
            f"{args.counts_per_rev} counts/rev",
            file=sys.stderr,
        )
        if args.diameter_cm:
            nominal = args.counts_per_rev / (math.pi * args.diameter_cm)
            print(
                f"  nominal from --diameter-cm {args.diameter_cm:g}: "
                f"{nominal:.3f} counts/cm "
                f"({100 * (per_cm - nominal) / nominal:+.1f} % measured)",
                file=sys.stderr,
            )
            # A whole-number ratio is a decoding mistake, not a wheel: x1 or x2
            # quadrature against the x4 the counts-per-rev assumes.
            ratio = per_cm / nominal
            for factor in (2.0, 4.0, 0.5, 0.25):
                if abs(ratio - factor) < 0.05:
                    print(
                        f"  that is {factor:g}× the nominal — check the "
                        f"quadrature decoding, not the wheel",
                        file=sys.stderr,
                    )
    print(f"\nRun with --counts-per-cm {per_cm:.3f}\n", file=sys.stderr)
    return 0


def counts_per_cm(args: argparse.Namespace) -> float:
    if args.counts_per_cm is not None:
        return args.counts_per_cm
    if args.counts_per_rev is not None and args.diameter_cm is not None:
        return args.counts_per_rev / (math.pi * args.diameter_cm)
    raise SystemExit(
        "give --counts-per-cm, or --counts-per-rev together with --diameter-cm "
        "(the wheel's full diameter, not a radius)"
    )


def run(args: argparse.Namespace) -> int:
    # Measuring needs counts and nothing else, so it must not demand the
    # calibration it exists to produce.
    per_cm = 86.9 if args.measure else counts_per_cm(args)
    wheel = (
        SimulatedWheel(per_cm, args.simulate_speed_cm_s)
        if args.simulate
        else SerialWheel(args.port, args.baud)
    )
    if args.measure:
        try:
            return measure(wheel, args)
        except KeyboardInterrupt:
            return 1
        finally:
            wheel.close()
    period = 1.0 / args.rate_hz
    stats = PollStats()
    total_counts = 0
    previous: int | None = None
    next_poll = time.perf_counter()

    print(
        f"publishing {args.shm} axis '{args.axis}' in cm "
        f"({per_cm:.4f} counts/cm, target {args.rate_hz} Hz, "
        f"{'simulated' if args.simulate else args.port})",
        file=sys.stderr,
    )
    device = InputDevice.create(args.shm, [(args.axis, Semantic.CUMULATIVE, 1.0)])
    last_report = time.perf_counter()
    try:
        while True:
            now = time.perf_counter()
            if now < next_poll:
                time.sleep(next_poll - now)
            next_poll += period
            if next_poll < time.perf_counter():  # fell behind: do not spiral
                next_poll = time.perf_counter() + period

            sent = time.perf_counter()
            position = wheel.poll()
            if position is None:
                stats.parse_failures += 1
            else:
                stats.record(time.perf_counter() - sent)
                if previous is not None:
                    total_counts += unwrap(previous, position)
                previous = position

            # Every pass, moving or not: the write is the heartbeat.
            device.write([total_counts / per_cm])

            if args.report_s and time.perf_counter() - last_report >= args.report_s:
                last_report = time.perf_counter()
                print(
                    f"{total_counts / per_cm:10.2f} cm   {stats.summary()}",
                    file=sys.stderr,
                )
    except KeyboardInterrupt:
        return 0
    finally:
        print(f"\n{stats.summary()}", file=sys.stderr)
        print(f"published {total_counts / per_cm:.2f} cm total", file=sys.stderr)
        device.close()
        wheel.close()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__.split("\n")[0],
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("--port", help="serial port of the ESP32, e.g. /dev/ttyUSB0")
    ap.add_argument("--baud", type=int, default=250_000, help="must match the sketch (250000)")
    ap.add_argument("--shm", default="/vstimd_wheel", help="segment name vstimd's rig config names")
    ap.add_argument("--axis", default="distance", help="axis name, matching the rig config")
    ap.add_argument(
        "--rate-hz",
        type=float,
        default=500.0,
        help="poll rate. Keep it ABOVE the display rate: at or below it, some "
        "frames see no new sample and the next sees two, which is visible "
        "stutter at exactly a running animal's speeds",
    )
    ap.add_argument("--counts-per-cm", type=float, help="calibration, measured")
    ap.add_argument("--counts-per-rev", type=int, help="encoder counts per revolution, after ×4")
    ap.add_argument("--diameter-cm", type=float, help="wheel diameter (a full extent, not a radius)")
    ap.add_argument(
        "--measure",
        type=float,
        metavar="CM",
        help="calibrate: count while the wheel is rolled this many centimetres, "
        "then print the counts_per_cm that follows. Measure the surface the "
        "animal runs on, not the nominal diameter",
    )
    ap.add_argument("--simulate", action="store_true", help="synthesise a wheel; no board needed")
    ap.add_argument("--simulate-speed-cm-s", type=float, default=20.0)
    ap.add_argument("--report-s", type=float, default=5.0, help="progress interval; 0 to silence")
    args = ap.parse_args(argv)

    if not args.simulate and not args.port:
        ap.error("give --port, or --simulate")
    if args.measure is not None and args.measure <= 0:
        ap.error("--measure takes the distance you will roll, in cm")
    if args.simulate and args.counts_per_cm is None and args.counts_per_rev is None:
        args.counts_per_cm = 86.9  # a 4096-count encoder on a 15 cm wheel
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
