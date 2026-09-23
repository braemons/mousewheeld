// SPDX-License-Identifier: AGPL-3.0-or-later
//
// `<mousewheeld-trace>` — position and velocity as they arrive.
//
// **Not vstimd's input panel, and the difference is the reason both exist.**
// vstimd plots what it read from the shared-memory segment, once a frame,
// decimated again for its own snapshot stream: the consumer's view, good for
// "is the camera being driven and is that still true". This one is the
// device's: it follows `StateService.WatchState`, and it can show three things vstimd
// structurally cannot —
//
//   * **device velocity beside host velocity.** The device computes one from
//     counts over a fixed window in the scan; the host computes another from
//     the sample path. They answer different questions — what the board acted
//     on, and what the animal did — and drawing them together is how a
//     disagreement between them is ever noticed.
//   * **zone hits**, on the same time base, as the rules they are.
//   * **gaps**, drawn as gaps. A line drawn straight through a hole is a claim
//     about an animal nobody observed.
//
// That last one is a contract point and not a client-side trick. **`seq` cannot
// tell a client what was lost**, because this stream is decimated: at 30 Hz out
// of a 500 Hz device, `seq` skips six samples between every two frames by
// design, and a panel that broke the line on a `seq` jump would draw a stream
// of holes on a perfectly healthy rig. Only the daemon can tell decimation from
// loss, so it says which: `lost_before` counts samples it never received since
// the frame before, and a non-zero one is the break.
//
// It is still a monitor and not the record. The path of record is the
// recording, and a trial's own span comes from the mark's path.

import { BasePanelElement, defineElementOnce } from "./base_panel_element.js";
import { drawEventRules, drawTimeSeries } from "./time_series_chart.js";
import { count, streamFrame } from "./wire_shapes.js";

const WINDOWS_S = [5, 15, 60];
const WIDTH = 460;
const CHART_HEIGHT = 96;
/// Two minutes at the browser's stream rate: more than the longest window,
/// bounded so a console left open overnight does not grow without end.
const MOST_POINTS = 4000;
const BROWSER_RATE_HZ = 30;

export class TracePanelElement extends BasePanelElement {
  constructor() {
    super();
    this.points = [];
    this.events = [];
    this.windowS = 15;
    this.axisName = null;
    this.lastSeq = null;
    this.live = false;
  }

  renderShell() {
    this.failureSlot = this.make("div", { class: "failure-slot" });
    this.charts = this.make("div", { text: "waiting for the first sample…" });
    this.linkPill = this.make("span", { class: "pill warn", text: "connecting…" });
    this.axisPicker = this.make("select", {
      onChange: (event) => {
        this.axisName = event.target.value;
        this.points = [];
        this.events = [];
        this.paint();
      },
    });
    this.windowPicker = this.make("div", { class: "row" }, [
      this.make("span", { class: "muted", text: "window" }),
      ...WINDOWS_S.map((seconds) =>
        this.make("button", {
          text: `${seconds} s`,
          dataset: { seconds: `${seconds}` },
          onClick: () => {
            this.windowS = seconds;
            this.paintWindowButtons();
            this.paint();
          },
        }),
      ),
    ]);
    this.root.replaceChildren(
      this.make("section", {}, [
        this.make("h2", {}, [
          this.make("span", { text: "Trace" }),
          this.make("span", { class: "spacer" }),
          this.axisPicker,
          this.linkPill,
        ]),
        this.describe(
          "Where the axis is and how fast it is moving, as the daemon receives it. Velocity " +
            "is drawn twice — what the device reported and what the host computed from the " +
            "path — because they answer different questions and a disagreement between them " +
            "is worth seeing. Zone hits are ruled on the same time base.",
        ),
        this.windowPicker,
        this.failureSlot,
        this.charts,
        this.make("p", {
          class: "note",
          text:
            "A monitor, not the record: this is the decimated browser stream and it keeps " +
            "only what fits on screen. The path of record is the recording, and a trial's " +
            "own span comes from its mark.",
        }),
      ]),
    );
    this.paintWindowButtons();
  }

  start() {
    this.pollEvery(5, async () => {
      const state = await this.api.readState();
      this.offerAxes((state.axes || []).map((axis) => axis.name));
    });
    this.followStream((options) => this.api.followState(BROWSER_RATE_HZ, options), {
      onOpen: () => this.setLink(true),
      onClose: () => this.setLink(false),
      onMessage: (message) => this.absorb(message),
    });
  }

  stopped() {
    this.setLink(false);
  }

