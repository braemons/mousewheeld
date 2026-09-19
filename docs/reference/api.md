# mousewheeld — the API

> **Status:** written against the daemon that serves it, and kept beside it. The
> interface itself — every type and every rpc — is `proto/mousewheeld/v1/`, and
> the Rust the services implement is generated from it. **This is not a second
> description of that**, which is why it does not repeat field lists. What it
> says is the part a schema cannot: what an rpc is *for*, when to use one rather
> than another, and what a refusal means.

**The API is gRPC**, on the same port the console panels are served from. Five
services — `Device`, `StateService`, `Zones`, `Calibration`, `Config` — and
thirty-two rpcs. The daemon also answers **server reflection**, so a client
discovers all of it from a running daemon without the `.proto` to hand:

```console
$ grpcurl -plaintext rig.local:8082 list
mousewheeld.v1.Calibration
mousewheeld.v1.Config
mousewheeld.v1.Device
mousewheeld.v1.StateService
mousewheeld.v1.Zones
$ grpcurl -plaintext rig.local:8082 mousewheeld.v1.StateService/ReadState
```

**Browsers speak the same port too.** `tonic-web` translates gRPC-Web in the
daemon, so the console panels call these rpcs directly and there is no proxy to
deploy and nothing to configure. `client/web/src/daemon_api_client.js` is the
whole of the browser's side of it.

Two callers, and they want different things.

- **triald** drives the trial loop. Per trial it places a mark and arms a zone
  set, and at the end it asks what the animal did. What it needs is small,
  stable, and in the critical path of every trial.
- **A person** — through the console panels, or `grpcurl` on the rig — needs
  everything else: is a board attached, what is the calibration, why did that
  zone not fire, what went down the wire.

Both get the same rpcs, and both get a generated client. That is the change from
what was here before: this used to be HTTP+JSON so that `curl` and `webread`
would work with nothing installed, and it is not any more, because these
services do not do CRUD. `Arm`, `StartMeasuring`, `SaveToFlash` and
`ApplyMeasurement` are *commands* — a verb and an outcome, not a resource and a
method — and spelling them as routes made every one of them a small invention.
`contracts/INTERACTIONS.md` §7 is the argument in full.

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

**Unknown fields in a *response* are for the consumer to ignore**, which is what
lets an old console talk to a new daemon. In a *request* they are refused by
name on the console's JSON path, before a handler sees one — a command that does
part of what was asked is worse than one that does none.

On the binary wire they are not, and cannot be: prost keeps unknown fields
rather than refusing them, which is the same property that buys the tolerance in
the other direction. So do not send this daemon a field it has never heard of
and expect to be told; ask `ReadVersion` first. `contracts/INTERACTIONS.md` §11
has the asymmetry and what actually enforces it.

**Every rpc has its own request message**, including the eighteen that carry
nothing. `google.protobuf.Empty` can never grow a field, so an rpc that took one
would need a second rpc the day it learns an argument; `ReadVersionRequest{}`
just grows one. Additive growth is the whole of §11's versioning rule.

### Refusals

A refusal is a gRPC status, and the status code carries the category:

| Code | Means |
|---|---|
| `not_found` | it does not exist — `no_such_zone_set`, `no_such_axis` |
| `failed_precondition` | the daemon is not in a state where this means anything — `not_measuring`, `nothing_armed` |
| `invalid_argument` | understood and refused — `bad_zone_set`, `unresolved_reference`, `no_such_line` |
| `unimplemented` | modelled and not built — the 2-D ball |
| `unavailable` | `no_device`: nothing is wrong with the request, and it will work when a board is attached. The one a caller may retry unchanged |

The category is not the whole refusal, so the refusal also travels as itself.
`mousewheeld.v1.Error` is encoded into the trailing metadata entry
**`mousewheeld-error-bin`**:

```json
{"error": "no_such_zone_set", "detail": "no zone set named corridor", "context": "corridor"}
```

`error` is stable and machine-readable — this is what a client switches on;
`detail` is one sentence for a person; `context` names **what to change**, a
zone-set name or the field that did not fit. A UI that renders only
`NOT_FOUND` throws away the useful half. Both clients read all three, and
`daemon/src/grpc/mod.rs` is the single place an `ApiError` becomes a status.

The status *message* carries `detail (context)` as well, for everything that has
not been told about the metadata: `grpcurl`, a log line, a failed test.

---

## 2. The services

### `Device`

| | |
|---|---|
| `ReadVersion` | the daemon's release, the **API contract's** major version, and the device protocol (what this daemon sends, the oldest it speaks, what the board greeted with). Advertised, never gated on. Answers without a board |
| `ReadDevice` | board, firmware, capacities, link statistics, what is on the board's flash |
| `OpenLink` | opens the link, or says why not. Idempotent. Named `OpenLink` and not `Connect` because a generated client already has a `connect` — the one that opens the channel |
| `ReadFirmware` | what is running, and what could be flashed |
| `ReadWireLog` | the wire's recent past: **the whole conversation**, and the last hundred samples. Samples are capped rather than the conversation, because at 500 Hz they are the only thing that would come back |
| `WatchWire` | the wire as it goes, both directions, uninterpreted |

### `StateService`

