# mousewheeld — the plan

> **Status:** plan. Nothing here is built. The preliminary ESP32 sketch
> in `rotary-encoder/` (imported with its history from `joschaschmiedt/mouse_wheel`)
> is kept for reference and not built on: it is a hand-knob encoder
> library with a 16-bit position, polled on any received byte, and none of it
> survives the design below. Milestones and their state are at the end.

## What mousewheeld is

The **locomotion input** of a braemons rig: a rotary encoder on a running wheel
(1-D), or — later, and possibly never — sensors on an air-supported ball (2-D),
read by a microcontroller and published to whoever needs to know how far the
animal went.

It has two consumers, and they want opposite things:

- **vstimd** moves the camera through a corridor. It needs the position *every
  frame*, within a frame, and cannot afford a request/response round trip to
  get it (`vstimd/dev/INPUT_LATENCY.md` §2).
- **triald** records what happened in a trial. It needs the path *once per
  trial*, exactly, and does not care whether it arrives a second late.

And one job neither consumer can do: **trigger zones** — "the animal has run
200 cm" — decided on the device, in the scan that sees the count, and put on a
TTL line where statemachined and daqd can act on it.

```
                 ┌──────────────────────────────────────────────┐
                 │ triald   decides · records                   │
                 └───┬──────────────────────┬───────────────────┘
      (D) command    │ mark · arm zone set  │ (E) path since mark, zone hits
      HTTP+JSON      ▼                      │     HTTP  ·  or ZMQ SUB
              ┌──────────────────┐◀─────────┘
   console ──▶│   mousewheeld    │──▶ ZMQ PUB  samples · zone hits  (anyone, any host)
   /elements  │ control │ data   │──▶ recording (full path, per session)
              └──┬───────────┬───┘
   USB · NDJSON  │           │ (F) vinput shm  /vstimd_wheel      same host
   · CRC         │           ▼
              ┌──┴─────────┐ ┌──────────┐
              │ firmware   │ │  vstimd  │  camera: LinearNav3D / DeviceDrivenTransform
              │ counts     │ └────▲─────┘
              │ zones      │      │ VTL
              │ analog out │ ┌────┴─────┐
              └──┬─────────┘ │   daqd   │
                 │ TTL ─────▶└──────────┘
                 └─────────▶ statemachined input line   ("goal_reached" → HIT)
```

### The name

**`mousewheeld`** — the family's `<function>d`. It names the rig's usual
hardware rather than an abstraction ("locomotiond") because nobody searches for
the abstraction, and the ball is a second device of the same daemon, not a
reason to rename it.

| | |
|---|---|
| `mousewheeld` | the project, the wire protocol, the daemon, the event schema |
| `mousewheeld-firmware` | the portable core plus HALs; packaged per board — `mousewheeld-firmware-teensy41`, `mousewheeld-firmware-esp32` |

---

## The rule this follows

`contracts/INTERACTIONS.md` §2: **a participant publishes what it observed and
commands nobody; a decision authority commands its participants and subscribes
to what they publish.**

mousewheeld is a participant. It has no client of any other daemon, no
`triald_base_url`, no `vstimd_address`. So:

- **vstimd does not call mousewheeld, and mousewheeld does not call vstimd.**
  mousewheeld writes a shared-memory segment whose layout vstimd defines; vstimd
  reads a device named in its rig config and never learns who writes it. This is
  exactly how `gpiochip-daqd` and VTL already work.
- **triald commands mousewheeld** (place a mark, arm a zone set) and reads what
  it publishes (the path since the mark). mousewheeld stays **trial-blind**: a
  mark's label is opaque text it stores and hands back.
- **Zones never go through triald.** An outcome that depends on distance is a
  TTL edge into statemachined, on the fast bus, the same way stimulus onset is.
  triald records the hit after the fact.

---

## The model

### Axes and counts

A device has **N axes**, `MOUSEWHEELD_MAX_AXES` per board (2 on both targets).
Each axis is:

| | |
|---|---|
| `source` | `quadrature` (implemented) · `optical_spi` (declared, refused as `unsupported_source`) |
| `invert` | count direction |
| `counts` | a signed **64-bit** accumulator on the device, extended from the hardware counter |

The firmware knows **counts and nothing else**. No centimetres, no floating
point on the wire. Calibration is a host concern (below), and the host compiles
every centimetre a user writes into counts before it reaches the board.

Two derived quantities, both per axis:

| | |
|---|---|
| **displacement** | signed, `counts − origin`. What a corridor position is |
| **distance** | direction-free odometer, `Σ |Δcounts|`. What "how much did it run" is |

Both exist on the device, because a zone may be defined on either.

### The 1-D wheel is the product; the 2-D ball is a skeleton

Only the 1-D wheel is built. The 2-D ball exists in the model so that adding it
later is filling in, not redesigning:

