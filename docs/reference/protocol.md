# mousewheeld — the wire protocol

> **Status:** specification, and the daemon's half is built against it. The
> firmware is written against this document, not the other way round.

The link between the **daemon** (a host process) and the **device** (firmware on
a microcontroller). USB CDC on the Teensy, a UART bridge on the ESP32;
newline-delimited JSON, a per-line identifier and a CRC.

The framing rules are statemachined's, restated here rather than shared — the
two protocols have almost nothing above the framing in common, and a shared
codec would couple a wheel to a state machine for the sake of forty lines. What
is *not* restated is the reasoning; that is
[statemachined's `docs/reference/protocol.md`](https://github.com/braemons/statemachined/blob/main/docs/reference/protocol.md) §1,
and it holds here unchanged.

Everything above the framing follows from three constraints:

- **The device knows counts and nothing else.** No centimetres on this wire, and
  no floating point in any value the device produces. Calibration is the
  daemon's, and every number that leaves the daemon is in centimetres.
- **`sample` is the only high-rate message**, and the protocol is shaped around
  it: a fixed member order, integers only, and a writer on the device that
  formats it without a general serializer.
- **Nothing that formats a line runs in an interrupt.** The scan pushes
  fixed-size structs into a ring; `loop()` formats them. That is a firmware
  rule, but it is visible here — it is why `sample` carries a batchable shape
  and why the device is allowed to drop rather than block.

---

## 1. Framing

One JSON object per line, terminated by a single `\n` (0x0A). A `\r` immediately
before the `\n` is accepted and ignored.

```
{"msg_type":"ping","message_id":41,"crc":"A3CE"}\n
```

| | |
|---|---|
| Encoding | ASCII. A byte ≥ 0x80 anywhere in a line is a framing error |
| Line length | At most `max_line` bytes including the `\n`; the device reports its limit in `hello_ack`. The reference board's is **512** |
| Object depth | At most 4 |
| Unknown members | **Ignored**, on both sides. This is how the protocol gains fields without a version bump |
| Unknown `msg_type` | Answered with `error` / `unknown_type`. Never silently dropped |
| Member order | Free, **except** `crc`, which is always last |

### 1.1 The CRC

`crc` is exactly four uppercase hex digits: **CRC-16/CCITT-FALSE** (polynomial
`0x1021`, initial value `0xFFFF`, no reflection, no final XOR) over the bytes of
the line **preceding** the literal `,"crc":`.

```
{"msg_type":"ping","message_id":41,"crc":"A3CE"}
└       covered by the CRC       ┘└not covered ┘
```

`crc` is last so a receiver can find it without parsing: scan backwards from the
`}` for `,"crc":"`, CRC everything before it, compare. One pass, no buffer.

A line whose CRC does not match is answered with `error` / `bad_crc` naming the
`message_id` if one could be read, and is otherwise dropped. **It is never acted
on, not even partially.** A CRC is not security; it catches the failure that
actually happens on this link — a truncated or spliced line after a USB
re-enumeration — early enough that a corrupt zone set is refused instead of
armed.

### 1.2 `message_id` is not `seq`

`message_id` is an unsigned 16-bit counter, independent per direction, wrapping
through zero. It exists for link-level retry and for nothing else.

`seq`, on `sample` and `zone_hit`, is a **sequence**: contiguous by
construction, and a gap in it means the daemon lost a line. The two are never
conflated, and the names are different for that reason.

**A consumer of the daemon's own stream must not read loss off `seq`.** The
daemon decimates for its subscribers, so `seq` skips there by design; what a
consumer reads is `lost_before`, which only the daemon can compute. See the
daemon's API, `WS /api/stream`.

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
| `zones_end` | `{checksum}` | `ok`, or `error` naming what did not fit |
| `arm` | `{arm_id, zone_set_version, origin}` | `armed` |
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

A zone, as the device receives it — **integer counts, no names, no units**:

```jsonc
{"msg_type":"zone","message_id":18,
 "i":0,                     // index in the set
 "ax":[0],                  // axis indices, in the order the bounds are given
 "m":0,                     // metric: 0 displacement, 1 distance
 "lo":[17384],              // low bounds, counts; null is open
 "hi":[null],               // high bounds
 "wrap":0,                  // period in counts, 0 for a straight track
 "fire":0,                  // 0 once, 1 rearm
 "hy":174,                  // hysteresis, counts
 "lvl":false,               // fire on arming if already inside
 "line":0,                  // output line index
 "act":0,                   // 0 pulse, 1 level
 "ms":10,
 "crc":"...."}
```

The daemon compiles centimetres into these counts against a named calibration,
which is why **changing the calibration marks every compiled set stale** and the
next arm re-uploads.

---

## 3. Device → host

| `msg_type` | Payload |
|---|---|
| `hello_ack` | `{board, firmware, protocol_version, n_axes, max_zones, max_lines, max_line, scan_hz, flashed:{name,version}?}` |
| `sample` | `{seq, t_us, c:[counts…], v:[counts/s…]?}` |
| `zone_hit` | `{seq, arm_id, zone, t_us, c:[counts…]}` |
| `armed` | `{arm_id, zone_set_version}` |
| `state_report` | counts, origins, armed zones, analog config, scan health, ring drops |
| `ok` | `{answers}` — the `message_id` of the command it acknowledges |
| `error` | `{code, detail, answers?}` |
| `log` | `{text}` |
| `pong` | `{answers}` |

**A reply names what it answers in `answers`, never in `message_id`.** Every
line carries its own `message_id` — the sender's per-direction counter — so a
reply that put the id it was answering in the same member would carry the field
twice and be refused as malformed. They are different numbers with different
jobs, and the mistake is cheap to make: this one was made, and the daemon's own
parser caught it the first time the link ran.

### 3.1 `sample`

```
{"msg_type":"sample","message_id":903,"seq":41822,"t_us":8391204,"c":[173884],"crc":"1F0C"}
```

- **Cumulative counts, never deltas.** A lost line costs resolution, never
  distance: the host differences successive totals, so a gap is folded into the
  next sample instead of disappearing. This is the same argument the shared
  memory segment makes for `Cumulative` axes, and it is the reason a wheel is
  safe to read at any rate.
- **`seq` is contiguous.** A jump is loss, and the daemon counts it.
- **`t_us` is the device clock**, free-running from boot. Correlating it with
  `CLOCK_MONOTONIC` on the host is the daemon's job, and host monotonic time is
  the join key with statemachined's trace and vstimd's vblank timestamps.
- **`v` is optional** and is the device's own velocity — counts per second over
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
dropped `zone_hit` is a lost line in a log; it is never a lost pulse.

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
