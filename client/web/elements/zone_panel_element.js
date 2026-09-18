// SPDX-License-Identifier: AGPL-3.0-or-later
//
// `<mousewheeld-zones>` — the zone-set store, a track diagram, and arming.
//
// A zone is a distance at which a TTL line goes high, decided on the device in
// the scan that sees the count. That is why the diagram matters more than the
// JSON above it: a zone set is a handful of numbers in centimetres, and the
// question a person actually has — *is the reward region where I think it is,
// and is the animal inside it right now* — is a picture, not a list.
//
// The track diagram is one axis drawn to scale, with each zone as a band and
// the live position as a marker on it. A zone that has fired is filled; one
// that is armed and waiting is outlined; one the set defines but nothing armed
// is greyed. `wrap_cm` draws the period rather than the raw distance, because
// on a circular track that is the position the zone is actually compared to.
//
// **An armed zone is drawn from what is armed, not from what is stored.** A
// parameterised set says `"min_cm": [{"reference": "goal_cm"}]`, and it has no
// number in it at all: drawn from the store, a set armed at 180 cm appears at
// zero, which is a picture of a zone that is not running. `GET /api/zones`
// carries each armed zone's resolved bounds for exactly this, and the store's
// own view labels an unresolved bound `$name` rather than pretending it is a
// distance.
//
// **Editing is the JSON, deliberately.** A form reproducing the zone set here
// would be a second, worse copy of its schema, drifting from the first time a
// field is added.
//
// **It is the API's JSON, not the file's, and the two differ on purpose.** What
// this editor holds is what `GET /api/zone-sets/{name}` answered: bounds as
// `{"value": 30}` / `{"reference": "goal_cm"}` / `{}`, enums spelled
// `ZONE_SHAPE_RECT`. The file in the store keeps the shorter spelling a person
// writes by hand — `30`, `"$goal_cm"`, `null`, `"rect"` — and the daemon
// converts between them. So **the text of a file cannot be pasted in here**,
// and what is typed here is not what lands on disk. That is the price of the
// interface being one typed thing (`proto/mousewheeld/v1/zones.proto`) while
// the file stays pleasant to edit, and it is the right way round: the shape a
// client must get exactly right is the one a generator produces.
//
// What the editor owes instead is **telling you as you type**. Every edit is
// compiled by the daemon — the real deserializer, the real compiler, against
// the calibration in force — and the verdict appears under the field. Nothing
// is saved to do it: validating used to mean storing the draft first, which put
// a set nobody had approved into the store on the way to finding out it was
// wrong.
//
// The check is the daemon's and not a JSON Schema implementation in here, on
// purpose. A validator in the browser would be a second description of a zone
// set, looser than the one that matters, and looser is worse than absent — it
// is believed. The schema *is* published, at `/api/zone-sets/schema`, for the
// two places that cannot call the daemon: a text editor with the file open, and
// CI.

import { BasePanelElement, defineElementOnce } from "./base_panel_element.js";
import { bound, metricName } from "./wire_shapes.js";

const POLL_SECONDS = 1;
const WIDTH = 460;
const TRACK_HEIGHT = 92;
const PAD_LEFT = 12;
const PAD_RIGHT = 12;

export class ZonePanelElement extends BasePanelElement {
  constructor() {
    super();
    this.selectedName = null;
    this.draft = null; // the zone set as text, while somebody is editing it
    this.lastState = null;
    this.lastArmed = null;
  }

