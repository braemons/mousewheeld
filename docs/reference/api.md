# mousewheeld — the HTTP API

> **Status:** written against the daemon that serves it, and kept beside it. The
> interface itself — every type and every rpc — is `proto/mousewheeld/v1/`,
> served by the daemon at `/api/proto`, and the Rust the routes speak is
> generated from it. **This is not a second description of that**, which is why
> it does not repeat field lists. What it says is the part a schema cannot:
> what a route is *for*, when to use one rather than another, and what a refusal
> means.
>
> Two spellings to know before reading anything below. A 64-bit integer is a
> **string** on the wire (`"counts": "41822"`), because JSON numbers are
> doubles; and an enum is its full name (`"ZONE_SHAPE_RECT"`), so that clients
> generated in different languages agree about the same byte. Both are
> protobuf's JSON mapping, and `daemon/tests/wire_json.rs` holds the daemon to
> them.

Two callers, and they want different things.

- **triald** drives the trial loop. Per trial it places a mark and arms a zone
  set, and at the end it asks what the animal did. What it needs is small,
  stable, and in the critical path of every trial.
- **A person** — through the console panels or `curl` — needs everything else:
  is a board attached, what is the calibration, why did that zone not fire,
  what went down the wire.

The second is why this is HTTP+JSON and not a binary protocol. A translator can
be a library; a thing you can ask *"is the wheel actually turning"* at two in
the morning cannot. `webread` in MATLAB and `curl` on the rig both work with
nothing installed.

**What is not here:** the position itself, at rate. vstimd reads that from
shared memory, every frame, and a camera cannot wait for a socket
(`contracts/INTERACTIONS.md` §3, interaction F). This API configures the thing
that writes it.

---

## 1. Conventions

**Units are in the names.** `position_cm`, `counts_per_cm`, `velocity_cm_s`,
`rate_hz`. The same name travels through this API, the rig config, the zone-set
files and both clients, so it is spelled once. Dimensionless quantities carry no
suffix.

**Counts are the device's; centimetres are everybody else's.** The firmware
knows counts and nothing else. Every number that leaves this daemon is in
centimetres, because the consumers — a corridor, a trial record, a
distance-triggered line — all need the same unit and none of them can read this
daemon's calibration.

**A size is a full extent.** `diameter_cm`, never a radius.

**Unknown fields in a request are refused**, by name, with the field spelled
out. A command that does part of what was asked is worse than one that does
none. Unknown fields in a *response* are for the consumer to ignore — that
asymmetry is `contracts/INTERACTIONS.md` §11, and it is what lets an old console
talk to a new daemon.

### Refusals

Every refusal has the same shape, whatever raised it:

```json
{"error": "no_such_zone_set", "detail": "no zone set named corridor", "context": "corridor"}
```

`error` is stable and machine-readable; `detail` is one sentence for a person;
`context` names **what to change** — a zone-set name, the field that did not
fit. A UI that renders only the status code throws away the useful half.

| Status | Means |
|---|---|
| 404 | it does not exist — `no_such_zone_set`, `no_such_axis` |
| 409 | the daemon is not in a state where this means anything — `not_measuring`, `nothing_armed` |
| 422 | understood and refused — `bad_body`, `zone_set_refused`, `unresolved_reference`, `no_such_line` |
| 501 | modelled, routed, not built — the 2-D ball |
| 503 | `no_device`: nothing is wrong with the request, and it will work when a board is attached |

---

## 2. The routes

### Version and device

| | |
|---|---|
| `GET /api/version` | the daemon's release, the **API contract's** major version, and the device protocol (what this daemon sends, the oldest it speaks, what the board greeted with). Advertised, never gated on |
| `GET /api/device` | board, firmware, capacities, link statistics, what is on the board's flash |
| `POST /api/device/connect` | opens the link, or says why not |
| `GET /api/device/firmware` | what is running, and what could be flashed |
| `GET /api/device/monitor` | the wire's recent past: **the whole conversation**, and the last hundred samples. Samples are capped rather than the conversation, because at 500 Hz they are the only thing that would come back |
| `WS /api/device/monitor/stream` | the wire as it goes, both directions, uninterpreted |

### State

| | |
|---|---|
| `GET /api/state` | per axis: counts, `position_cm`, `distance_cm`, and **both** velocities — the host's, from the sample path, and the device's, from counts over a window in its scan. They answer different questions and are never averaged together. Plus `seq_gaps`, `ring_drops`, the measured rate, and whether the link has gone quiet |
| `WS /api/stream?rate_hz=` | the decimated stream. Each frame carries `lost_before`: samples the daemon **never received**, accumulated across the ones decimation dropped. A consumer breaks its line on that and on nothing else — `seq` skips here by design |
| `POST /api/position/zero` | moves the **API origin**, on the board and on the host's mirror of it. Never the accumulator a camera differences: a value that jumps backwards jumps the corridor backwards. Teleporting a corridor is a command to vstimd, from whoever runs the experiment |

