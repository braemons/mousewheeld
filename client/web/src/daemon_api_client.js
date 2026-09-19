// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The one place in this UI that knows a network exists.
//
// It speaks **gRPC-Web** to the five services in `proto/mousewheeld/v1/`, over
// the same port the panels themselves are served from — `tonic-web` translates
// in the daemon, so there is no proxy to deploy and nothing to configure.
//
// Every element takes a `base` attribute rather than assuming same-origin,
// because the point of the `/elements/` contract is that a console served from
// somewhere else drops `<mousewheeld-trace>` into its own page. An element that
// called the daemon at its own origin would work perfectly on the rig's own
// page and silently talk to the console's host everywhere else.
//
// **A panel never sees a protobuf message.** This is the browser's half of the
// convert seam, and it is the same rule the daemon keeps in `daemon/src/convert/`:
// the generated types stop here. What crosses is protobuf's JSON mapping —
// `fromJson` on the way out, `toJson` on the way back — which is a plain object
// with the field names the proto spells, and which refuses an unknown field in
// a request by name, in the browser, exactly as the daemon would
// (`contracts/INTERACTIONS.md` §11).
//
// Methods are named after what they ask for rather than after their rpcs, so a
// panel reads as what it wants.

import { createClient } from "@connectrpc/connect";
import { ConnectError, Code } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { fromBinary, fromJson, toJson } from "@bufbuild/protobuf";

import { Device } from "../gen/mousewheeld/v1/device_pb.js";
import { StateService } from "../gen/mousewheeld/v1/state_pb.js";
import { Calibration } from "../gen/mousewheeld/v1/calibration_pb.js";
import { Zones } from "../gen/mousewheeld/v1/zones_pb.js";
import { Config } from "../gen/mousewheeld/v1/config_pb.js";
import { ErrorSchema } from "../gen/mousewheeld/v1/error_pb.js";

/// Where the daemon puts the refusal as itself. See `daemon/src/grpc/mod.rs`.
const REFUSAL_METADATA_KEY = "mousewheeld-error-bin";

/// A refusal from the daemon: the status, the machine-readable code, the
/// sentence, and the context that names what to change.
///
/// `context` is why this is a class rather than a thrown string. A zone set
/// that does not fit names what overflowed; a UI that shows only
/// `INVALID_ARGUMENT` throws exactly that away.
///
/// The three fields come from `mousewheeld.v1.Error` in the status metadata,
/// not from parsing the message: a client that reads a sentence to find out
/// which refusal it was is a client that breaks when the sentence is reworded.
export class DaemonRefusedTheRequest extends Error {
  constructor(status, body) {
    super(body.detail || status);
    this.name = "DaemonRefusedTheRequest";
    this.status = status;
    this.code = body.error || "rpc_failed";
    this.detail = body.detail || status;
    this.context = body.context || "";
  }

  /// Whatever the transport threw, as a refusal.
  ///
  /// A daemon that is not running, a CORS rejection and a cancelled stream all
  /// arrive here too. They have no `mousewheeld.v1.Error` — nothing refused
  /// anything, the call never landed — so the code is the gRPC one and the
  /// detail is what the browser said.
  static from(thrown) {
    if (thrown instanceof DaemonRefusedTheRequest) return thrown;
    const failure = ConnectError.from(thrown);
    // gRPC spells its codes `not_found`; the generated enum spells them
    // `NotFound`. Show the one the daemon's own logs and grpcurl show.
    const status = (Code[failure.code] ?? "Unknown").replace(/(?<=[a-z])(?=[A-Z])/g, "_").toLowerCase();
    return new DaemonRefusedTheRequest(status, refusalIn(failure) ?? { detail: failure.rawMessage });
  }
}

/// `mousewheeld.v1.Error` out of the trailing metadata, or `null`.
///
/// gRPC-Web carries a `-bin` metadata value base64-encoded, and without
/// padding, which `atob` will not take.
function refusalIn(failure) {
  const encoded = failure.metadata?.get(REFUSAL_METADATA_KEY);
  if (!encoded) return null;
  try {
    const padded = encoded + "=".repeat((4 - (encoded.length % 4)) % 4);
    const binary = atob(padded.replace(/-/g, "+").replace(/_/g, "/"));
    const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
    return toJson(ErrorSchema, fromBinary(ErrorSchema, bytes), { alwaysEmitImplicit: true });
  } catch {
    return null; // a refusal we cannot read is still a refusal; keep the sentence
  }
}

export class DaemonApiClient {
  constructor(baseUrl) {
    this.baseUrl = (baseUrl || "").replace(/\/+$/, "");
    const transport = createGrpcWebTransport({ baseUrl: this.baseUrl || "/" });
    this.device = createClient(Device, transport);
    this.state = createClient(StateService, transport);
    this.calibration = createClient(Calibration, transport);
    this.zones = createClient(Zones, transport);
    this.config = createClient(Config, transport);
  }

  /// One unary call: JSON in, JSON out, refusals as `DaemonRefusedTheRequest`.
  ///
  /// The descriptors come off the service rather than being named again here,
  /// so a request or response type that changes in the proto changes here by
  /// itself and cannot be half-updated.
  async call(client, method, request = {}) {
    try {
      const answer = await client[method.localName](fromJson(method.input, request));
      return toJson(method.output, answer, { alwaysEmitImplicit: true });
    } catch (failure) {
      throw DaemonRefusedTheRequest.from(failure);
    }
  }