  renderShell() {
    this.failureSlot = this.make("div", { class: "failure-slot" });
    this.namePicker = this.make("select", {
      onChange: (event) => {
        this.selectedName = event.target.value;
        this.draft = null;
        this.loadSelected();
      },
    });
    this.diagram = this.make("div");
    this.editor = this.make("textarea", {
      rows: 12,
      spellcheck: false,
      onInput: (event) => {
        this.draft = event.target.value;
        this.paintDiagram();
        // Debounced: a keystroke is not a question, and a compile per character
        // would ask the daemon one several times a word.
        clearTimeout(this.validateTimer);
        this.validateTimer = setTimeout(() => this.validateDraft(), 400);
      },
    });
    this.verdict = this.make("div", { class: "note" });
    this.status = this.make("span", { class: "muted" });
    this.originPicker = this.make("select", {}, [
      this.make("option", { value: "current", text: "origin: current (a trial)" }),
      this.make("option", { value: "absolute", text: "origin: absolute (the device's)" }),
    ]);
    this.patchField = this.make("input", {
      type: "text",
      placeholder: '{"goal_cm": 180}',
      title:
        "Per-trial values, substituted into any \"$name\" in the set before it is compiled. " +
        "The board never sees a $name.",
      style: "width: 14rem",
    });

    this.root.replaceChildren(
      this.make("section", {}, [
        this.make("h2", {}, [
          this.make("span", { text: "Zones" }),
          this.make("span", { class: "spacer" }),
          this.namePicker,
        ]),
        this.describe(
          "Distances at which a trigger line fires, decided on the device. The store holds " +
            "the sets; arming compiles one against the current calibration and uploads it.",
        ),
        this.failureSlot,
        this.diagram,
        this.make("div", { class: "row" }, [
          this.originPicker,
          this.patchField,
          this.make("button", { class: "primary", text: "arm", onClick: () => this.arm() }),
          this.make("button", { text: "disarm", onClick: () => this.attempt(() => this.api.disarmZones()) }),
          this.make("button", {
            text: "save to flash",
            title: "Write the set on the board, and its armed state, to the board's flash, so a " +
              "standalone rig comes up with its zones. The store on the host stays the source.",
            onClick: () => this.attempt(() => this.api.saveZonesToFlash()),
          }),
          this.status,
        ]),
        this.make("h3", { text: "the set" }),
        this.editor,
        this.verdict,
        this.make("div", { class: "row" }, [
          this.make("button", { text: "save to the store", onClick: () => this.save() }),
          this.make("button", { text: "revert", onClick: () => { this.draft = null; this.loadSelected(); } }),
          this.make("a", {
            href: this.api.zoneSetSchemaUrl(),
            target: "_blank",
            class: "muted",
            style: "font-size:0.8rem",
            text: "file schema",
            title:
              "The JSON Schema of a zone set FILE — the stored spelling, not the one in this " +
              "editor. Put its URL in a file's \"$schema\" line and a text editor validates it " +
              "as you type; point a checker at it in CI. The API's own shape is at /api/proto.",
          }),
        ]),
      ]),
    );
  }

  start() {
    this.pollEvery(POLL_SECONDS, async () => {
      const [names, state, armed] = await Promise.all([
        this.api.readZoneSetNames(),
        this.api.readState(),
        this.api.readArmedZones(),
      ]);
      this.offerNames(names.zone_sets || []);
      this.lastState = state;
      this.lastArmed = armed;
      this.paintArmedStatus();
      this.paintDiagram();
    });
  }

  stopped() {
    clearTimeout(this.validateTimer);
  }

  offerNames(names) {
    const current = [...this.namePicker.options].map((option) => option.value);
    if (current.join("\u0000") !== names.join("\u0000")) {
      this.namePicker.replaceChildren(...names.map((name) => this.make("option", { value: name, text: name })));
    }
    if (this.selectedName === null && names.length > 0) {
      this.selectedName = names[0];
      this.namePicker.value = this.selectedName;
      this.loadSelected();
    }
  }

  async loadSelected() {
    if (this.selectedName === null) return;
    const set = await this.attempt(() => this.api.readZoneSet(this.selectedName));
    if (set === null) return;
    this.paintVerdict("", "");
    // Never overwrite what somebody is typing: the poll runs at 1 Hz and this
    // is the one field on the panel with a cursor in it.
    if (this.draft === null) this.editor.value = JSON.stringify(set, null, 2);
    this.paintDiagram();
  }

  /// The set as it is on screen — the draft if there is one, the daemon's copy
  /// otherwise — so the diagram shows the edit, not the saved version.
  currentSet() {
    try {
      return JSON.parse(this.draft ?? this.editor.value);
    } catch {
      return null;
    }
  }

  paintArmedStatus() {
    const armed = this.lastArmed || {};
    if (!armed.zone_set) {
      this.status.className = "muted";
      this.status.textContent = "nothing armed";
      return;
    }
    const fired = (armed.zones || []).filter((zone) => zone.fired).length;
    this.status.className = "good";
    this.status.textContent = `armed ${armed.zone_set} v${armed.zone_set_version} · ${fired} fired`;
  }

  paintDiagram() {
    const set = this.currentSet();
    if (set === null) {
      this.diagram.replaceChildren(this.make("p", { class: "warn", text: "not valid JSON — the diagram is waiting" }));
      return;
    }
    const zones = Array.isArray(set.zones) ? set.zones : [];
    const armedByName = new Map(((this.lastArmed || {}).zones || []).map((zone) => [zone.name, zone]));
    const axes = (this.lastState || {}).axes || [];

    // One track per axis the set mentions; a 2-D ball would draw two, which is
    // why this loops rather than assuming the wheel.
    const axisNames = [...new Set(zones.flatMap((zone) => zone.axes || []))];
    if (axisNames.length === 0) axisNames.push(axes[0]?.name ?? "wheel");

    this.diagram.replaceChildren(
      ...axisNames.map((axisName) => this.drawTrack(axisName, zones, armedByName, axes)),
    );
  }