### Zones

A zone is a distance at which a trigger line fires, decided **on the device**,
in the scan that sees the count. This API stores the sets, compiles them and
arms them; it is not in the path of a hit.

| | |
|---|---|
| `GET /api/zone-sets` | the names in the store |
| `GET` / `PUT /api/zone-sets/{name}` | one set, **as authored** — `$name` references intact, centimetres intact. The store is what a person edits and copies between rigs; compilation happens at arm |
| `GET /api/zone-sets/schema` | the set's JSON Schema (2020-12). Put its URL in a file's `"$schema"` line and an editor validates as you type; point a checker at it in CI |
| `POST /api/zone-sets/validate` | compile a **draft** — what somebody is typing, saving nothing |
| `POST /api/zone-sets/{name}/validate` | compile what is **stored** — "is that set still good after the calibration changed" |
| `GET /api/zones` | what is armed, what has fired, and each zone's bounds **as armed**: the `$name`s resolved and the counts the board is comparing against, converted back to centimetres |
| `POST /api/zones/arm` | resolve the patch, compile, upload, and **wait for the board to say `armed`**. A timeout is a refusal: reporting an arm that never happened would let a trial run believing a line is armed that is not |
| `POST /api/zones/disarm` | |
| `POST /api/zones/save` | write the armed set to the board's flash, so a standalone rig comes up with its zones. The store on the host stays the source |

**Per-trial values.** A numeric bound may be `"$name"`, resolved from the
`patch` given at arm time — `{"min_cm": ["$goal_cm"]}` with
`arm {patch: {"goal_cm": 180}}`. Substitution happens host-side; the board never
sees a `$name`, and one that reaches the compiler is a refusal rather than a
zero. A goal distance that silently became 0 cm is a trial that looks like it
ran.

### Calibration

| | |
|---|---|
| `GET` / `PUT /api/calibration` | counts per centimetre, per axis, with the wheel's dimensions kept for the arithmetic a measurement is checked against |
| `POST /api/calibration/measure/start` | name an axis and the distance you are about to roll |
| `POST /api/calibration/measure/finish` | measured against configured, the circumference it implies, and a warning when the ratio is a whole number |
| `POST /api/calibration/measure/apply` | make it the rig's truth |
| `GET` / `PUT /api/calibration/ball` | **501** — the 2-D skeleton |

**Measure it; do not compute it from the nominal diameter.** What a corridor
position is made of is how far the *feet* travelled, on the surface the animal
runs on, with whatever tread is on it — a few percent off the arithmetic every
time. A measurement that comes out a whole-number multiple of the configured
value is a decoder counting ×1 or ×2 where the counts-per-revolution assumed ×4;
it looks exactly like a wheel of the wrong size, and the report says so before
anybody applies it.

**Three presses, not one.** Applying invalidates every compiled zone set and
disarms, so it is a separate decision from taking the measurement.

### Config and lines

| | |
|---|---|
| `GET` / `PATCH /api/config` | the stream rate, the display rate it is compared against, the ring length — and whether the shared-memory segment is actually **open**, with the number of writes that have reached it. A name in a config file and a mapped segment are different things, and the difference is the whole reason a corridor sometimes does not move |
| `GET /api/lines` | the output line map. **Not writable:** wiring is changed where wiring is described, in the rig config |

**`rate_hz` should be above `display_hz`.** At or below it, some frames see no
new sample and the next sees two: the corridor advances in uneven steps, at
exactly the speeds a running animal produces, with every individual sample
correct. The daemon warns; `starves_the_display` says so in the answer.

---

## 3. What a trial does

Symmetric with triald's other participants, and trial-blind throughout — a
label is opaque text this daemon stores and hands back unread.

1. **At configure:** `POST /api/zones/arm` with the trial's set, its patch, and
   `origin: "current"` so displacement and distance start at zero.
2. **During:** nothing. A zone hit is a TTL edge into statemachined, on the fast
   bus, decided on the device. This API is not in that path.
3. **At the end:** read the path back. *(Marks and `GET /api/marks/{id}/path`
   are M2's remainder and not built yet; until they are, `GET /api/zones` says
   what fired and where.)*

---

## 4. Not built yet

Named here so a reader does not go looking: **marks and paths**
(`POST /api/marks`, `GET /api/marks/{id}/path`), **recording**, and the **ZMQ
event stream**. They are `dev/PLAN.md`'s M2 and M4.
