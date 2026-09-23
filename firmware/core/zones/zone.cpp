// SPDX-License-Identifier: AGPL-3.0-or-later
#include "zones/zone.h"

#include <string.h>

namespace mousewheeld {

Refusal check_zone(const Zone& zone, uint8_t n_axes, LineMask lines) {
  if (zone.n_intervals == 0) {
    return Refusal::because("bad_zone", "a zone needs at least one interval");
  }
  uint8_t seen = 0;
  for (uint8_t i = 0; i < zone.n_intervals; ++i) {
    const Interval& interval = zone.intervals[i];
    if (interval.axis >= n_axes) {
      return Refusal::because("bad_axis", "axis %u, and %u are configured",
                              interval.axis, n_axes);
    }
    if (seen & (1u << interval.axis)) {
      return Refusal::because("bad_zone", "axis %u has two intervals", interval.axis);
    }
    seen |= static_cast<uint8_t>(1u << interval.axis);
    if (interval.has_lo && interval.has_hi && interval.lo > interval.hi) {
      return Refusal::because("bad_zone", "axis %u: lo is above hi", interval.axis);
    }
    if (zone.wrap > 0 && ((interval.has_lo && (interval.lo < 0 || interval.lo >= zone.wrap)) ||
                          (interval.has_hi && (interval.hi < 0 || interval.hi >= zone.wrap)))) {
      return Refusal::because("bad_zone", "axis %u: a bound outside [0, wrap)", interval.axis);
    }
  }
  if (zone.wrap < 0) return Refusal::because("bad_zone", "wrap is negative");
  if (zone.hysteresis < 0) return Refusal::because("bad_zone", "hysteresis is negative");
  if (zone.line >= kMaxLines || !(lines & (1u << zone.line))) {
    return Refusal::because("bad_line", "line %u is not configured", zone.line);
  }
  if (zone.action == Action::kPulse && zone.pulse_ms == 0) {
    return Refusal::because("bad_zone", "a pulse of 0 ms");
  }
  return Refusal::none();
}

Refusal ZoneUpload::begin(uint32_t version, uint32_t n, const char* name) {
  open_ = false;
  if (n > kMaxZones) {
    return Refusal::because("zone_table_full", "%u zones, and this board holds %u",
                            static_cast<unsigned>(n), kMaxZones);
  }
  if (strlen(name) > kMaxNameLength) {
    return Refusal::because("bad_name", "longer than %u characters",
                            static_cast<unsigned>(kMaxNameLength));
  }
  staging_ = ZoneSet{};
  staging_.n = static_cast<uint8_t>(n);
  staging_.version = version;
  strncpy(staging_.name, name, kMaxNameLength);
  received_ = 0;
  open_ = true;
  return Refusal::none();
}

Refusal ZoneUpload::add(uint32_t index, const Zone& zone) {
  if (!open_) return Refusal::because("no_upload", "a zone without zones_begin");
  if (index >= staging_.n) {
    return Refusal::because("bad_zone", "index %u of a set of %u",
                            static_cast<unsigned>(index), staging_.n);
  }
  staging_.zones[index] = zone;
  received_ |= 1u << index;
  return Refusal::none();
}

Refusal ZoneUpload::commit(ZoneSet& live) {
  if (!open_) return Refusal::because("no_upload", "zones_end without zones_begin");
  const uint32_t expected = staging_.n == 32 ? 0xFFFFFFFFu : (1u << staging_.n) - 1;
  if (received_ != expected) {
    for (uint8_t i = 0; i < staging_.n; ++i) {
      if (!(received_ & (1u << i))) {
        return Refusal::because("incomplete_upload", "zone %u never arrived", i);
      }
    }
  }
  staging_.committed = true;
  live = staging_;
  open_ = false;
  return Refusal::none();
}

}  // namespace mousewheeld