- the wire, the API and the vinput descriptor are **N-axis from day one**;
- zones take a per-axis interval list, so a 2-D rectangle is already expressible;
- ball calibration — sensor mounting angle, ball diameter, the transform from two
  sensor axes to `x`, `y`, `yaw` — exists as models and routes that answer
  **`501 Not Implemented`**;
- `optical_spi` is a valid enum value the firmware refuses by name.

The native build and the host tests run with two quadrature axes, so the N-axis
paths are exercised even though no rig has a second axis.

### Calibration

Per axis, host side, persisted in the rig config:

```toml
[[axis]]
name           = "wheel"
counts_per_rev = 4096          # after quadrature ×4
diameter_cm    = 15.0          # → counts_per_cm derived, or given directly:
# counts_per_cm = 86.9
invert         = false
```

**mousewheeld owns calibration, not vstimd.** `INPUT_LATENCY.md` §6 currently
puts `scale` in vstimd's `[[input.device.axis]]`. That makes two copies of a
hardware fact, one of which triald cannot see — and triald needs centimetres
too. So mousewheeld publishes **centimetres** into the shm segment and writes
`scale = 1.0` into the axis descriptor; vstimd's rig config names the device and
does not restate its calibration. That section of vstimd's document changes with
M3.

A **measurement procedure** replaces arithmetic nobody trusts:
`calibration/measure/start {axis, known_distance_cm}` → the experimenter turns
the wheel a marked distance → `finish` returns the measured `counts_per_cm` next
to the configured one → `apply` persists it. Every record carries the calibration
in force, and a change is an event, never an in-place edit.

### Trigger zones

**Zones are dynamic; the lines they drive are not.** The split is
statemachined's, between its graphs and its line map:

| | Where it lives | Changes |
|---|---|---|
| **Output lines** — name → pin, safe level | the **rig config**, uploaded at connect (`lines`) | when the wiring does |
| **Zone sets** — geometry, metric, fire rule, which named line | the daemon's **zone-set store**, uploaded on arm | per session, per trial type, per trial |

A zone set says `"line": "zone_goal"` and never a pin number, so one zone set
runs on every rig that has a `zone_goal` line, and a rewired rig edits one file.

A zone set is **named, stored by the daemon, compiled to counts and uploaded**.
Arming a set already on the board is one small message; arming a different one
uploads it first, which for a handful of zones is a few lines. Like
statemachined's graphs, the set on the board can **optionally be saved to flash**
and restored at boot, so a standalone board comes up with its zones (see
*Standalone*) — but the store on the host is the source, and a flashed set is a
copy that the board reports by name and version in `hello_ack`.

**Per-trial values without editing the store.** A numeric field may be a
`"$name"` reference, resolved from the `patch` given at arm time — the same
mechanism as statemachined's per-trial patches. A goal distance drawn per trial
is `"min_cm": ["$goal_cm"]` plus `arm {zone_set: "goal", patch: {"goal_cm": 180}}`.
Substitution happens host-side before compilation, and the board re-receives the
set only if the compiled counts actually changed.

As authored:

```jsonc
{
  "zone_set_version": 3,
  "zones": [
    { "name": "goal",
      "shape": "rect",                 // the only shape today
      "axes": ["wheel"],
      "metric": "displacement",        // | "distance"
      "min_cm": [200.0], "max_cm": [null],   // null = open bound
      "fire": "once",                  // | "rearm", with hysteresis_cm
      "output": {"line": "zone_goal", "action": "pulse", "ms": 10} },

    { "name": "reward_region",
      "shape": "rect", "axes": ["wheel"], "metric": "displacement",
      "min_cm": [120.0], "max_cm": [140.0],
      "wrap_cm": 300.0,                // circular track: evaluated modulo the period
      "fire": "rearm", "hysteresis_cm": 2.0,
      "output": {"line": "zone_region", "action": "level"} }   // high while inside
  ]
}
```

Semantics, and they are wire-visible so they are specified:

- A zone **fires on entry** — the false→true edge of "inside", evaluated in the
  scan that sees the count change. Not on the level: arming a zone the animal is
  already inside does not fire it, unless `"level": true`.
- `once` disarms the zone after it fires. `rearm` re-arms it after the position
  has left the zone by `hysteresis_cm`, so encoder jitter at a boundary is not a
  pulse train.
- `level` outputs follow "inside" (with the same hysteresis); `pulse` is high for
  at most `ms`, as in statemachined.
- **Arming sets the origin.** `arm {origin: "current"}` makes displacement and
  distance zero at the arm point, which is what a trial wants; `"absolute"` keeps
  the device origin.
- Declaration order resolves two zones firing in one scan; both still fire, and
  the order is the order of their `zone_hit` messages.
- **`shape` is a discriminator, not a flag.** A future `circle` or `polygon` is a
  new value; the firmware refuses a shape it does not know as `bad_shape`, so it
  never evaluates half a zone.

