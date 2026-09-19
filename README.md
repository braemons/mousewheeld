# mousewheeld

The **locomotion input** of a braemons rig: a rotary encoder on a running wheel,
read by a microcontroller and published to whoever needs to know how far the
animal went.

It has two consumers that want opposite things — [vstimd](https://github.com/braemons/vstimd)
moves a camera through a corridor and needs the position *every frame*, within
the frame; [triald](https://github.com/braemons/triald) records what happened
and needs the path *once per trial*, exactly. And one job neither can do:
**trigger zones** — "the animal has run 200 cm" — decided on the device, in the
scan that sees the count, and put on a TTL line.

```
   console ──▶│   mousewheeld    │──▶ ZMQ PUB   samples · zone hits  (anyone, any host)
   /elements  │ control │ data   │──▶ recording (the path of record)
              └──┬───────────┬───┘
   USB · NDJSON  │           │  vinput shm  /vstimd_wheel  ──▶ vstimd, every frame
   · CRC         │           ▼
              ┌──┴─────────┐
              │ firmware   │── TTL ──▶ statemachined · daqd
              │ counts · zones · analog out
              └────────────┘
```

## Read this first

- **[`dev/PLAN.md`](dev/PLAN.md)** — the design, and the reasoning behind each
  decision. Milestones and their state are at the end.
- The rule everything here follows is `contracts/INTERACTIONS.md` §2:
  **a participant publishes what it observed and commands nobody.** mousewheeld
  has no client of any other daemon, no `vstimd_address`, no `triald_base_url`.
  vstimd reads a shared-memory segment whose layout it defines and never learns
  who writes it; triald commands this daemon and subscribes to what it says.

## What is built

| | |
|---|---|
| **The API**, authored | `proto/mousewheeld/v1/` — types *and* rpcs, hand-written and reviewed. The daemon answers **gRPC server reflection**, so a client discovers every rpc from the running daemon. `daemon/src/wire/` is generated from it and committed; `daemon/src/convert/` is the seam to `daemon/src/model/`, which is what the daemon thinks in |
| **The wire** | [`docs/reference/protocol.md`](docs/reference/protocol.md), and `daemon/src/link/` — framing, CRC-16/CCITT-FALSE, the typed messages, the clock correlation and the continuity offset that keeps a published accumulator from stepping backwards across a board reset |
| **The zone-set store and compiler** | centimetres in, integer counts out, against a named calibration |
| **Calibration**, with its guided measurement | `daemon/src/grpc/calibration.rs` over `daemon/src/model/calibration.rs` |
| **The Python client and its CLI** | `client/python/` — `mousewheeld.Rig`, the API as frozen dataclasses, and `mousewheel` on a terminal. The generated stubs are private and committed; no protobuf type crosses the package boundary |
| **The console panels** | `client/web/elements/` — five custom elements served by this daemon at its own version. One generated file among them: `daemon_api_client.js`, the gRPC-Web client, bundled by `make web` and committed so a release build needs no npm |
| **Publishing to vstimd** | `daemon/src/publish/` — the `vinput` segment, written first of everything a sample causes, in centimetres, through vstimd's own crate pinned at `v0.3.0-alpha1` |
| **Packaging** | `packaging/` — nfpm, a systemd unit, a udev rule, sysusers |
| **The API** | [`docs/reference/api.md`](docs/reference/api.md), written by hand — what an rpc is for and what a refusal means. The interface itself is [`proto/mousewheeld/v1/`](proto/mousewheeld/v1/), so this document never repeats a field list |
| **The zone set's schema** | [`docs/reference/zone-set.schema.json`](docs/reference/zone-set.schema.json) — JSON Schema 2020-12, extracted from those types and committed, because a zone set is a *file*: point a `"$schema"` line at it and an editor checks it as you type. `make check` fails when it drifts |
| a board on a pty | `--simulate` — a simulator speaking the protocol on the far end of a real pty, so the daemon runs the link code it will run against a Teensy |

**Not built:** the firmware, the real-time thread's scheduling discipline, the
ZMQ publisher, the recording, marks and paths, the Python client. Everything that needs a board is M1; everything else is reachable
without one.

## Running it

```sh
make dev          # a board simulator on a pty; panels at http://127.0.0.1:8082/
make check        # build, clippy, tests
make schema       # regenerate docs/reference/zone-set.schema.json from the types
make package      # deb and rpm
```

`make dev` runs a debug build on purpose: `rust-embed` serves `client/web/elements/`
from disk there, so editing a panel and reloading the page is enough. A release
binary embeds them, and a panel that did not change after an edit is almost
always a release build.

On a rig:

```sh
systemctl enable --now mousewheeld     # after editing /etc/braemons/mousewheeld-rig-config.toml
```

## Calibrating

**Measure it; do not compute it from the wheel's nominal diameter.** What a
corridor position is made of is how far the *feet* travelled — inside the rim,
on whatever tread is glued to it — and that is a few percent off the arithmetic
every time. The Calibration panel walks the procedure: name a distance, roll the
wheel that far by hand in one continuous motion, and read the measurement
against the configured value. A whole-number ratio between them is a decoder
counting ×1 or ×2 where the counts-per-revolution assumed ×4; the daemon says so
before you apply it.

The calibration lives here and nowhere else. Everything this daemon publishes is
in **centimetres**, so vstimd's `[[input.device.axis]]` keeps `scale = 1.0` and
restates nothing.

## Versions

`Device.ReadVersion` reports three numbers that are not the same number: the
daemon's release, the **API contract's** major version, and the **device
protocol** — what this daemon sends, the oldest it will talk to, and what the
attached board greeted with. It also reports the `vinput` layout version, which
is the one interface here that is *checked* rather than advertised.

That split is the family's rule (`contracts/INTERACTIONS.md` §11): a version is
advertised between daemons and never gated on, because a gate turns every
upgrade into a coordinated one; it is checked on a shared-memory layout, where a
mismatch is bytes reinterpreted; and a device wire keeps a **floor** rather than
an equality, because firmware is flashed separately and will be older.

Requests refuse unknown fields and responses ignore them — a command that does
part of what was asked is worse than one that does none, and a consumer that
cannot survive a newer producer makes every upgrade a flag day.

## Security

Like every other daemon on a braemons rig: **no authentication, CORS open, and
the rig network is the security boundary.** That is a deliberate choice with one
precondition — the rig network must be *isolated*, not merely behind an
institute firewall.

## Licence

AGPL-3.0-or-later, for the whole repository, firmware included.
