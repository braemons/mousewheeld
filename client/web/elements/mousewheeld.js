// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The `/elements/` contract, for mousewheeld. statemachined's
// `docs/developer/daemon.md` §5 specifies it; this is mousewheeld's instance.
//
//     <script type="module" src="http://rig.local:8082/elements/mousewheeld.js"></script>
//     <mousewheeld-device      base="http://rig.local:8082"></mousewheeld-device>
//     <mousewheeld-trace       base="http://rig.local:8082"></mousewheeld-trace>
//     <mousewheeld-zones       base="http://rig.local:8082"></mousewheeld-zones>
//     <mousewheeld-calibration base="http://rig.local:8082"></mousewheeld-calibration>
//     <mousewheeld-monitor     base="http://rig.local:8082"></mousewheeld-monitor>
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
//     support is enough. vstimd's own UI is React + Vite and this one has no
//     build step at all; neither has to become the other.
//   * `base` is an attribute rather than an assumption, because a console is
//     not served from the rig.
//
// Importing this module registers all five. Importing one panel's module
// directly registers only that one, and is also supported — a console that
// wants the trace and nothing else should not pay for the zone editor.
//
// **No build step, no framework, no CDN.** These files are served as written.

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
// The wire's shapes, exported for the same reason: protobuf's JSON mapping puts
// a 64-bit integer in a string and spells an enum in full, and a console that
// reads this daemon's answers directly should not have to rediscover that.
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