On the device a `rect` is one integer interval per axis. Cost per scan is bounded
by armed zones, and zero when the counts did not move (see *Performance rules*).

### Analog output

The firmware can drive a voltage proportional to position, **with no daemon
running**. It is the debugging tool that needs nothing but a scope, and it is how
a rig without the daemon — or a legacy acquisition system — sees the wheel.

| mode | output |
|---|---|
| `wrap` | displacement modulo `range_counts`, a sawtooth |
| `clamp` | displacement clamped to `[lo, hi]` |
| `velocity` | device velocity (see *Velocity*), centred, clamped |

| Board | How |
|---|---|
| ESP32 (classic WROOM-32) | built-in 8-bit DAC, GPIO25/26. **ESP32-S3/C3 have no DAC** |
| Teensy 4.1 | no DAC. Filtered PWM by default; an SPI DAC (MCP4822, 12-bit) as a build option |

Updated from the foreground at a fixed rate, never from the counter interrupt.

### Velocity

Computed in **both** places, for different jobs, and never confused:

| | Device | Host |
|---|---|---|
| From | counts over a fixed window in the scan, at scan resolution | the sample path, with host-configurable filtering |
| For | the analog `velocity` mode; `state_report`; optionally `v` on each `sample` | the API, the UI, zone-free analysis, path summaries |
| Unit on the wire | counts/s, integer | `velocity_cm_s` |
| Recorded | yes, as sent (`device_velocity_cm_s`) | yes (`velocity_cm_s`) |

Both are recorded because they answer different questions — what the board acted
on, and what the animal did — and a record that silently kept only one could not
be checked against the other. The API names which one it returns.

### Standalone

A board must be useful with nothing attached but power:

- axis config, the line map, analog output and — optionally — the zone set and
  its armed state are **saved to flash** (`save`) and restored at boot, as
  statemachined's autorun;
- a plain-text **`debug` mode** prints human-readable lines for a serial
  terminal. Entered only by an explicit command and left on `hello`, so it can
  never interleave with the protocol.

---

## The wire protocol

USB CDC (Teensy) or UART bridge (ESP32), **newline-delimited JSON, a
`message_id` and a CRC-16/CCITT-FALSE per line**. Same framing rules as
statemachined's `docs/reference/protocol.md` §1, written again here rather than
shared (see *Relationship to statemachined*). Full spec lands in
`docs/reference/protocol.md`.

### Host → device

| Message | Payload | Notes |
|---|---|---|
| `hello` | protocol version | Reply `hello_ack`: board, firmware, n_axes, max_zones, max_line, stream rates supported |
| `axes` | `[{source, invert}]` | Refused per axis by name (`unsupported_source`) |
| `lines` | `[{index, pin, safe}]` | From the rig config. Zones refer to lines by index; names stay host-side |
| `stream` | `{rate_hz, velocity}` | 0 = off; default **200 Hz**. Samples are cumulative, so any rate loses nothing but resolution |
| `zones_begin` / `zone` / `zones_end` | `{zone_set_version, n}` / one zone / `{checksum}` | Chunked; the live set is untouched until `zones_end` commits |
| `arm` | `{arm_id, zone_set_version, origin}` | Reply `armed` with both, as statemachined's. Patches are already resolved; the board never sees a `$name` |
| `disarm` | `{arm_id}` | |
| `zero` | `{axes?}` | Moves the device origin. Never the accumulator (see *Continuity*) |
| `analog` | `{axis, mode, range…}` | |
| `save`, `ping`, `state`, `debug` | | |

### Device → host

| Message | Payload |
|---|---|
| `sample` | `{seq, t_us, c:[counts…], v?:[counts/s…]}` — **fixed shape, formatted by a dedicated writer** |
| `zone_hit` | `{seq, arm_id, zone, t_us, c:[counts…]}` |
| `armed` | `{arm_id, zone_set_version}` |
| `state_report` | counts, origin, armed zones, analog config, scan health, ring drops |
| `error`, `log`, `pong` | |

`sample` is the only high-rate message and the protocol is shaped around it:

- **Cumulative counts, never deltas.** A lost line costs resolution, never
  distance — the same argument `INPUT_LATENCY.md` §4.1 makes for the shm segment.
- **`seq`** on samples and zone hits, so the host sees a gap and says so.
- **`t_us`** is the device clock; host correlation is the daemon's job.
- **200 Hz by default; 100 Hz is enough for the record.** A line is ~50 bytes,
  so 200 Hz is 10 kB/s — nothing on either link. M1 still measures link cost per
  line on both boards, because statemachined found ~3 ms of USB stack per
  *command* on the R4 and the budget should be a number, not a hope.
