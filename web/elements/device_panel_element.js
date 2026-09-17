// SPDX-License-Identifier: AGPL-3.0-or-later
//
// `<mousewheeld-device>` — what board is attached, and how the link behaves.
//
// The first question anybody asks of this daemon is "is the wheel actually
// connected", and the second is "which firmware is on it". Both belong on one
// panel, with the counters that say whether the answer is still true: a board
// that has quietly stopped sending looks exactly like a board that is fine
// until somebody looks at a sequence number.
//
// The counters are the point of the panel, not decoration:
//
//   * **seq gaps** — samples the daemon never received. A path with a hole in
//     it must say so rather than report a shorter distance that looks complete.
//   * **ring drops** — the device's own rings overflowed, which is the firmware
//     telling on itself.
//   * **stream rate against display rate** — a wheel published at or below the
//     display rate makes a camera that stutters at exactly a running animal's
//     speeds, and this is where that is visible before an experiment runs.

import { BasePanelElement, defineElementOnce } from "./base_panel_element.js";

const POLL_SECONDS = 1;

export class DevicePanelElement extends BasePanelElement {
  renderShell() {
    this.failureSlot = this.make("div", { class: "failure-slot" });
    this.body = this.make("div", { text: "reading the device…" });
    this.connectButton = this.make("button", {
      text: "connect",
      onClick: () => this.attempt(() => this.api.connectToTheDevice()),
    });
    this.zeroButton = this.make("button", {
      text: "zero position",
      title:
        "Moves the API origin, for zones and for what this UI shows. Never the " +
        "accumulator published to a camera — that one is differenced every frame, " +
        "and a value that jumps backwards jumps the corridor backwards.",
      onClick: () => this.attempt(() => this.api.zeroPosition()),
    });
    this.root.replaceChildren(
      this.make("section", {}, [
        this.make("h2", {}, [
          this.make("span", { text: "Device" }),
          this.make("span", { class: "spacer" }),
          this.zeroButton,
          this.connectButton,
        ]),
        this.describe(
          "The board this daemon owns: what it is, what firmware it runs, and whether the " +
            "link is still delivering. A board that stopped sending looks exactly like one " +
            "that is fine, which is why the gaps and drops are counted here.",
        ),
        this.failureSlot,
        this.body,
      ]),
    );
  }

  start() {
    this.pollEvery(POLL_SECONDS, async () => {
      const [device, state, config] = await Promise.all([
        this.api.readDevice(),
        this.api.readState(),
        this.api.readConfig(),
      ]);
      this.paint(device, state, config);
    });
  }

  paint(device, state, config) {
    const link = device.link || {};
    const capacities = device.capacities || {};
    const health = state.health || {};
    const connected = Boolean(device.connected);

    this.connectButton.disabled = connected;
    this.connectButton.textContent = connected ? "connected" : "connect";
    this.zeroButton.disabled = !connected;

    this.body.replaceChildren(
      this.fieldList([
        [
          "link",
          this.make("span", {
            class: connected ? "pill good" : "pill bad",
            text: connected ? "connected" : "no device",
          }),
        ],
        ["port", this.make("span", { class: "mono", text: device.port || "—" })],
        ["board", device.board || "—"],
        ["firmware", this.make("span", { class: "mono", text: device.firmware_version || "—" })],
        ["protocol", device.protocol_version ?? "—"],
        ["axes", this.axesSummary(device, state)],
        ["stream", this.streamSummary(config, health)],
        ["seq gaps", this.counter(health.seq_gaps, "samples the daemon never received")],
        ["ring drops", this.counter(health.ring_drops, "the device's own rings overflowed")],
        ["reconnects", link.connection_count ?? 0],
        ["up", formatDeviceMicroseconds(device.uptime_device_us)],
        ["capacity", this.capacitySummary(capacities)],
        ["zone set on flash", device.flashed_zone_set ? `${device.flashed_zone_set.name} v${device.flashed_zone_set.version}` : "none"],
      ]),
    );
    if (link.last_error) {
      this.body.append(this.make("p", { class: "warn", text: `last refusal from the device: ${link.last_error}` }));
    }
  }

  /// One line per axis: what it is counting and what that is in centimetres.
  axesSummary(device, state) {
    const rows = this.make("div");
    for (const axis of state.axes || []) {
      rows.append(
        this.make("div", { class: "mono" }, [
          this.make("span", { text: `${axis.name}  ` }),
          this.make("span", { text: `${axis.counts} counts  ` }),
          this.make("span", { text: `${(axis.position_cm ?? 0).toFixed(2)} cm  ` }),
          this.make("span", { class: "muted", text: `${(axis.velocity_cm_s ?? 0).toFixed(1)} cm/s` }),
        ]),
      );
    }
    if ((state.axes || []).length === 0) rows.append(this.make("span", { text: "—" }));
    return rows;
  }

  /// The rate the samples arrive at, and whether it clears the display it
  /// drives. Below the display rate is a finding, not a statistic.
  streamSummary(config, health) {
    const rateHz = config.rate_hz ?? 0;
    const displayHz = config.display_hz ?? 0;
    const tooSlow = displayHz > 0 && rateHz <= displayHz;
    return this.make("span", {
      class: tooSlow ? "bad" : "",
      title: tooSlow
        ? "At or below the display rate, some frames see no new sample and the next " +
          "sees two: the corridor moves in uneven steps, at exactly the speeds a " +
          "running animal produces. Raise rate_hz above display_hz."
        : "",
      text:
        `${rateHz} Hz` +
        (displayHz ? ` against a ${displayHz} Hz display` : "") +
        (health.measured_rate_hz ? `  (measured ${health.measured_rate_hz.toFixed(0)} Hz)` : ""),
    });
  }

  counter(value, why) {
    const count = value ?? 0;
    return this.make("span", { class: count > 0 ? "bad" : "", title: why, text: `${count}` });
  }

  capacitySummary(capacities) {
    const text = Object.entries(capacities)
      .map(([name, value]) => `${name}=${value}`)
      .join("  ");
    return this.make("span", { class: "mono muted", text: text || "—" });
  }
}

export function formatDeviceMicroseconds(microseconds) {
  if (typeof microseconds !== "number") return "—";
  const seconds = microseconds / 1e6;
  if (seconds < 90) return `${seconds.toFixed(1)} s`;
  const minutes = seconds / 60;
  if (minutes < 90) return `${minutes.toFixed(1)} min`;
  return `${(minutes / 60).toFixed(1)} h`;
}

defineElementOnce("mousewheeld-device", DevicePanelElement);