  /// One server-streaming call, as an async iterable of JSON frames.
  ///
  /// `signal` is how a panel stops it — a stream with no way to end it is a
  /// stream that outlives the panel that opened it. `onHeader` is the moment
  /// the daemon accepted the call, which is what a "streaming" pill means: a
  /// quiet wire yields no frames for minutes and is perfectly healthy.
  async *follow(client, method, request, { signal, onHeader } = {}) {
    try {
      const frames = client[method.localName](fromJson(method.input, request), { signal, onHeader });
      for await (const frame of frames) {
        yield toJson(method.output, frame, { alwaysEmitImplicit: true });
      }
    } catch (failure) {
      if (signal?.aborted) return; // our own close, not a failure
      throw DaemonRefusedTheRequest.from(failure);
    }
  }

  // ------------------------------------------------------------- device ---

  readVersion() {
    return this.call(this.device, Device.method.readVersion);
  }

  readDevice() {
    return this.call(this.device, Device.method.readDevice);
  }

  connectToTheDevice() {
    return this.call(this.device, Device.method.openLink);
  }

  readFirmwareVersions() {
    return this.call(this.device, Device.method.readFirmware);
  }

  readWireLog() {
    return this.call(this.device, Device.method.readWireLog);
  }

  followWire(options) {
    return this.follow(this.device, Device.method.watchWire, {}, options);
  }

  // ------------------------------------------------------ state & stream ---

  readState() {
    return this.call(this.state, StateService.method.readState);
  }

  /// The decimated state stream. `rateHz` is what the browser asks for, not
  /// what the device samples at — the daemon decimates, and says what it lost.
  followState(rateHz, options) {
    return this.follow(this.state, StateService.method.watchState, { rate_hz: rateHz }, options);
  }

  /// Moves the API origin. Never the published accumulator — vstimd differences
  /// that every frame, and a value that jumps back jumps the camera back.
  zeroPosition(axes) {
    return this.call(this.state, StateService.method.zeroPosition, axes === undefined ? {} : { axes });
  }

  // -------------------------------------------------------- calibration ---

  readCalibration() {
    return this.call(this.calibration, Calibration.method.readCalibration);
  }

  replaceCalibration(calibration) {
    return this.call(this.calibration, Calibration.method.replaceCalibration, calibration);
  }

  startMeasuring(axis, knownDistanceCm) {
    return this.call(this.calibration, Calibration.method.startMeasuring, {
      axis,
      known_distance_cm: knownDistanceCm,
    });
  }

  finishMeasuring() {
    return this.call(this.calibration, Calibration.method.finishMeasuring);
  }

  applyMeasurement() {
    return this.call(this.calibration, Calibration.method.applyMeasurement);
  }

  // --------------------------------------------------------------- zones ---

  readZoneSetNames() {
    return this.call(this.zones, Zones.method.listZoneSets);
  }

  readZoneSet(name) {
    return this.call(this.zones, Zones.method.readZoneSet, { name });
  }

  replaceZoneSet(name, zoneSet) {
    return this.call(this.zones, Zones.method.replaceZoneSet, { name, zone_set: zoneSet });
  }

  /// Compile the set as it is *stored* — "is that one still good after the
  /// calibration changed".
  validateZoneSet(name) {
    return this.call(this.zones, Zones.method.validateZoneSet, { name });
  }

  /// Compile a **draft**: what somebody is typing, before it is saved.
  ///
  /// The body goes through the daemon's own deserializer and compiler, so what
  /// comes back is the refusal the real thing would give — no second, looser
  /// description of a zone set in a browser to disagree with it.
  validateDraft(zoneSet) {
    return this.call(this.zones, Zones.method.validateDraft, zoneSet);
  }

  /// The set as the **file** it is stored as, for an editor.
  ///
  /// `readZoneSet` above gives the same set as a message. Both are real: a
  /// generated client wants the message, and a text box wants the text. What
  /// this client must never do is convert between them — that would be a
  /// second description of a zone set, in a browser, disagreeing with the
  /// daemon's the first time either changes.
  readZoneSetFile(name) {
    return this.call(this.zones, Zones.method.readZoneSetFile, { name });
  }

  writeZoneSetFile(name, text) {
    return this.call(this.zones, Zones.method.writeZoneSetFile, { name, text });
  }

  /// Parse and compile a file that is not stored — what somebody is typing.
  validateZoneSetFile(text) {
    return this.call(this.zones, Zones.method.validateZoneSetFile, { text });
  }

  /// The zone set's JSON Schema, for an editor or a checker.
  ///
  /// The same document `mousewheeld schema` prints and the repository commits
  /// at `docs/reference/zone-set.schema.json`. It is served as an rpc as well
  /// because the one a *running* daemon believes is the one worth validating
  /// against, and a rig is not always on the version the checkout is.
  readZoneSetSchema() {
    return this.call(this.zones, Zones.method.readZoneSetSchema);
  }

  readArmedZones() {
    return this.call(this.zones, Zones.method.readArmed);
  }

  armZoneSet(request) {
    return this.call(this.zones, Zones.method.arm, request);
  }

  disarmZones() {
    return this.call(this.zones, Zones.method.disarm);
  }

  saveZonesToFlash() {
    return this.call(this.zones, Zones.method.saveToFlash);
  }

  readOutputLines() {
    return this.call(this.config, Config.method.readLines);
  }

  // -------------------------------------------------------------- config ---

  readConfig() {
    return this.call(this.config, Config.method.readConfig);
  }

  updateConfig(changes) {
    return this.call(this.config, Config.method.patchConfig, changes);
  }
}