- **The camera is the one consumer that wants more.** vstimd differences the shm
  value once per frame. With samples at 200 Hz and a display at 240 Hz, some
  frames see no new sample and the next sees two: the corridor moves in uneven
  steps, visibly, at exactly the speeds a running animal produces. So the stream
  rate must be **at least the display rate** on a rig whose camera follows the
  wheel — 250 Hz for 120 Hz, 500 Hz for 240 Hz, still only 25 kB/s. The daemon
  warns when `rate_hz` is below the `display_hz` given in its config. The record
  is unaffected by the higher rate; decimating it is an analysis choice.

---

## Firmware

### The portable core

Plain C++17, no `Arduino.h`, no dynamic allocation, no floating point in any path
whose result crosses the wire. Hardware behind a HAL:

```c++
namespace mousewheeld::hal {
  void         init();                            // outputs safe before anything else
  Microseconds micros_now();
  int32_t      read_counter(uint8_t axis);        // raw hardware counter, wraps
  void         write_outputs(LineMask high, LineMask low);
  void         write_analog(uint8_t channel, uint16_t value);
  size_t       serial_read(uint8_t*, size_t);
  size_t       serial_write(const uint8_t*, size_t);
  bool         flash_load(Blob*); bool flash_save(const Blob&);
}
```

```
firmware/
├── core/
│   ├── config.h               capacities, unit aliases
│   ├── position/              64-bit extension of a wrapping counter · origin · odometer
│   ├── zones/                 zone table, rect evaluation, fire/rearm/hysteresis
│   ├── output/                TTL pulse/level scheduling · analog modes
│   ├── stream/                sample and zone-hit ring, fixed-format writers
│   ├── protocol/              framing · crc16 · json reader · session
│   └── hal.h
├── hal/                       teensy41.cpp · esp32.cpp · native.cpp
└── src/main.cpp               thin
```

### Boards

| | Teensy 4.1 — **reference** | ESP32 (classic WROOM-32) |
|---|---|---|
| Counting | i.MX RT1062 hardware quadrature decoder (ENC1–4, via XBAR; fixed pin set) | PCNT: 16-bit hardware counter per unit, glitch filter, watch-point interrupt at ±limit extends it |
| Axes | up to 4 decoders; 2 configured | 2 PCNT units |
| Link | native USB, 480 Mbit | USB-UART bridge (CP210x/CH340): baud matters, adds ~1–2 ms and jitter |
| Analog | PWM + RC, or SPI DAC | 8-bit DAC |
| Clock | `micros()`, cycle counter available | `esp_timer_get_time()` |

Teensy is the reference because the link is the latency that reaches the camera,
and native USB is the better link. The ESP32 is supported because it is what
labs have and because it has the DAC.

**No ISR-per-edge counting on either board.** A 4096-count wheel at 3 rev/s is
12 kHz of edges; hardware counters make edge rate irrelevant to the scan.

### The scan

A timer interrupt at `MOUSEWHEELD_SCAN_HZ` (start at 5 kHz, measured and raised if
it earns it):

1. read each hardware counter, extend to 64-bit;
2. **if no axis moved: return** — one compare per axis;
3. update displacement and distance, evaluate armed zones, schedule outputs;
4. push `{t_us, counts}` to the sample ring at the stream divider, and any
   `zone_hit` to the event ring.

`loop()` services the link, drains the rings into formatted lines, updates the
analog output, and ends pulses.

### Performance rules

statemachined paid for each of these with a measurement
(`statemachined/docs/operations/hardware.md`). They are requirements here from
the first commit, not optimisations for later.

1. **Nothing that formats a line runs in the interrupt.** statemachined's
   response latency was 223 µs instead of one scan period because a ~130-byte
   line with a CRC was built in the ISR; moving it out made it 100 µs, sd 0.
   Worse, `send_line()` spins on a full transmit queue, which from an interrupt
   is a deadlock the host tests cannot see. The ISR copies fixed-size structs
   into a ring. Only `loop()` formats.
2. **Single-producer, single-consumer rings; a full ring drops the newest entry
   and counts it.** That is what keeps them lock-free with an interrupt at one
   end. Drops are reported in `state_report` and visible in `seq`. **Zone hits
   have their own ring** and the output they drive does not depend on it: the
   TTL is scheduled in the scan, the message is only the record.
3. **Pace the drain.** A bounded number of lines per `loop()` pass, interleaved
   with the transmit drain, so a backlog never overruns the USB queue.
4. **Early-out when nothing changed.** The common scan — an animal sitting still
   — is a compare per axis and nothing else.
5. **Register-level HAL reads.** The counter read is the hot path; no vendor
   convenience call that costs microseconds.
6. **An own JSON reader**: no allocation, depth and member count bounded,
   structure validated once and indexed, a malformed line refused whole. Not
   ArduinoJson (7 allocates, 6 is unmaintained) — the reason statemachined gave.
7. **A fixed-format writer for `sample` and `zone_hit`.** No general serializer
   on the only high-rate path: integers into a preallocated buffer, CRC
   accumulated as it writes.
8. **Capacities are compile-time and reported in `hello_ack`.** A zone set that
   does not fit is refused at upload naming what overflowed.
