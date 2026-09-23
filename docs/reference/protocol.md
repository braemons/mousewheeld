# mousewheeld — the wire protocol

> **Status:** specification, and the daemon's half is built against it. The
> firmware is written against this document, not the other way round.

The link between the **daemon** (a host process) and the **device** (firmware on
a microcontroller). USB CDC on the Teensy, a UART bridge on the ESP32;
**protobuf messages in COBS frames with a CRC-16**.

The shapes are [`proto/mousewheeld/link/v1/link.proto`](../../proto/mousewheeld/link/v1/link.proto),
generated with prost for the daemon (`make proto`, into `daemon/src/wire/link/`)
and with nanopb for the firmware (`make firmware-proto`, into
`firmware/core/proto/`). This document is the prose: what the messages mean,
which the `.proto` cannot say. The member names below are the `.proto`'s.

This link used to be newline-delimited JSON, like statemachined's. It is
protobuf because the board-side cost of a JSON reader and writer — and of
keeping two hand-written codecs in step — bought nothing a generated one does
not, and nanopb decodes into fixed-size structs with no allocation.

Everything above the framing follows from three constraints:

- **The device knows counts and nothing else.** No centimetres on this wire, and
  no floating point in any value the device produces. Calibration is the
  daemon's, and every number that leaves the daemon is in centimetres.
- **`sample` is the only high-rate message**, and the protocol is shaped around
  it: integers only, cumulative, and small enough that a frame of them is tens
  of bytes.
- **Nothing that encodes a frame runs in an interrupt.** The scan pushes
  fixed-size structs into a ring; `loop()` encodes them. That is a firmware
  rule, but it is visible here — it is why `sample` carries a batchable shape
  and why the device is allowed to drop rather than block.

---

## 1. Framing

Both directions, identically:

```
COBS( protobuf message ‖ CRC-16, big-endian ) ‖ 0x00
```

The daemon sends a `HostMessage` and the board a `DeviceMessage`: a
`message_id` and a `oneof body` naming the message.

| | |
|---|---|
| Delimiter | A single `0x00`. COBS guarantees no other zero byte in a frame, so a receiver that lost bytes — a USB re-enumeration, a board that reset mid-frame — finds the next frame at the next zero without parsing anything. Back-to-back delimiters are not a frame, and a sender may use them to resynchronise a receiver |
| Frame length | At most `max_frame` bytes encoded, including the delimiter; the board reports its limit in `hello_ack`. The ESP32's is **256**, room for every message this protocol has |
| Unknown fields | **Ignored**, on both sides — protobuf does this. It is how the protocol gains fields without a version bump |
| Unknown `body` | Answered with `error` / `unknown_type`. Never silently dropped |

### 1.1 The CRC

**CRC-16/CCITT-FALSE** (polynomial `0x1021`, initial value `0xFFFF`, no
reflection, no final XOR) over the protobuf bytes, appended big-endian before
COBS encoding.

A frame that is not valid COBS, is too long, or whose CRC does not match is
answered with `error` / `bad_cobs`, `frame_too_long` or `bad_crc`, **without**
`answers` — nothing in a frame that failed its check can be believed, the
`message_id` included — and dropped. **It is never acted on, not even
partially.** A CRC is not security; it catches the failure that actually happens
on this link — a truncated or spliced frame after a USB re-enumeration — early
enough that a corrupt zone set is refused instead of armed.

### 1.2 `message_id` is not `seq`

`message_id` is an unsigned 16-bit counter — carried in a `uint32` — independent
per direction, wrapping through zero. It exists for link-level retry and for nothing else.

`seq`, on `sample` and `zone_hit`, is a **sequence**: contiguous by
construction, and a gap in it means the daemon lost a frame. The two are never
conflated, and the names are different for that reason.

**A consumer of the daemon's own stream must not read loss off `seq`.** The
daemon decimates for its subscribers, so `seq` skips there by design; what a
consumer reads is `lost_before`, which only the daemon can compute. See the
daemon's API, `StateService.WatchState`.

---

## 2. Host → device

| `msg_type` | Payload | Reply |
|---|---|---|
| `hello` | `{protocol_version}` | `hello_ack` |
| `axes` | `{axes:[{source,invert}]}` | `ok`, or `error`/`unsupported_source` naming the axis |
| `lines` | `{lines:[{index,pin,safe_high}]}` | `ok` |
| `stream` | `{rate_hz, velocity}` | `ok`. `rate_hz: 0` stops the stream |
| `zones_begin` | `{zone_set_version, n, name}` | — |
| `zone` | one compiled zone (below) | — |
| `zones_end` | `{}` | `ok`, or `error`/`incomplete_upload` naming the first index that never arrived |
| `arm` | `{arm_id, zone_set_version, origin}` | `armed`, or `error`/`version_mismatch` if the board holds another version |
| `disarm` | `{arm_id}` | `ok` |
| `zero` | `{axes:[index…]}` — empty for all | `ok` |
| `analog` | `{axis, mode, range_counts, lo, hi}` | `ok` |
| `save` | `{}` | `ok` |
| `ping` | `{}` | `pong` |
| `state` | `{}` | `state_report` |
| `debug` | `{on}` | `ok` |