  setLink(live) {
    this.live = live;
    this.linkPill.className = live ? "pill good" : "pill bad";
    this.linkPill.textContent = live ? "streaming" : "not streaming";
  }

  offerAxes(names) {
    const current = [...this.axisPicker.options].map((option) => option.value);
    if (current.join("\u0000") === names.join("\u0000")) return;
    this.axisPicker.replaceChildren(
      ...names.map((name) => this.make("option", { value: name, text: name })),
    );
    if (this.axisName === null || !names.includes(this.axisName)) this.axisName = names[0] ?? null;
    if (this.axisName !== null) this.axisPicker.value = this.axisName;
  }

  /// One frame of the stream: a sample, and whatever fired since the last one.
  ///
  /// The daemon names what it lost rather than handing a consumer a shorter
  /// answer that looks complete, so `lost_before` is recorded on the previous
  /// point and becomes a break in the line. See the note at the top on why this
  /// is not read off `seq`.
  absorb(message) {
    const frame = streamFrame(message);
    // An arm this build does not know: a frame from a newer daemon, skipped
    // rather than half-read.
    if (frame === null) return;

    if (frame.kind === "zone_hit") {
      this.events.push({ t: count(frame.host_monotonic_ns) / 1e9, label: frame.zone });
      if (this.events.length > 64) this.events.shift();
      return;
    }
    const axis = (frame.axes || []).find((one) => one.name === this.axisName);
    if (axis === undefined) return;

    const previous = this.points[this.points.length - 1];
    if (previous !== undefined && count(frame.lost_before) > 0) previous.gapAfter = true;
    this.lastSeq = count(frame.seq);
    this.points.push({
      t: count(frame.host_monotonic_ns) / 1e9,
      position_cm: axis.position_cm,
      host_velocity_cm_s: axis.velocity_cm_s,
      device_velocity_cm_s: axis.device_velocity_cm_s ?? axis.velocity_cm_s,
      gapAfter: false,
    });
    if (this.points.length > MOST_POINTS) this.points.splice(0, this.points.length - MOST_POINTS);
    this.paintSoon();
  }

  paintWindowButtons() {
    for (const button of this.windowPicker.querySelectorAll("button")) {
      const selected = Number(button.dataset.seconds) === this.windowS;
      button.className = selected ? "primary" : "";
    }
  }

  paint() {
    const newest = this.points[this.points.length - 1];
    if (newest === undefined) {
      this.charts.replaceChildren(this.make("span", { class: "muted", text: "waiting for the first sample…" }));
      return;
    }
    const from = newest.t - this.windowS;
    const points = this.points.filter((point) => point.t >= from);
    const events = this.events.filter((event) => event.t >= from);
    const shared = { windowS: this.windowS, now: newest.t, width: WIDTH, height: CHART_HEIGHT };

    const position = drawTimeSeries((...args) => this.svg(...args), {
      ...shared,
      points,
      includeZero: false,
      series: [{ name: "position", color: "var(--series-host)", pick: (point) => point.position_cm }],
    });
    position.append(...drawEventRules((...args) => this.svg(...args), { ...shared, events }));

    const velocity = drawTimeSeries((...args) => this.svg(...args), {
      ...shared,
      points,
      includeZero: true,
      series: [
        { name: "host", color: "var(--series-host)", pick: (point) => point.host_velocity_cm_s },
        { name: "device", color: "var(--series-device)", pick: (point) => point.device_velocity_cm_s },
      ],
    });
    velocity.append(...drawEventRules((...args) => this.svg(...args), { ...shared, events }));

    this.charts.replaceChildren(
      this.make("figure", {}, [
        this.make("figcaption", { text: `position — ${newest.position_cm.toFixed(2)} cm` }),
        position,
      ]),
      this.make("figure", {}, [
        this.make("figcaption", {
          text: `velocity — host ${newest.host_velocity_cm_s.toFixed(1)}, device ${newest.device_velocity_cm_s.toFixed(1)} cm/s`,
        }),
        velocity,
      ]),
      this.make("div", { class: "legend" }, [
        this.key("host velocity, from the sample path", "var(--series-host)"),
        this.key("device velocity, as the board reported it", "var(--series-device)"),
        events.length > 0 ? this.make("span", { class: "key", text: `zone hits: ${events.length}` }) : null,
      ]),
    );
  }

  key(label, color) {
    return this.make("span", { class: "key" }, [
      this.make("span", { class: "swatch", style: `background:${color}` }),
      this.make("span", { text: label }),
    ]);
  }
}

defineElementOnce("mousewheeld-trace", TracePanelElement);
