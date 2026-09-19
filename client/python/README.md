# mousewheeld-client

The locomotion daemon's API, as Python types.

```python
from mousewheeld import Rig

with Rig("rig.local") as rig:
    rig.open_link()
    rig.arm("goal", patch={"goal_cm": 180})

    for frame in rig.watch_state(rate_hz=50):
        print(frame.axes[0].position_cm)
```

Speaks gRPC to a daemon that owns a wheel. The interface it speaks is authored
by hand in that daemon's `proto/mousewheeld/v1/` — types *and* rpcs — and this
package is generated from it and then hidden behind types of its own.

## No protobuf types come out of this package, and none go in

The generated code lives in `mousewheeld._proto`, which is private;
`mousewheeld.types` is the vocabulary and `mousewheeld._convert` is the seam.
Somebody writing a session should not have to learn a generated API to read a
number, and this package can keep a name on the day the interface adds a field.

It buys three things you notice within a page of writing a script:

* **`None` means absent.** protobuf spells an absent float as a message with no
  field set and an absent enum as zero, and both read as a value. Here a
  calibration that has never been measured has `measured_at is None`, and a
  zone with no upper bound has `max_cm == (None,)`.
* **Enums are the short names** — `ZoneMetric.DISPLACEMENT`, not
  `ZONE_METRIC_DISPLACEMENT`. The long spelling exists so that clients in
  different languages agree about the same byte; a person typing a script is
  not a wire. A value this build has never heard of *raises* rather than
  becoming the first entry in the table.
* **A quantity spells its unit**, as everywhere else in this family:
  `position_cm`, `velocity_cm_s`, `rate_hz`. The same names travel through the
  proto, the rig config, the zone-set files and the console panels.

## Refusals

`Refused` carries four things, and the one to branch on is `error`:

```python
from mousewheeld import Refused

try:
    rig.arm("goal")
except Refused as refusal:
    refusal.error     # 'unresolved_reference' — stable, machine-readable
    refusal.detail    # '$goal_cm is not in the arm patch'
    refusal.context   # 'goal' — what to change
    refusal.status    # 'invalid_argument' — the gRPC code, the category
```

The first three come from `mousewheeld.v1.Error` in the call's trailing
metadata, not from parsing the sentence. `NotConnected` is the subclass for
`unavailable` — a daemon that is not up yet, a board not plugged in — and it is
the only refusal `retryable` is true for: everything else is asking you to
change something, and a loop that retried it would hammer a rig about a typo.

## A command line, too

`mousewheel` — without the `d`, because the daemon installs a binary called
`mousewheeld` on the same rigs. Everything prints JSON, so it pipes into `jq`:

```console
$ mousewheel --rig rig.local state | jq '.axes[0].position_cm'
$ mousewheel --rig rig.local arm goal --patch goal_cm=180 --label 'trial 42'
$ mousewheel --rig rig.local set goal > goal.json      # the file as it is on disk
$ mousewheel --rig rig.local check --file goal.json    # compiled, not stored
$ mousewheel --rig rig.local watch --rate-hz 10 | jq -c '.axes[0].velocity_cm_s'
```

## Development

```console
$ make proto        # regenerate mousewheeld/_proto/ from ../../proto
$ make check-proto  # fail if the committed stubs are not what proto/ produces
$ make test         # the seam with no daemon, and the client against one
$ make typecheck    # ty
$ make check        # all three
```

The generated stubs are committed, so installing needs no protoc and
`grpcio-tools` is a development dependency rather than a runtime one
(`contracts/DAEMON_LAYOUT.md`). The test suite starts its own daemon —
`--simulate`, a wheel on a thread behind a real pty, on a free port, in a
temporary directory — so it exercises the same services, refusals and stream a
rig does and needs no hardware. It skips when there is no release binary to
start, so `cargo build --release` in the repository root first.
