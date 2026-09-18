// SPDX-License-Identifier: AGPL-3.0-or-later
//
// `<mousewheeld-calibration>` — counts per centimetre, and how it was arrived at.
//
// This daemon owns the calibration, which is why it is a panel and not a line
// in somebody's config file. The wheel's counts-per-cm is a fact about the
// hardware that everything downstream needs in the same units: the corridor
// vstimd draws, the zones the firmware compares against, the distance a trial
// records. One copy, here, and centimetres on every wire out.
//
// **The guided measurement is the point of the panel.** Arithmetic from the
// datasheet — `counts_per_rev / (π · diameter_cm)` — is the number you check a
// measurement against, not the number to use: what a corridor position is made
// of is how far the *feet* travelled, on the surface the animal runs on, with
// whatever tread is on it, and that is a few percent off the nominal every
// time. So: name a distance, roll the wheel it, and read what the counts say.
//
// A measurement is never applied silently. `finish` reports measured against
// configured and `apply` is a separate press, because a calibration change
// invalidates every compiled zone set and is recorded as an event.

import { BasePanelElement, defineElementOnce } from "./base_panel_element.js";
import { count } from "./wire_shapes.js";

const POLL_SECONDS = 1;
/// Ratios that mean a decoder, not a wheel: quadrature counted ×1 or ×2 where
/// the counts-per-rev assumed ×4. It looks exactly like a wheel of the wrong
/// size, and it is the mistake this panel exists to catch.
const DECODING_RATIOS = [4, 2, 0.5, 0.25];

export class CalibrationPanelElement extends BasePanelElement {
  constructor() {
    super();
    this.measuring = null; // { axis, known_distance_cm }
    this.result = null;
  }

  renderShell() {
    this.failureSlot = this.make("div", { class: "failure-slot" });
    this.axisFields = this.make("div");
    this.axisPicker = this.make("select");
    this.distanceField = this.make("input", { type: "number", value: "100", min: "1", step: "1" });
    this.startButton = this.make("button", { class: "primary", text: "start", onClick: () => this.startMeasuring() });
    this.finishButton = this.make("button", { text: "finish", disabled: true, onClick: () => this.finishMeasuring() });
    this.applyButton = this.make("button", { text: "apply", disabled: true, onClick: () => this.applyMeasurement() });
    this.liveCounts = this.make("span", { class: "mono muted" });
    this.verdict = this.make("div");

    this.root.replaceChildren(
      this.make("section", {}, [
        this.make("h2", {}, [this.make("span", { text: "Calibration" })]),
        this.describe(
          "Counts per centimetre, per axis — the one copy on the rig, since everything " +
            "downstream is told centimetres. Measure it against the surface the animal runs " +
            "on; the diameter arithmetic is what you check the measurement against.",
        ),
        this.failureSlot,
        this.axisFields,
        this.make("h3", { text: "measure" }),
        this.make("p", {
          class: "note",
          text:
            "Roll the wheel the distance you name, by hand, along its running surface, in the " +
            "direction the animal runs — a metre in one continuous motion. A metre averages " +
            "away where you started and stopped; one revolution does not.",
        }),
        this.make("div", { class: "row" }, [
          this.axisPicker,
          this.distanceField,
          this.make("span", { class: "muted", text: "cm" }),
          this.startButton,
          this.finishButton,
          this.liveCounts,
        ]),
        this.verdict,
        this.make("h3", { text: "the ball" }),
        this.make("p", {
          class: "note",
          text:
            "Two-dimensional ball calibration — mounting angle, ball diameter, the transform " +
            "onto x, y and yaw — answers 501 until the hardware exists. The model is there so " +
            "that adding it is filling in rather than redesigning.",
        }),
      ]),
    );
  }

  start() {
    this.pollEvery(POLL_SECONDS, async () => {
      const [calibration, state] = await Promise.all([this.api.readCalibration(), this.api.readState()]);
      this.paint(calibration, state);
    });
  }

  paint(calibration, state) {
    const axes = calibration.axes || [];
    this.offerAxes(axes.map((axis) => axis.name));

    this.axisFields.replaceChildren(
      ...axes.map((axis) =>
        this.make("div", {}, [
          this.make("h3", { text: axis.name }),
          this.fieldList([
            ["counts per cm", this.editableNumber(axis, "counts_per_cm", 3)],
            ["counts per rev", this.editableNumber(axis, "counts_per_rev", 0)],
            ["diameter cm", this.editableNumber(axis, "diameter_cm", 2)],
            ["nominal", this.nominalFor(axis)],
            [
              "invert",
              this.make("input", {
                type: "checkbox",
                checked: Boolean(axis.invert),
                onChange: (event) => this.replace(axis.name, { invert: event.target.checked }),
              }),
            ],
            ["measured", axis.measured_at ? `${axis.measured_at}` : this.make("span", { class: "warn", text: "never — this is arithmetic, not a measurement" })],
          ]),
        ]),
      ),
    );

    if (this.measuring !== null) {
      const axis = (state.axes || []).find((one) => one.name === this.measuring.axis);
      const counts = axis ? Math.abs(count(axis.counts) - this.measuring.counts_at_start) : 0;
      this.liveCounts.textContent = `${counts} counts`;
    }
  }

