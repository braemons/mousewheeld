// SPDX-License-Identifier: AGPL-3.0-or-later
//
// What every panel has in common: a shadow root, a `base` attribute, a way to
// poll or follow a stream without leaking either, and one honest place for a
// refusal to land.
//
// **Poll by default; open a socket only where the thing is a stream.** A
// browser tab with five panels open would otherwise hold five sockets against a
// daemon whose whole reason for existing is one serial port. Polling a snapshot
// at 1 Hz costs a rig nothing and cannot fall behind. Two panels are streams by
// nature — the trace and the wire monitor — and they say so.

import { DaemonApiClient, DaemonRefusedTheRequest } from "./daemon_api_client.js";
import { adoptSharedStyles } from "./shared_panel_stylesheet.js";

export class BasePanelElement extends HTMLElement {
  static observedAttributes = ["base"];

  constructor() {
    super();
    this.root = this.attachShadow({ mode: "open" });
    adoptSharedStyles(this.root);
    this.pollTimers = [];
    this.openSockets = [];
    this.reconnectTimers = [];
    this.failure = null;
    this.failureIsFromPoll = false;
  }

  get api() {
    return new DaemonApiClient(this.getAttribute("base") || "");
  }

  connectedCallback() {
    this.renderShell();
    this.shellRendered = true;
    this.start();
  }

  disconnectedCallback() {
    // A panel removed from the DOM must stop talking to the rig. Not tidiness:
    // a console that swaps panels on every nav click would otherwise accumulate
    // pollers and sockets against a daemon holding one serial port.
    this.stop();
  }

  attributeChangedCallback(name, previous, current) {
    // An element upgraded after the page parsed gets its attributes *before*
    // `connectedCallback`, so this fires with no shell to stop or start. That
    // path is not a restart: `connectedCallback` is about to run with the new
    // value anyway.
    if (previous !== current && this.isConnected && this.shellRendered) {
      this.stop();
      this.start();
    }
  }

  /// Subclasses override these three.
  renderShell() {}
  start() {}
  stopped() {}

  stop() {
    for (const timer of this.pollTimers) clearInterval(timer);
    for (const timer of this.reconnectTimers) clearTimeout(timer);
    this.pollTimers = [];
    this.reconnectTimers = [];
    for (const socket of this.openSockets) {
      socket.onclose = null; // this close is ours, and must not trigger a retry
      try {
        socket.close();
      } catch {
        /* already closing */
      }
    }
    this.openSockets = [];
    this.stopped();
  }

  /// Call `read` now and every `seconds`, and never let two overlap.
  ///
  /// The overlap guard matters: `GET /api/device` reaches the device session,
  /// and a panel that fired a second read before the first returned would queue
  /// work behind a board that is answering at its own pace.
  pollEvery(seconds, read) {
    let inFlight = false;
    const once = async () => {
      if (inFlight) return;
      inFlight = true;
      try {
        await read();
        this.clearPollFailure();
      } catch (error) {
        this.showFailure(error, { fromPoll: true });
      } finally {
        inFlight = false;
      }
    };
    once();
    this.pollTimers.push(setInterval(once, seconds * 1000));
    return once;
  }

  /// Follow a WebSocket, reconnecting until this panel is taken off the page.
  ///
  /// A console is left open across the thing it is watching — a rig restarted
  /// between blocks, a daemon upgraded, a cable. A panel that needs a page
  /// reload to notice its daemon came back is a panel somebody stops trusting.
  /// Backoff caps at ten seconds, so a rig that was off all night is picked up
  /// within ten seconds of coming up rather than hammered all night.
  followStream(url, { onMessage, onOpen, onClose }) {
    let attempt = 0;
    const open = () => {
      let socket;
      try {
        socket = new WebSocket(url);
      } catch (error) {
        this.showFailure(error);
        return;
      }
      this.openSockets.push(socket);
      socket.onopen = () => {
        attempt = 0;
        this.clearFailure();
        onOpen?.();
      };
      socket.onmessage = (event) => {
        try {
          onMessage(JSON.parse(event.data));
        } catch (error) {
          this.showFailure(error);
        }
      };
      socket.onclose = () => {
        this.openSockets = this.openSockets.filter((tracked) => tracked !== socket);
        onClose?.();
        const wait = Math.min(1000 * 2 ** attempt, 10_000);
        attempt += 1;
        this.reconnectTimers.push(setTimeout(open, wait));
      };
    };
    open();
  }