The set's **name** goes with it because `hello_ack` reports a flashed set by
name, and a board that was only ever told a version cannot do that. It is the
one string the device stores.

**Uploads are chunked and atomic.** `zones_begin` opens a staging table; each
`zone` fills one row; `zones_end` commits it. The live set is untouched until
the commit, so a link that dies mid-upload leaves the board running what it was
running.

A zone, as the device receives it — **integer counts, no names, no units**
(`Zone` in the `.proto`):

| Field | |
|---|---|
| `index` | its place in the set; `zones_end` is refused unless every index in `0..n` arrived |
| `intervals` | per axis: `{axis, lo?, hi?}` in counts, an absent bound open |
| `metric` | `DISPLACEMENT` (counts − origin) or `DISTANCE` (the odometer) |
| `wrap` | period in counts, 0 for a straight track; bounds must lie in `[0, wrap)` |
| `fire` | `ONCE`, or `REARM` after leaving by `hysteresis` counts |
| `level_on_arm` | fire on arming if already inside |
| `line`, `action`, `pulse_ms` | the output line index; `PULSE` for `pulse_ms`, or `LEVEL` while engaged |

A zone is refused whole — `bad_zone`, `bad_axis`, `bad_line` — if the board could
not evaluate all of it: an axis or line it was not told of, a value from a newer
protocol, a pulse of 0 ms.

The daemon compiles centimetres into these counts against a named calibration,
which is why **changing the calibration marks every compiled set stale** and the
next arm re-uploads.

---

## 3. Device → host

| `msg_type` | Payload |
|---|---|
| `hello_ack` | `{board, firmware, protocol_version, n_axes, max_zones, max_lines, max_frame, scan_hz, flashed:{name,version}?}` |
| `sample` | `{seq, t_us, c:[counts…], v:[counts/s…]?}` |
| `zone_hit` | `{seq, arm_id, zone, t_us, c:[counts…]}` |
| `armed` | `{arm_id, zone_set_version, origin:[counts…]}` — the origin the board set, which the host adopts (below) |
| `state_report` | counts, origins, odometers, velocities, the armed set and which zones can still fire, analog config, ring drops, scan overruns and the longest scan, frames refused |
| `ok` | `{answers}` — the `message_id` of the command it acknowledges |
| `error` | `{code, detail, answers?}` |
| `log` | `{text}` |
| `pong` | `{answers}` |

**A reply names what it answers in `answers`, never in `message_id`.** Every
frame carries its own `message_id` — the sender's per-direction counter. They
are different numbers with different jobs, and the mistake is cheap to make:
this one was made, and the daemon's own parser caught it the first time the link
ran.

**`armed` carries the origin.** With `origin: CURRENT` the board sets its origin
when it reads the `arm`, which is later than any sample the host has seen. A
host that mirrored the origin from its last sample would report every position
for the set off by the stream's lag — 9 counts at 100 cm/s and 500 Hz, which the
daemon's test against the firmware caught.

### 3.1 `sample`

```
DeviceMessage{message_id: 903, sample: {seq: 41822, t_us: 8391204, c: [173884]}}
```

- **Cumulative counts, never deltas.** A lost frame costs resolution, never
  distance: the host differences successive totals, so a gap is folded into the
  next sample instead of disappearing. This is the same argument the shared
  memory segment makes for `Cumulative` axes, and it is the reason a wheel is
  safe to read at any rate.
- **`seq` is contiguous.** A jump is loss, and the daemon counts it.
- **`t_us` is the device clock**, free-running from boot. Correlating it with
  `CLOCK_MONOTONIC` on the host is the daemon's job, and host monotonic time is
  the join key with statemachined's trace and vstimd's vblank timestamps.
- **`v` is empty unless `stream` asked for it**, and is the device's own velocity — counts per second over
  a fixed window in the scan. Coarser than the host's, and the one the analog
  output and the zone evaluation were actually done against, which is why both
  are recorded and never averaged together.
- **At least the display rate**, on a rig whose camera follows the wheel: 250 Hz
  for 120 Hz, 500 Hz for 240 Hz. Below it, some frames see no new sample and the
  next sees two, and the corridor moves in uneven steps at exactly the speeds a
  running animal produces. The daemon warns when `rate_hz` is at or below the
  `display_hz` in its config.

### 3.2 `zone_hit`

Its own ring on the device, and **the TTL does not depend on it**: the output is
scheduled in the scan that saw the count, and the message is only the record. A
dropped `zone_hit` is a lost record in a log; it is never a lost pulse.

---

## 4. Reset, and what the host does about it

A device reset — a reflash, a USB re-enumeration, somebody's elbow — restarts
`t_us` and the counters at zero. The host notices it by a `hello_ack` it did not
ask for, or by a `seq` or `t_us` that went backwards.

**What must not happen is a step backwards in what the daemon publishes.** vstimd
differences its segment every frame, so a value that jumps back jumps the camera
back. The daemon holds an offset per axis and adds it to what the device
reports, so the published accumulator is continuous across a reset that the
device itself has no memory of. Origins — `zero`, an arm with
`origin: "current"` — move a *different* number, and never that one.