9. **Measurements become tests.** Scan rate, overruns per command, link cost per
   line and the `seq` gap rate at each stream rate are asserted by
   `make test-hardware`, so a regression fails on its first assertion instead of
   quietly making a table in this document untrue.

Unit tests build the core under CMake on the host (doctest), never through
PlatformIO, exactly as statemachined does.

---

## The daemon

**One Rust binary.** The hard part of the daemon is the per-sample path, and that
was always going to be native; what is left — an HTTP API, a zone compiler,
calibration arithmetic, config, mDNS, serving static files — needs nothing
Python offers. A Python shell around a Rust core would buy a two-language build
(maturin *and* nfpm), a foreign-function boundary through the middle of the
daemon, and a restart of the HTTP side that freezes the camera. One binary has
none of those.

It is also vstimd's stack already: `axum` (with `ws`), `rust-embed`, `zeromq`,
`prost`, `serde`, `tokio`. A braemons developer who has read vstimd's server can
read this one, and the packaging is vstimd's.

### Two sides of one process

```
┌──────────────────────────── mousewheeld ───────────────────────────────────┐
│  CONTROL · tokio runtime                                                   │
│  axum: HTTP API · WS (decimated) · /elements/ (rust-embed)                 │
│  zone-set store and compiler (cm → counts) · calibration · config · marks  │
│  device session (hello, uploads, reconnection) · mDNS                      │
│         ▲ non-sample messages, decimated state   │ commands (lines to send) │
│         │ bounded lock-free queues, both ways    ▼                         │
│  ───────┼──────────────────────────────────────────────────────────────────│
│  REAL-TIME · one dedicated OS thread, elevated priority, no allocation      │
│  serial fd → frame/CRC → sample fast-parse → clock map → continuity        │
│     ├─▶ vinput shm       seqlock write + heartbeat         first, always   │
│     ├─▶ ZMQ PUB          protobuf; drops for slow subscribers              │
│     ├─▶ path ring        30 min, queried by mark or time window            │
│     └─▶ recording        lossless, per session (hands buffers to a writer) │
└────────────────────────────────────────────────────────────────────────────┘
```

**Everything that happens per sample stays on the real-time thread**, and the
thread never waits on the control side: it does not take a lock the HTTP handlers
take, does not `.await`, and does not touch the file system directly — full
recording buffers go to a writer thread through a queue, the same discipline as
vstimd's render thread and its messenger. The control side sees only the messages
a person or a consumer acts on — `armed`, `zone_hit`, `error`, `state_report` —
plus a decimated state for the UI. It reaches the path ring through a read-only
snapshot, never by blocking the writer.

**The serial port has one owner**, the real-time thread. The control side sends a
command by queueing a line; replies come back by `message_id`. Retry, CRC and
reconnection are the device session's, on the control side, because none of it is
per-sample.

**`SCHED_FIFO` for the real-time thread** where the capability is granted
(`CAP_SYS_NICE`, set by packaging), a logged warning where it is not — the same
arrangement `INPUT_LATENCY.md` §13 plans for vstimd's render thread.

### Continuity

vstimd **differences** a cumulative axis every frame. A value that jumps
backwards jumps the camera backwards. So the number in shm is continuous,
whatever happens below it:

- a firmware reset, a reflash or a USB reconnect restarts device counts at zero;
  the real-time thread holds an offset so the published value continues;
- `zero`, marks and `arm {origin: "current"}` move **origins** — for the API,
  zones and records — never the published accumulator;
- **teleporting the corridor back to its start is not mousewheeld's.** It is a
  command to vstimd, from whoever runs the experiment.

The published value is `f64` centimetres, per `INPUT_LATENCY.md` §4.3: a
kilometre at centimetre resolution exhausts `f32`.

### The clock

