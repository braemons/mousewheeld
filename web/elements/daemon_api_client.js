// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The one place in this UI that knows a network exists.
//
// Every element takes a `base` attribute rather than assuming same-origin,
// because the point of the `/elements/` contract is that a console served from
// somewhere else drops `<mousewheeld-trace>` into its own page. An element that
// called `fetch("/api/state")` would work perfectly on the rig's own page and
// silently talk to the console's host everywhere else.
//
// Methods are named after what they ask for rather than after their URLs, so a
// panel reads as what it wants and each route is written down once. The routes
// are `dev/PLAN.md`, *The API*.

/// A refusal from the daemon: the status, the machine-readable code, the
/// sentence, and the context that names what to change.
///
/// `context` is why this is a class rather than a thrown string. A zone set
/// that does not fit names what overflowed; a UI that shows only "409" throws
/// exactly that away.
export class DaemonRefusedTheRequest extends Error {
  constructor(status, body) {
    const detail = (body && body.detail) || `HTTP ${status}`;
    super(detail);
    this.name = "DaemonRefusedTheRequest";
    this.status = status;
    this.code = (body && body.error) || "http_error";
    this.detail = detail;
    this.context = (body && body.context) || "";
  }
}

export class DaemonApiClient {
  constructor(baseUrl) {
    this.baseUrl = (baseUrl || "").replace(/\/+$/, "");
  }

  urlFor(path) {
    return `${this.baseUrl}${path}`;
  }

  /// The WebSocket origin, derived from `base` and falling back to this page's.
  ///
  /// A relative `base` is the normal case on the rig's own page, and
  /// `new URL(path, location.href)` is what turns it into something
  /// `new WebSocket()` accepts — it refuses a relative URL outright.
  webSocketUrlFor(path) {
    const absolute = new URL(this.urlFor(path), globalThis.location?.href ?? "http://localhost/");
    absolute.protocol = absolute.protocol === "https:" ? "wss:" : "ws:";
    return absolute.toString();
  }

  async request(method, path, body) {
    const response = await fetch(this.urlFor(path), {
      method,
      headers: body === undefined ? {} : { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    const text = await response.text();
    let parsed = null;
    try {
      parsed = text ? JSON.parse(text) : null;
    } catch {
      parsed = null;
    }
    if (!response.ok) throw new DaemonRefusedTheRequest(response.status, parsed);
    return parsed;
  }

  get(path) {
    return this.request("GET", path);
  }
  post(path, body) {
    return this.request("POST", path, body === undefined ? {} : body);
  }
  put(path, body) {
    return this.request("PUT", path, body);
  }
  patch(path, body) {
    return this.request("PATCH", path, body);
  }

  // ------------------------------------------------------------- device ---

  readDevice() {
    return this.get("/api/device");
  }

  connectToTheDevice() {
    return this.post("/api/device/connect");
  }

  readFirmwareVersions() {
    return this.get("/api/device/firmware");
  }

  readWireLog() {
    return this.get("/api/device/monitor");
  }

  wireStreamUrl() {
    return this.webSocketUrlFor("/api/device/monitor/stream");
  }

  // ------------------------------------------------------ state & stream ---

  readState() {
    return this.get("/api/state");
  }

  /// The decimated state stream. `rateHz` is what the browser asks for, not
  /// what the device samples at — the daemon decimates, and says what it lost.
  stateStreamUrl(rateHz) {
    return this.webSocketUrlFor(`/api/stream?rate_hz=${encodeURIComponent(rateHz)}`);
  }

  /// Moves the API origin. Never the published accumulator — vstimd differences
  /// that every frame, and a value that jumps back jumps the camera back.
  zeroPosition(axes) {
    return this.post("/api/position/zero", axes === undefined ? {} : { axes });
  }

  // -------------------------------------------------------- calibration ---

  readCalibration() {
    return this.get("/api/calibration");
  }

  replaceCalibration(calibration) {
    return this.put("/api/calibration", calibration);
  }

  startMeasuring(axis, knownDistanceCm) {
    return this.post("/api/calibration/measure/start", {
      axis,
      known_distance_cm: knownDistanceCm,
    });
  }

  finishMeasuring() {
    return this.post("/api/calibration/measure/finish");
  }

  applyMeasurement() {
    return this.post("/api/calibration/measure/apply");
  }

  // --------------------------------------------------------------- zones ---

  readZoneSetNames() {
    return this.get("/api/zone-sets");
  }

  readZoneSet(name) {
    return this.get(`/api/zone-sets/${encodeURIComponent(name)}`);
  }

  replaceZoneSet(name, zoneSet) {
    return this.put(`/api/zone-sets/${encodeURIComponent(name)}`, zoneSet);
  }

  /// Compile the set as it is *stored* — "is that one still good after the
  /// calibration changed".
  validateZoneSet(name) {
    return this.post(`/api/zone-sets/${encodeURIComponent(name)}/validate`);
  }

  /// Compile a **draft**: what somebody is typing, before it is saved.
  ///
  /// The body goes through the daemon's own deserializer and compiler, so what
  /// comes back is the refusal the real thing would give — no second, looser
  /// description of a zone set in a browser to disagree with it.
  validateDraft(zoneSet) {
    return this.post("/api/zone-sets/validate", zoneSet);
  }

  /// Where the zone set's JSON Schema lives, for an editor or a checker.
  zoneSetSchemaUrl() {
    return this.urlFor("/api/zone-sets/schema");
  }

  readArmedZones() {
    return this.get("/api/zones");
  }

  armZoneSet(request) {
    return this.post("/api/zones/arm", request);
  }

  disarmZones() {
    return this.post("/api/zones/disarm");
  }

  saveZonesToFlash() {
    return this.post("/api/zones/save");
  }

  readOutputLines() {
    return this.get("/api/lines");
  }

  // -------------------------------------------------------------- config ---

  readConfig() {
    return this.get("/api/config");
  }

  updateConfig(changes) {
    return this.patch("/api/config", changes);
  }
}