  /// Run one action, showing whatever it refuses with.
  async attempt(action) {
    try {
      const result = await action();
      this.clearFailure();
      return result;
    } catch (error) {
      this.showFailure(error);
      return null;
    }
  }

  // ------------------------------------------------------------ failures ---
  //
  // A refusal has to outlive the next poll. Every panel here polls once a
  // second, so a banner cleared by the next successful read is on screen for
  // under a second — and the press that earned it is exactly the moment a
  // person looks away to the thing they were changing. So a poll clears only
  // what a poll put there; what an action was refused with stays until the
  // person tries something else.

  showFailure(error, { fromPoll = false } = {}) {
    this.failure = error;
    this.failureIsFromPoll = fromPoll;
    this.paintFailure();
  }

  clearFailure() {
    if (this.failure !== null) {
      this.failure = null;
      this.failureIsFromPoll = false;
      this.paintFailure();
    }
  }

  /// A poll succeeded: clear a banner a poll raised, and leave a refusal alone.
  clearPollFailure() {
    if (this.failure !== null && this.failureIsFromPoll) this.clearFailure();
  }

  paintFailure() {
    const slot = this.root.querySelector(".failure-slot");
    if (slot === null) return;
    if (this.failure === null) {
      slot.replaceChildren();
      return;
    }
    const error = this.failure;
    const banner = this.make("div", { class: "failure" });
    if (error instanceof DaemonRefusedTheRequest) {
      // Every refusal names what to change, so show it: a UI that renders only
      // the status code throws away the useful half.
      banner.textContent = `${error.code}: ${error.detail}`;
      if (error.context) banner.append(this.make("span", { class: "context", text: `  (${error.context})` }));
    } else {
      banner.textContent = `${error}`;
    }
    slot.replaceChildren(banner);
  }

  // ------------------------------------------------------------- markup ---
  //
  // Built rather than templated. There is no build step and no framework here,
  // so the alternative is string concatenation into innerHTML — which is how a
  // zone named `<script>` becomes an execution.

  make(tag, properties = {}, children = []) {
    const node = document.createElement(tag);
    for (const [key, value] of Object.entries(properties)) {
      if (key === "class") node.className = value;
      else if (key === "text") node.textContent = value;
      else if (key.startsWith("on") && typeof value === "function") {
        node.addEventListener(key.slice(2).toLowerCase(), value);
      } else if (key === "dataset") Object.assign(node.dataset, value);
      else if (key in node) node[key] = value;
      else node.setAttribute(key, value);
    }
    node.append(...children.filter((child) => child !== null && child !== undefined));
    return node;
  }

  /// An SVG node. Same shape as `make`, and separate because SVG children are
  /// in their own namespace and `createElement` silently makes the wrong thing.
  svg(tag, properties = {}, children = []) {
    const node = document.createElementNS("http://www.w3.org/2000/svg", tag);
    for (const [key, value] of Object.entries(properties)) {
      if (key === "text") node.textContent = value;
      else if (key.startsWith("on") && typeof value === "function") {
        node.addEventListener(key.slice(2).toLowerCase(), value);
      } else node.setAttribute(key, `${value}`);
    }
    node.append(...children.filter((child) => child !== null && child !== undefined));
    return node;
  }

  /// A definition list of label/value pairs — the shape most of this UI is.
  fieldList(pairs) {
    const list = this.make("dl", { class: "fields" });
    for (const [label, value] of pairs) {
      list.append(
        this.make("dt", { text: label }),
        value instanceof Node ? this.make("dd", {}, [value]) : this.make("dd", { text: `${value}` }),
      );
    }
    return list;
  }

  /// The one sentence a panel owes a reader who has never seen it.
  describe(text) {
    return this.make("p", { class: "muted", text });
  }
}

/// Registering twice is not an error worth throwing over: a console may load
/// this module and one of its panels' modules, and the second registration
/// would take down the page it was meant to draw.
export function defineElementOnce(tagName, elementClass) {
  if (!customElements.get(tagName)) customElements.define(tagName, elementClass);
}

export { DaemonRefusedTheRequest };