  offerAxes(names) {
    const current = [...this.axisPicker.options].map((option) => option.value);
    if (current.join("\u0000") === names.join("\u0000")) return;
    this.axisPicker.replaceChildren(...names.map((name) => this.make("option", { value: name, text: name })));
  }

  /// A number that writes itself back on blur rather than on every keystroke:
  /// this panel polls at 1 Hz and would otherwise fight a person mid-number.
  editableNumber(axis, field, decimals) {
    const value = axis[field];
    return this.make("input", {
      type: "number",
      step: decimals === 0 ? "1" : `0.${"0".repeat(decimals - 1)}1`,
      value: typeof value === "number" ? value.toFixed(decimals) : "",
      onChange: (event) => this.replace(axis.name, { [field]: Number(event.target.value) }),
    });
  }

  /// What the datasheet says, beside what the wheel does.
  nominalFor(axis) {
    if (!axis.counts_per_rev || !axis.diameter_cm) return "—";
    const nominal = axis.counts_per_rev / (Math.PI * axis.diameter_cm);
    const drift = axis.counts_per_cm ? (100 * (axis.counts_per_cm - nominal)) / nominal : 0;
    return this.make("span", {
      class: Math.abs(drift) > 10 ? "warn" : "muted",
      text: `${nominal.toFixed(3)} counts/cm from the diameter (${drift >= 0 ? "+" : ""}${drift.toFixed(1)} % in force)`,
    });
  }

  replace(axisName, changes) {
    return this.attempt(() => this.api.replaceCalibration({ axes: [{ name: axisName, ...changes }] }));
  }

  async startMeasuring() {
    const axisName = this.axisPicker.value;
    const knownDistanceCm = Number(this.distanceField.value);
    const started = await this.attempt(() => this.api.startMeasuring(axisName, knownDistanceCm));
    if (started === null) return;
    this.measuring = { axis: axisName, known_distance_cm: knownDistanceCm, counts_at_start: count(started.counts) };
    this.result = null;
    this.startButton.disabled = true;
    this.finishButton.disabled = false;
    this.applyButton.disabled = true;
    this.verdict.replaceChildren(
      this.make("p", { class: "muted", text: `counting on ${axisName} — roll ${knownDistanceCm} cm, then press finish.` }),
    );
  }

  async finishMeasuring() {
    const result = await this.attempt(() => this.api.finishMeasuring());
    this.startButton.disabled = false;
    this.finishButton.disabled = true;
    this.liveCounts.textContent = "";
    if (result === null) return;
    this.result = result;
    this.measuring = null;
    this.applyButton.disabled = false;
    this.paintVerdict(result);
  }

  paintVerdict(result) {
    const measured = result.measured_counts_per_cm;
    const configured = result.configured_counts_per_cm;
    const drift = configured ? (100 * (measured - configured)) / configured : 0;
    const ratio = configured ? measured / configured : 1;
    const decodingSuspect = DECODING_RATIOS.find((factor) => Math.abs(ratio - factor) < 0.05);

    const rows = [
      ["rolled", `${result.known_distance_cm} cm`],
      ["counted", `${count(result.counts)} counts`],
      ["measured", this.make("span", { class: "mono", text: `${measured.toFixed(3)} counts/cm` })],
      ["configured", this.make("span", { class: "mono muted", text: `${configured.toFixed(3)} counts/cm` })],
      [
        "difference",
        this.make("span", {
          class: Math.abs(drift) > 10 ? "warn" : "",
          text: `${drift >= 0 ? "+" : ""}${drift.toFixed(1)} %`,
        }),
      ],
      ["implies", `${(result.counts_per_rev ? result.counts_per_rev / measured : 0).toFixed(2)} cm circumference`],
    ];

    this.verdict.replaceChildren(
      this.fieldList(rows),
      decodingSuspect
        ? this.make("p", {
            class: "bad",
            text:
              `That is ${decodingSuspect}× the configured value. A whole-number ratio is a ` +
              `decoding mistake, not a wheel — check the quadrature multiplier before applying it.`,
          })
        : null,
      count(result.counts) < 100
        ? this.make("p", { class: "bad", text: "too few counts to calibrate anything — did the wheel turn?" })
        : null,
      this.make("div", { class: "row" }, [
        this.applyButton,
        this.make("span", {
          class: "muted",
          text: "applying records an event and marks every compiled zone set stale",
        }),
      ]),
    );
  }

  async applyMeasurement() {
    const applied = await this.attempt(() => this.api.applyMeasurement());
    if (applied === null) return;
    this.applyButton.disabled = true;
    this.verdict.replaceChildren(
      this.make("p", { class: "good", text: `applied: ${applied.counts_per_cm.toFixed(3)} counts/cm` }),
    );
  }
}

defineElementOnce("mousewheeld-calibration", CalibrationPanelElement);