| | |
|---|---|
| `ReadState` | per axis: counts, `position_cm`, `distance_cm`, and **both** velocities — the host's, from the sample path, and the device's, from counts over a window in its scan. They answer different questions and are never averaged together. Plus `seq_gaps`, `ring_drops`, the measured rate, and whether the link has gone quiet |
| `WatchState` | the decimated stream. Each frame carries `lost_before`: samples the daemon **never received**, accumulated across the ones decimation dropped. A consumer breaks its line on that and on nothing else — `seq` skips here by design |
| `ZeroPosition` | moves the **API origin**, on the board and on the host's mirror of it. Never the accumulator a camera differences: a value that jumps backwards jumps the corridor backwards. Teleporting a corridor is a command to vstimd, from whoever runs the experiment |

### `Zones`

A zone is a distance at which a trigger line fires, decided **on the device**,
in the scan that sees the count. This API stores the sets, compiles them and
arms them; it is not in the path of a hit.

| | |
|---|---|
| `ListZoneSets` | the names in the store |
| `ReadZoneSet` · `ReplaceZoneSet` | one set as a **message**, **as authored** — `$name` references intact, centimetres intact. The store is what a person edits and copies between rigs; compilation happens at arm |
| `ReadZoneSetFile` · `WriteZoneSetFile` · `ValidateZoneSetFile` | the same set as the **file** it is stored as, carried as text. What an editor with a cursor in it is editing is bytes: a file that does not parse still has to travel, and it comes back refused with a line and a column |
| `ReadZoneSetSchema` | the **file's** JSON Schema (2020-12), as the running daemon believes it. Also `mousewheeld schema` on the rig, and committed at `docs/reference/zone-set.schema.json`, because an editor and CI must reach it without a daemon running |
| `ValidateDraft` | compile a draft message — what a client is building, saving nothing |
| `ValidateZoneSet` | compile what is **stored** — "is that set still good after the calibration changed" |
| `ReadArmed` | what is armed, what has fired, and each zone's bounds **as armed**: the `$name`s resolved and the counts the board is comparing against, converted back to centimetres |
| `Arm` | resolve the patch, compile, upload, and **wait for the board to say `armed`**. A timeout is a refusal: reporting an arm that never happened would let a trial run believing a line is armed that is not |
| `Disarm` | |
| `SaveToFlash` | write the armed set to the board's flash, so a standalone rig comes up with its zones. The store on the host stays the source |

**Two spellings of a zone set, and both are real.** The message is what a
generated client builds; the file is what a person edits, with `"displacement"`
for a metric and `"$goal_cm"` for a reference. The daemon converts between them
— once, in `daemon/src/convert/zones.rs` — and serves both, so that no client
has to. A browser that did its own conversion would be a third description of a
zone set, disagreeing with the other two the first time either changed.

**Per-trial values.** A numeric bound may be `"$name"`, resolved from the
`patch` given at arm time — `{"min_cm": ["$goal_cm"]}` with
`arm {patch: {"goal_cm": 180}}`. Substitution happens host-side; the board never
sees a `$name`, and one that reaches the compiler is a refusal rather than a
zero. A goal distance that silently became 0 cm is a trial that looks like it
ran.

### `Calibration`

| | |
|---|---|
| `ReadCalibration` · `ReplaceCalibration` | counts per centimetre, per axis, with the wheel's dimensions kept for the arithmetic a measurement is checked against |
| `StartMeasuring` | name an axis and the distance you are about to roll |
| `FinishMeasuring` | measured against configured, the circumference it implies, and a warning when the ratio is a whole number |
| `ApplyMeasurement` | make it the rig's truth |
| `ReadBall` · `ReplaceBall` | **`unimplemented`** — the 2-D skeleton |

**Measure it; do not compute it from the nominal diameter.** What a corridor
position is made of is how far the *feet* travelled, on the surface the animal
runs on, with whatever tread is on it — a few percent off the arithmetic every
time. A measurement that comes out a whole-number multiple of the configured
value is a decoder counting ×1 or ×2 where the counts-per-revolution assumed ×4;
it looks exactly like a wheel of the wrong size, and the report says so before
anybody applies it.

**Three presses, not one.** Applying invalidates every compiled zone set and
disarms, so it is a separate decision from taking the measurement.

### `Config`

| | |
|---|---|
| `ReadConfig` · `PatchConfig` | the stream rate, the display rate it is compared against, the ring length — and whether the shared-memory segment is actually **open**, with the number of writes that have reached it. A name in a config file and a mapped segment are different things, and the difference is the whole reason a corridor sometimes does not move |
| `ReadLines` | the output line map. **Not writable:** wiring is changed where wiring is described, in the rig config |

**`rate_hz` should be above `display_hz`.** At or below it, some frames see no
new sample and the next sees two: the corridor advances in uneven steps, at
exactly the speeds a running animal produces, with every individual sample
correct. The daemon warns; `starves_the_display` says so in the answer.

---

## 3. What a trial does

Symmetric with triald's other participants, and trial-blind throughout — a
label is opaque text this daemon stores and hands back unread.

1. **At configure:** `Zones.Arm` with the trial's set, its patch, and
   `ARM_ORIGIN_CURRENT` so displacement and distance start at zero.
2. **During:** nothing. A zone hit is a TTL edge into statemachined, on the fast
   bus, decided on the device. This API is not in that path.
3. **At the end:** read the path back. *(Marks and paths are M2's remainder and
   not built yet; until they are, `Zones.ReadArmed` says what fired and where.)*

---

## 4. Not built yet

Named here so a reader does not go looking: **marks and paths**, **recording**,
and the **ZMQ event stream**. They are `dev/PLAN.md`'s M2 and M4.