  drawTrack(axisName, zones, armedByName, axes) {
    const axis = axes.find((one) => one.name === axisName);
    const mine = zones.filter((zone) => (zone.axes || []).includes(axisName));
    const wrap = mine.map((zone) => zone.wrap_cm).find((value) => typeof value === "number") ?? null;

    // Each zone, with the bounds actually in force: the armed ones where the
    // set is armed, the stored ones otherwise. The scale has to come from these
    // and not from the store, or a set armed at 180 cm by a patch is drawn on a
    // track ten centimetres long.
    const drawn = mine.map((zone) => {
      const index = (zone.axes || []).indexOf(axisName);
      const live = armedByName.get(zone.name);
      return { zone, live, low: boundOf(live, zone, "min_cm", index), high: boundOf(live, zone, "max_cm", index) };
    });

    const bounds = [0];
    for (const { low, high } of drawn) {
      for (const value of [low, high]) {
        const number = boundValue(value);
        if (number !== null) bounds.push(number);
      }
    }
    const position = this.positionOn(axis, mine, wrap);
    if (position !== null) bounds.push(position);
    const span = wrap ?? Math.max(10, Math.max(...bounds) * 1.1);

    const plotWidth = WIDTH - PAD_LEFT - PAD_RIGHT;
    const x = (cm) => PAD_LEFT + Math.max(0, Math.min(1, cm / span)) * plotWidth;
    // Everything the animal is doing goes above the track and everything the
    // zone set declares goes below it. They collided when they shared the space
    // above, and the one moment the marker sits on a zone's name is exactly the
    // moment both matter.
    const trackY = 46;
    const node = this.svg("svg", { width: WIDTH, height: TRACK_HEIGHT, viewBox: `0 0 ${WIDTH} ${TRACK_HEIGHT}` });

    node.append(
      this.svg("line", {
        x1: PAD_LEFT,
        x2: WIDTH - PAD_RIGHT,
        y1: trackY,
        y2: trackY,
        stroke: "var(--chart-grid)",
        "stroke-width": 8,
        "stroke-linecap": "round",
      }),
    );

    for (const { zone, live, low: lowRaw, high: highRaw } of drawn) {
      // A reference has no place on the track: it is drawn as the whole span,
      // faintly, and labelled with the name it is waiting for.
      const unresolved = isReference(lowRaw) || isReference(highRaw);
      const from = unresolved ? 0 : (boundValue(lowRaw) ?? 0);
      const to = unresolved ? span : (boundValue(highRaw) ?? span);
      const fill = live?.fired ? "var(--series-device)" : live ? "var(--series-host)" : "var(--chart-stale)";
      node.append(
        this.svg("rect", {
          x: x(from),
          y: trackY - 9,
          width: Math.max(2, x(to) - x(from)),
          height: 18,
          rx: 3,
          fill,
          "fill-opacity": unresolved ? 0.08 : live?.fired ? 0.85 : live ? 0.3 : 0.18,
          stroke: fill,
          "stroke-width": live ? 2 : 1,
          "stroke-dasharray": unresolved ? "4 3" : "",
        }),
        this.svg("text", {
          x: x(from),
          y: trackY + 24,
          "font-size": 10,
          fill: "var(--panel-heading)",
          text: `${zone.name}${live?.fired ? " · fired" : ""}`,
        }),
        this.svg("text", {
          x: x(from),
          y: trackY + 35,
          "font-size": 9,
          fill: "var(--muted)",
          text:
            `${describeBound(lowRaw, "0")}–${describeBound(highRaw, "∞")} cm` +
            (unresolved ? " (at arm)" : ""),
        }),
      );
    }

    if (position !== null) {
      node.append(
        this.svg("polygon", {
          points: `${x(position)},${trackY - 13} ${x(position) - 5},${trackY - 22} ${x(position) + 5},${trackY - 22}`,
          fill: "var(--panel-heading)",
        }),
        this.svg("line", {
          x1: x(position),
          x2: x(position),
          y1: trackY - 13,
          y2: trackY + 13,
          stroke: "var(--panel-heading)",
          "stroke-width": 2,
        }),
        this.svg("text", {
          // Clamped inside the plot: at either end of the track the label would
          // otherwise hang off the panel, which is where a wrapping corridor
          // spends a good part of its time.
          x: Math.max(PAD_LEFT, Math.min(WIDTH - PAD_RIGHT, x(position))),
          y: trackY - 27,
          "text-anchor": "middle",
          "font-size": 10,
          fill: "var(--panel-heading)",
          text: `${position.toFixed(1)} cm`,
        }),
      );
    }

    node.append(
      this.svg("text", { x: PAD_LEFT, y: TRACK_HEIGHT - 2, "font-size": 9, fill: "var(--muted)", text: "0 cm" }),
      this.svg("text", {
        x: WIDTH - PAD_RIGHT,
        y: TRACK_HEIGHT - 2,
        "text-anchor": "end",
        "font-size": 9,
        fill: "var(--muted)",
        text: `${span.toFixed(0)} cm${wrap ? " (wraps)" : ""}`,
      }),
    );

    return this.make("figure", {}, [
      this.make("figcaption", {
        text: `${axisName}${wrap ? `, modulo ${wrap} cm` : ""}${position === null ? " — no reading" : ""}`,
      }),
      node,
    ]);
  }

