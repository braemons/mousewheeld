// SPDX-License-Identifier: AGPL-3.0-or-later
//
// `<mousewheeld-monitor>` — the wire itself, both directions, as it goes.
//
// The panel for the moment the layers stop agreeing. The Zones panel says the
// goal is at 200 cm; the line never fires; what actually went down the wire?
// Nothing here interprets anything: these are the lines the daemon wrote and
// the lines the board answered, in order, with the time they crossed.
//
// It reads the daemon's ring first and then follows the stream, so opening it
// shows the greeting that happened before anybody clicked — a monitor that
// started at "now" would miss every fault that had already happened, which is
// most of them.
//
// **`sample` lines are hidden by default**, and that is not tidiness. The
// stream runs at hundreds of hertz; unfiltered, the one refusal a person is
// looking for scrolls past in a tenth of a second. The toggle is right there,
// and the count of what is hidden with it.
//
// **It sends nothing.** A terminal that could type at the board would be a
// second host on a link whose commands all have API routes that keep the
// daemon's idea of the device true — and every one of those routes shows up
// here anyway.

import { BasePanelElement, defineElementOnce } from "./base_panel_element.js";

/// What the browser holds. The daemon's ring is larger; this is what a table
/// can be scrolled through without the tab becoming the slow part of the rig.
const MOST_LINES_ON_SCREEN = 2000;

export class SerialMonitorPanelElement extends BasePanelElement {
  constructor() {
    super();
    this.lines = [];
    this.hiddenSamples = 0;
    this.showsSamples = false;
    this.isFollowing = true;
    this.textFilter = "";
  }

  renderShell() {
    this.failureSlot = this.make("div", { class: "failure-slot" });
    this.rows = this.make("tbody");
    this.linkPill = this.make("span", { class: "pill warn", text: "connecting…" });
    this.countLabel = this.make("span", { class: "muted" });
    this.scroller = this.make("div", { class: "scroller", style: "max-height:22rem" }, [
      this.make("table", {}, [
        this.make("thead", {}, [
          this.make("tr", {}, [
            this.make("th", { text: "host time" }),
            this.make("th", { text: "" }),
            this.make("th", { text: "line" }),
          ]),
        ]),
        this.rows,
      ]),
    ]);
    // Following means "stay at the bottom". Scrolling up is how a person says
    // they are reading something, so it stops following until they come back.
    this.scroller.addEventListener("scroll", () => {
      const distanceFromBottom =
        this.scroller.scrollHeight - this.scroller.scrollTop - this.scroller.clientHeight;
      this.isFollowing = distanceFromBottom < 24;
    });

    this.root.replaceChildren(
      this.make("section", {}, [
        this.make("h2", {}, [
          this.make("span", { text: "Wire" }),
          this.make("span", { class: "spacer" }),
          this.linkPill,
        ]),
        this.describe(
          "Every line between the daemon and the board, in order, uninterpreted. This is a " +
            "log, not the record: it is thrown away when the ring wraps.",
        ),
        this.make("div", { class: "row" }, [
          this.make("label", {}, [
            this.make("input", {
              type: "checkbox",
              checked: this.showsSamples,
              onChange: (event) => {
                this.showsSamples = event.target.checked;
                this.paint();
              },
            }),
            this.make("span", { text: " show samples" }),
          ]),
          this.make("input", {
            type: "search",
            placeholder: "filter",
            onInput: (event) => {
              this.textFilter = event.target.value.toLowerCase();
              this.paint();
            },
          }),
          this.make("button", {
            text: "clear",
            onClick: () => {
              this.lines = [];
              this.hiddenSamples = 0;
              this.paint();
            },
          }),
          this.countLabel,
        ]),
        this.failureSlot,
        this.scroller,
      ]),
    );
  }

  start() {
    this.attempt(async () => {
      const backfill = await this.api.readWireLog();
      this.lines = backfill.lines || [];
      this.paint();
    });
    this.followStream(this.api.wireStreamUrl(), {
      onOpen: () => this.setLink(true),
      onClose: () => this.setLink(false),
      onMessage: (line) => this.absorb(line),
    });
  }

  stopped() {
    this.setLink(false);
  }

  setLink(live) {
    this.linkPill.className = live ? "pill good" : "pill bad";
    this.linkPill.textContent = live ? "following" : "not following";
  }

  absorb(line) {
    this.lines.push(line);
    if (this.lines.length > MOST_LINES_ON_SCREEN) {
      this.lines.splice(0, this.lines.length - MOST_LINES_ON_SCREEN);
    }
    this.paint();
  }

  visibleLines() {
    let hidden = 0;
    const visible = [];
    for (const line of this.lines) {
      const isSample = /"sample"|^DATA:/.test(line.text || "");
      if (isSample && !this.showsSamples) {
        hidden += 1;
        continue;
      }
      if (this.textFilter && !(line.text || "").toLowerCase().includes(this.textFilter)) continue;
      visible.push(line);
    }
    this.hiddenSamples = hidden;
    return visible;
  }

  paint() {
    const visible = this.visibleLines();
    this.rows.replaceChildren(
      ...visible.map((line) =>
        this.make("tr", {}, [
          this.make("td", { class: "mono muted", text: formatHostTime(line.host_monotonic_ns) }),
          this.make("td", {
            class: line.direction === "out" ? "mono" : "mono muted",
            title: line.direction === "out" ? "daemon → board" : "board → daemon",
            text: line.direction === "out" ? "→" : "←",
          }),
          this.make("td", { class: line.level === "error" ? "mono bad" : "mono", text: line.text }),
        ]),
      ),
    );
    this.countLabel.textContent =
      `${visible.length} lines` + (this.hiddenSamples > 0 ? `, ${this.hiddenSamples} samples hidden` : "");
    if (this.isFollowing) this.scroller.scrollTop = this.scroller.scrollHeight;
  }
}

/// Host monotonic nanoseconds as seconds, which is what a person comparing two
/// lines a few milliseconds apart actually needs.
export function formatHostTime(nanoseconds) {
  if (typeof nanoseconds !== "number") return "—";
  return (nanoseconds / 1e9).toFixed(3);
}

defineElementOnce("mousewheeld-monitor", SerialMonitorPanelElement);
