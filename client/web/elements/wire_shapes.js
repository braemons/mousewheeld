// SPDX-License-Identifier: AGPL-3.0-or-later
//
// What a panel reads, once the client has handed it protobuf's JSON mapping.
//
// `daemon_api_client.js` converts every answer with `toJson` — the binary
// gRPC-Web frame never reaches a panel — and that mapping has three habits a
// panel should not have to remember:
//
//   * a 64-bit integer is a **string**, because JSON numbers are doubles and
//     `counts` would silently lose its last digits above 2^53;
//   * an enum is its full name, `ZONE_METRIC_DISPLACEMENT`, so that a client
//     generated in another language agrees about the same byte;
//   * a `oneof` is an object with one key, `{"sample": {…}}`, rather than a
//     discriminator beside flattened fields.
//
// Every one of those is right in an interface and wrong in a panel, so the
// translation happens here — once — exactly as `daemon/src/convert/` does it on
// the other side. A panel that reads `frame.sample.seq` in three places is a
// panel that will be edited in two of them.

/// A 64-bit integer as a number.
///
/// Safe for every counter this rig produces: `counts` at 4096 per revolution
/// passes 2^53 after about nine billion kilometres, and `host_monotonic_ns`
/// after 104 days of uptime — worth knowing, not worth a BigInt in a panel.
export function count(value) {
  return typeof value === "string" ? Number(value) : (value ?? 0);
}

/// `ZONE_METRIC_DISPLACEMENT` → `displacement`, which is what the zone set file
/// says and what a person reading a panel expects to see.
export function enumName(value, prefix) {
  if (typeof value !== "string") return "";
  return (value.startsWith(prefix) ? value.slice(prefix.length) : value).toLowerCase();
}

export const metricName = (value) => enumName(value, "ZONE_METRIC_");
export const shapeName = (value) => enumName(value, "ZONE_SHAPE_");
export const fireName = (value) => enumName(value, "FIRE_RULE_");
export const actionName = (value) => enumName(value, "OUTPUT_ACTION_");
export const directionName = (value) => enumName(value, "WIRE_DIRECTION_");
export const levelName = (value) => enumName(value, "WIRE_LEVEL_");

/// One frame of `StateService.WatchState`, flattened to `{kind, …}`.
///
/// The stream carries `{"sample": {…}}` or `{"zone_hit": {…}}`; a reader wants
/// to switch on one field. Returns `null` for an arm this build does not know,
/// which is a frame from a newer daemon and is skipped rather than half-read.
export function streamFrame(message) {
  if (message && message.sample) return { kind: "sample", ...message.sample };
  if (message && message.zone_hit) return { kind: "zone_hit", ...message.zone_hit };
  return null;
}

/// A resolved bound: `{"value": 30}` for a number, `{}` for an open end.
///
/// protobuf has no nullable double inside a repeated field, so the absence is
/// carried by an empty object. `null` is what the rest of the UI draws with.
export function bound(value) {
  return value && typeof value.value === "number" ? value.value : null;
}