  /// Where the marker goes: the metric the zones on this axis are defined on,
  /// wrapped the same way they are. A zone measured on distance compared
  /// against a displacement marker would be a picture of the wrong thing.
  positionOn(axis, zones, wrap) {
    if (axis === undefined) return null;
    const onDistance = zones.some((zone) => metricName(zone.metric) === "distance");
    const value = onDistance ? axis.distance_cm : axis.position_cm;
    if (typeof value !== "number") return null;
    return wrap ? ((value % wrap) + wrap) % wrap : value;
  }

  /// Compile what is in the editor, saving nothing.
  async validateDraft() {
    let parsed;
    try {
      parsed = JSON.parse(this.draft ?? this.editor.value);
    } catch (problem) {
      this.paintVerdict("bad", `${problem.message}`);
      return;
    }
    try {
      const report = await this.api.validateDraft(parsed);
      this.paintVerdict(
        report.ok ? "good" : "bad",
        report.ok
          ? `compiles: ${report.zone_count} zone${report.zone_count === 1 ? "" : "s"} at ` +
              `${report.counts_per_cm.toFixed(2)} counts/cm` +
              (report.references.length
                ? ` · needs ${report.references.map((name) => `$${name}`).join(", ")} at arm`
                : "")
          : report.problem,
      );
    } catch (refusal) {
      // A refused body is the daemon's own message, naming the field: exactly
      // what an editor should show, and better than anything this panel could
      // work out for itself.
      this.paintVerdict("bad", `${refusal.detail ?? refusal}`);
    }
  }

  paintVerdict(kind, text) {
    this.verdict.className = kind === "good" ? "note good" : "note bad";
    this.verdict.textContent = text;
  }

  async save() {
    if (this.selectedName === null) return null;
    let parsed;
    try {
      parsed = JSON.parse(this.editor.value);
    } catch (error) {
      this.showFailure(error);
      return null;
    }
    const saved = await this.attempt(() => this.api.replaceZoneSet(this.selectedName, parsed));
    if (saved !== null) this.draft = null;
    return saved;
  }

  async arm() {
    if (this.selectedName === null) return;
    let patch;
    try {
      patch = this.patchField.value.trim() === "" ? undefined : JSON.parse(this.patchField.value);
    } catch (error) {
      this.showFailure(error);
      return;
    }
    await this.attempt(() =>
      this.api.armZoneSet({
        zone_set: this.selectedName,
        origin: this.originPicker.value,
        patch,
      }),
    );
  }
}

/// A bound waiting for a value the arm patch has not supplied yet.
///
/// On the wire an authored bound is a `oneof`: `{"value": 30}`, or
/// `{"reference": "goal_cm"}`, or `{}` for an open end. An *armed* bound cannot
/// be a reference at all — the daemon resolved it before the board saw it — and
/// says so by being a different type, `ResolvedBound`, which has no reference
/// arm to check.
function isReference(value) {
  return typeof value?.reference === "string";
}

/// The bound to draw: the armed one where there is one, the stored one
/// otherwise. An open bound stays open in both.
function boundOf(live, zone, field, index) {
  const armed = live?.[field]?.[index];
  if (armed !== undefined) return armed;
  return zone[field]?.[index];
}

/// A bound as a number, or `null` for open or still-unresolved.
function boundValue(value) {
  return isReference(value) ? null : bound(value);
}

function describeBound(value, whenOpen) {
  if (isReference(value)) return `$${value.reference}`;
  const number = bound(value);
  return number === null ? whenOpen : number.toFixed(0);
}

defineElementOnce("mousewheeld-zones", ZonePanelElement);