Every sample carries device `t_us`; the real-time thread maps it to host
`CLOCK_MONOTONIC` nanoseconds, the correlation statemachined does in
`device/device_clock_correlation.py`, reimplemented. **Host monotonic time is the
join key** with statemachined's trace and vstimd's vblank timestamps
(`INPUT_LATENCY.md` §11.3 names `CLOCK_MONOTONIC` as vstimd's domain). Without
it, "how far did it run during trial 42" cannot be asked.

### The wire shape is the file shape

statemachined gets this from pydantic; here it is `serde`. The zone set, the
calibration and the config are one set of Rust types that are at once the HTTP
body, the file in the store and the rig-config section — no DTOs, which is
vstimd's rule (*the config format is the runtime shape*). `utoipa` derives the
OpenAPI document from the same types and the daemon serves it at
`/api/openapi.json`, so the clients below are checked against the daemon rather
than against a description of it. Unknown fields are refused on input, so a typo
in a zone set is an error rather than a zone with a default.

### Clients

Two, and only two. Neither holds logic the daemon does not; both are views onto
the API.

| | For | Is |
|---|---|---|
| **Web client** | the console | custom elements under `/elements/mousewheeld.js`, embedded in the binary, served at the daemon's own version — statemachined's `/elements/` contract (`docs/developer/daemon.md` §5). No build step, no framework, no CDN |
| **Python client** | configuring from a script or a notebook | `client/python`, package `mousewheeld-client`: typed wrappers over the HTTP API — device, calibration (including the guided measurement), config, zone sets, arm/disarm, marks and paths |

The Python client is **for configuring, not for the fast path.** It never reads
shm and never needs to. A script that wants the live stream subscribes to the
ZMQ socket with `pyzmq` and the generated `events_pb2`, which the client ships as
a convenience; a script that wants a trial's path asks `marks/{id}/path`. triald
talks HTTP directly and does not depend on this package, as it does not depend on
statemachined's.

### Repository layout

```
mousewheeld/
├── README.md · BUILD.md · LICENSE
├── Cargo.toml                 workspace: daemon
├── platformio.ini             teensy41 · esp32
├── CMakeLists.txt             host build of the firmware core, for the tests
├── firmware/                  see Firmware
├── proto/mousewheeld/v1/events.proto
├── daemon/                    the binary
│   ├── Cargo.toml             depends on vinput (vstimd, pinned tag)
│   ├── build.rs               prost-build
│   └── src/
│       ├── main.rs            `mousewheeld serve` · `mousewheeld relay`
│       ├── realtime/          the thread: serial_link · framing · sample_parser ·
│       │                      clock_correlation · continuity · vinput_writer ·
│       │                      event_publisher · path_ring · recording_writer
│       ├── device/            device_session: hello, uploads, retry, reconnection
│       ├── model/             axis · calibration · zone_set · config · line_map
│       ├── zones/             zone_set_store · zone_set_compiler · patch_resolution
│       ├── calibration/       measurement_procedure
│       ├── api/               one *_routes.rs per group in *The API*, openapi.rs
│       ├── relay/             ZMQ SUB → local vinput shm
│       └── mdns_service_advertisement.rs
├── web/elements/              mousewheeld.js and its panels (embedded)
├── client/python/             mousewheeld-client
├── tests/core/                firmware core, mirrors firmware/core
├── packaging/                 nfpm · systemd · udev · sysusers · setcap
└── dev/PLAN.md                this file
```

File names are long on purpose, as in statemachined: `zone_set_compiler.rs`, not
`compile.rs`.

---

## Publishing

### (F) vinput shared memory — to vstimd, on the same host

As `vstimd/dev/INPUT_LATENCY.md` §4 specifies it:

| | |
|---|---|
| name | `/vstimd_wheel` (configurable) |
| header | magic `"VIN1"`, version, `n_axes`, device name `mousewheeld` |
| axis descriptor | `wheel` · `Cumulative` · `scale = 1.0` (cm) — a ball later adds `x`, `y`, `yaw` |
| state | seqlock · `[f64; N]` · writer heartbeat, monotonic ns |

The real-time thread is a producer through `VinputOwner` from vstimd's `vinput` crate,
**pinned to a vstimd release tag**. Depending on the crate rather than
reimplementing the layout is the one exception to "no shared code", and it is not
really one: the layout is a contract between a writer and a reader, and the
reader's own definition of it is the only copy that cannot drift. `vtl` is
consumed the same way by `gpiochip-daqd`.

**Status on vstimd's side: none of it exists.** No `vinput` crate, and
`ExternalPosition2D` never reads shm. mousewheeld is its first producer, so M3 is
built together with vstimd's `vinput`, `LinearNav3D` and the `[[input.device]]`
rig-config section, and the calibration change in *Calibration* lands in
`INPUT_LATENCY.md` §6 at the same time.

Write order per sample: **shm first**, then everything else. The camera is the
only consumer that cannot wait for a publish, a ring insert or a file write.

### ZMQ PUB — to anyone, on any host

The same shape as `vstimd/proto/vstimd/v1/events.proto`, deliberately, so a
subscriber that reads vstimd's events reads these with the same code:

- two frames, `[topic][Event]`; topics `sample`, `zone_hit`, `link`, `calibration`;
- `sequence` across the stream and `topic_sequence` per topic — PUB drops for a
  slow subscriber and says nothing, so the gap is the only loss signal;
- every sample carries `device_us`, `host_monotonic_ns`, counts and centimetres;
- `--event-port` (default **5557**, after vstimd's 5555 REP and 5556 events) and
  `--no-events`; the port goes in the mDNS TXT record.

**Protobuf, not JSON**, because this socket's precedent is vstimd's and a
subscriber should not need two decoders for one kind of stream. HTTP and the
WebSocket stay JSON, because their precedent is statemachined's and a browser is
on the other end.

**A camera on another host** is served by a **relay** — `mousewheeld relay`, a
subcommand of the same binary and a subscriber on the vstimd host that writes local vinput shm. vstimd stays exactly
as it is and still has no network position backend, which `INPUT_LATENCY.md` §5.5
declines on purpose. PUB loss becomes a larger delta on the next frame, which a
cumulative axis absorbs; a dead link becomes a stale heartbeat, which vstimd
already handles.

### Recording

The real-time thread records every sample and zone hit, lossless, to a per-session file
(binary, length-prefixed `Event` protobufs, the same messages as the PUB socket).
This is **the path of record**. triald's per-event payloads are capped at 64 KiB
and bulk data "belongs in its own file referenced by path" (triald `dev/PLAN.md`,
*Custom messages*); this is that file.

---

## The API

HTTP+JSON on **8082** (statemachined 8081, vstimd 8080, triald 8420). Units in
names, as in vstimd: `position_cm`, `counts_per_cm`, `velocity_cm_s`.

### Device

| | |
|---|---|
| `GET /api/device` | board, firmware, capacities, link state, calibration in force |
| `POST /api/device/connect` | |
| `GET /api/device/firmware` | |
| `GET /api/device/monitor` · `WS /api/device/monitor/stream` | the raw wire, both directions |

### State and stream

| | |
|---|---|
| `GET /api/state` | per axis: counts, `position_cm`, `distance_cm`, `velocity_cm_s`; armed and fired zones; link, stale, ring drops, seq gaps |
| `WS /api/stream?rate_hz=` | decimated state for browsers; names what a slow consumer lost |
| `POST /api/position/zero` | moves the API origin (never the published accumulator) |

### Path travelled

mousewheeld does not know what a trial is. It hands out **marks**:

| | |
|---|---|
| `POST /api/marks` `{label?}` | → `{mark_id, host_monotonic_ns, counts, position_cm}`. The label is opaque; triald writes `trial 42` |
| `GET /api/marks/{id}/path?until=now\|<mark_id>&samples=none\|decimated\|full` | summary `{displacement_cm[], distance_cm, duration_ms, max_velocity_cm_s, zone_hits[], seq_gaps}` and optionally the samples |
| `GET /api/path?from_ns=&to_ns=` | any window still in the ring buffer (default 30 min) |
| `/api/recording/{start,pause,resume,stop}` · `GET /api/recording/{name}` | as statemachined |

A summary names its own `seq_gaps`. A path with a hole in it says so rather than
reporting a shorter distance that looks complete.

### Calibration

| | |
|---|---|
| `GET` / `PUT /api/calibration` | per axis `counts_per_rev`, `diameter_cm` or `counts_per_cm`, `invert` |
| `POST /api/calibration/measure/start` `{axis, known_distance_cm}` | |
| `POST /api/calibration/measure/finish` | → measured vs configured |
| `POST /api/calibration/measure/apply` | persists; recorded as an event |
| `GET` / `PUT /api/calibration/ball` | **501** — the 2-D skeleton |

### Params

| | |
|---|---|
| `GET` / `PATCH /api/config` | stream rate, `display_hz`, device and host velocity windows, ring length, shm name, stale heartbeat period, event port, analog output |
| `GET /api/lines` | the output line map, read from the rig config. Not writable over the API: wiring is changed where wiring is described |

### Zones

| | |
|---|---|
| `GET /api/zone-sets` · `GET` / `PUT /api/zone-sets/{name}` | the store |
| `POST /api/zone-sets/{name}/validate` | compile against the current calibration and board capacities, without uploading |
| `POST /api/zones/arm` `{zone_set, patch?, origin, label?}` | resolve `$name`s, compile, upload if changed, wait for `armed`; → `{arm_id}` |
| `POST /api/zones/save` | write the set on the board, and its armed state, to board flash |
| `POST /api/zones/disarm` | |
| `GET /api/zones` | what is armed, what fired, when |

A zone set is compiled against a calibration. **Changing the calibration marks
compiled sets stale** and the next arm recompiles; a set armed under one
calibration is never silently reinterpreted under another.

### The console

`/elements/mousewheeld.js`, the contract statemachined's `docs/developer/daemon.md`
§5 defines: custom elements served by the daemon, talking to the daemon directly.
Panels: device and link, a live position and velocity trace, the zone-set editor
with a 1-D track diagram, calibration (the measurement procedure as a guided
panel), and the serial monitor. mDNS `_mousewheeld._tcp` with the TXT keys
statemachined publishes (`id`, `version`, `api`, `elements`, `device`, `port`)
plus `pub`. The console adds one line to `SERVICE_TYPES`.

---

## What this adds to the contracts

New rows in `contracts/INTERACTIONS.md` §3:

| | | Direction | Rate |
|---|---|---|---|
| **D** | triald → mousewheeld: mark, arm zone set | command | per trial |
| **E** | mousewheeld ⇢ triald: path since mark, zone hits | read / subscribed | per trial, or every event |
| **F** | mousewheeld ⇢ vstimd: vinput shm | fast bus | every sample |

Zone TTLs join the fast-bus table beside VTL.

### What triald does with it

Symmetric with interaction A:

- `TrialType` gains **`mousewheel_zone_set: str`** — a name, never a slot, empty
  meaning "arm nothing". mousewheeld holds the store and refuses a name it does
  not have, which triald reports as a configuration error. Spelled with the
  prefix on triald's side for the reason `statemachine_graph` is.
- **At configure:** `POST /api/marks {label}`, then `/api/zones/arm` if a set is
  named.
- **At the end:** `GET /api/marks/{id}/path` → a **contribution**
  (`PATCH /api/trial/current`, `source: "mousewheeld"`) carrying the summary and
  a reference to the recording by path and mark id.

Two triald dependencies, both already planned there and neither built: the
outbound client (INTERACTIONS §10 item 5) and contributions (triald
`dev/PLAN.md`, *Contributions, not one report*).

---

## Relationship to statemachined

**Siblings, not a library.** The APIs have almost nothing in common — a state
machine names outcomes, a wheel counts — and contracts' rule is no shared code.
What transfers is what statemachined *measured*: the performance rules above, the
framing and CRC conventions, the one-way layered core, host-built unit tests,
Renode-free hardware tests with asserted budgets, packaging and the console
contract. Each is rewritten here to this project's shapes.

---

## Testing

| Tier | What |
|---|---|
| core (CMake, doctest) | counter extension across wrap, origin and odometer, every zone semantic (edge vs level, once, rearm, hysteresis, wrap, open bounds, two axes), ring drop accounting, framing, JSON refusals, the fixed-format writers |
| daemon (cargo test) | framing, the sample parser against the firmware's writer output, continuity across reset/reconnect, clock mapping, vinput read-back through vstimd's own reader |
| daemon integration (cargo test) | against `native` firmware on a pty — whole arm/fire/path cycles with scripted counts, through the HTTP API |
| Python client (pytest) | against a running daemon with `native` firmware; the client's models checked against `/api/openapi.json` |
| hardware (`make test-hardware`) | scan rate, link cost per line and per command, seq gaps per stream rate, zone TTL latency on a scope pin; a quadrature signal from a second board as the encoder |
| end to end (contracts) | triald marks and arms, a scripted wheel crosses a zone, statemachined's input sees the TTL, the trial record carries the path |

---

## Decided

| | |
|---|---|
| License | **AGPLv3-or-later** for the whole repository, firmware included, matching vstimd, triald and the console. Nothing here lifts GPL code |
| Stream rate | **200 Hz** default; 100 Hz is enough for the record. At least the display rate where the camera follows the wheel (*The wire protocol*) |
| Velocity | **Both**, device and host, both recorded (*Velocity*) |
| Path ring | **30 min** in memory. A mark older than that is answered from the recording if one is running, `410 Gone` if not |
| Zones and lines | Output lines in the **rig config**; zone sets **dynamic** in the daemon's store, per-trial patches, optionally flashed to the board (*Trigger zones*) |
| Daemon | **One Rust binary**, vstimd's stack (*The daemon*) |
| Clients | A **web client** for the console and a **Python client** for configuring; nothing else (*Clients*) |
| Relay | A **subcommand** of the same binary, `mousewheeld relay` — one package, no Python on the vstimd host either way |

## Open questions

1. **Board pin sets.** Which pins carry the encoder inputs (fixed by the
   hardware decoders on the Teensy), zone output lines and analog output on each
   target, in `docs/operations/hardware.md` — settled at M1 against a real board.

---

## Milestones

| | | |
|---|---|---|
| **M0** | ⬜ | Firmware core: counter extension, origin/odometer, zones, rings, framing, JSON reader, fixed writers. Host tests green under gcc/clang and sanitizers |
| **M1** | ⬜ | Teensy 4.1 HAL, then ESP32 HAL. Analog output, flash, debug mode. **Scan rate and link cost measured and asserted** |
| **M2** | ⬜ | Daemon: real-time thread (link, parser, clock, continuity, ring, recording) and the control side — device session, state, stream, calibration, config, marks, OpenAPI. Python client |
| **M3** | ⬜ | vinput producer, **built with vstimd's `vinput` crate and `LinearNav3D`**. Encoder-to-photon latency measured |
| **M4** | ⬜ | ZMQ PUB with `events.proto`; `mousewheeld relay` |
| **M5** | ⬜ | Zones end to end: line map from the rig config, store, patches, compiler, arm, flash, TTL into statemachined and daqd |
| **M6** | ⬜ | Web client (console elements), mDNS, packaging |
| **M7** | ⬜ | triald: marks, `mousewheel_zone_set`, contribution; a contracts end-to-end stage |
| — | ⬜ | 2-D ball: optical sensor HAL, ball calibration, `x`/`y`/`yaw` descriptor — when the hardware exists |
