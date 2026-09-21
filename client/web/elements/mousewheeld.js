// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The `/elements/` contract, for mousewheeld. statemachined's
// `docs/developer/daemon.md` §5 specifies it; this is mousewheeld's instance.
//
//     <script type="module" src="http://rig.local:8083/elements/mousewheeld.js"></script>
//     <mousewheeld-device      base="http://rig.local:8083"></mousewheeld-device>
//     <mousewheeld-trace       base="http://rig.local:8083"></mousewheeld-trace>
//     <mousewheeld-zones       base="http://rig.local:8083"></mousewheeld-zones>
//     <mousewheeld-calibration base="http://rig.local:8083"></mousewheeld-calibration>
//     <mousewheeld-monitor     base="http://rig.local:8083"></mousewheeld-monitor>
//
// **This URL and these five tag names are what the console repo depends on.**
// The console is a static shell with no domain logic: every panel it shows is
// served by the daemon that owns the hardware it is about, at that daemon's own
// version, which is what stops a console bundling a copy of this UI and
// drifting the first time a field changes.
//
// Three properties this shape buys, each of them the point:
//
//   * Every panel has a **shadow root**, so a console's global styles cannot
//     reach into one dropped onto its page — natively, with no tooling.
//   * Every attribute is a **string**, so React's attribute-only custom-element
//     support is enough. vstimd's own UI is React + Vite and this one is plain
//     ES modules; neither has to become the other.
//   * `base` is an attribute rather than an assumption, because a console is
//     not served from the rig.
//
// Importing this module registers all five. Importing one panel's module
// directly registers only that one, and is also supported — a console that
// wants the trace and nothing else should not pay for the zone editor.
//
// **No framework and no CDN.** Every file here is served as written, with one
// exception that is generated and committed: `daemon_api_client.js` is the
// protobuf client, bundled from `proto/mousewheeld/v1/` by `make web`, because
// the API is gRPC-Web and a browser cannot make a protobuf client out of
// nothing. Nothing else has a build step, and `make dev` still means edit a
// panel and reload the page.

export { DaemonApiClient, DaemonRefusedTheRequest } from "./daemon_api_client.js";
export { BasePanelElement } from "./base_panel_element.js";
export { DevicePanelElement } from "./device_panel_element.js";
export { TracePanelElement } from "./trace_panel_element.js";
export { ZonePanelElement } from "./zone_panel_element.js";
export { CalibrationPanelElement } from "./calibration_panel_element.js";
export { SerialMonitorPanelElement } from "./serial_monitor_panel_element.js";
// Pure, and exported because a console that draws a wheel's trace should be
// able to draw it the way this UI does rather than inventing a second
// convention for the same numbers.
export { domainOf, decimalsFor, drawTimeSeries, drawEventRules } from "./time_series_chart.js";
// The wire's shapes, exported for the same reason: the client hands a panel
// protobuf's JSON mapping, which puts a 64-bit integer in a string and spells
// an enum in full, and a console reading this daemon's answers should not have
// to rediscover that.
export { count, enumName, metricName, streamFrame, bound } from "./wire_shapes.js";

/// The tag names, so a console can iterate them rather than hard-code a list
/// that goes stale when a panel is added.
export const MOUSEWHEELD_ELEMENT_NAMES = [
  "mousewheeld-device",
  "mousewheeld-trace",
  "mousewheeld-zones",
  "mousewheeld-calibration",
  "mousewheeld-monitor",
];
