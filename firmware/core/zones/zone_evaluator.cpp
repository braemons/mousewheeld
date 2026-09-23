// SPDX-License-Identifier: AGPL-3.0-or-later
#include "zones/zone_evaluator.h"

namespace mousewheeld {
namespace {

int64_t modulo(int64_t value, int64_t period) {
  const int64_t r = value % period;
  return r < 0 ? r + period : r;
}

// How far outside `zone` the metrics are: 0 when inside, else the largest
// distance to an interval, the short way round on a wrapped track.
int64_t outside_by(const Zone& zone, const Metrics& metrics) {
  int64_t worst = 0;
  for (uint8_t i = 0; i < zone.n_intervals; ++i) {
    const Interval& interval = zone.intervals[i];
    int64_t value = zone.metric == Metric::kDistance ? metrics.distance[interval.axis]
                                                     : metrics.displacement[interval.axis];
    if (zone.wrap > 0) value = modulo(value, zone.wrap);

    const bool below = interval.has_lo && value < interval.lo;
    const bool above = interval.has_hi && value > interval.hi;
    if (!below && !above) continue;

    int64_t by;
    if (zone.wrap > 0 && interval.has_lo && interval.has_hi) {
      const int64_t up_to_lo = modulo(interval.lo - value, zone.wrap);
      const int64_t down_to_hi = modulo(value - interval.hi, zone.wrap);
      by = up_to_lo < down_to_hi ? up_to_lo : down_to_hi;
    } else {
      by = below ? interval.lo - value : value - interval.hi;
    }
    if (by > worst) worst = by;
  }
  return worst;
}

}  // namespace

uint32_t ZoneEvaluator::arm(const ZoneSet& set, uint32_t arm_id, const Metrics& metrics) {
  set_ = set;
  arm_id_ = arm_id;
  armed_ = true;
  engaged_ = 0;
  can_fire_ = set_.n == 32 ? 0xFFFFFFFFu : (1u << set_.n) - 1;

  uint32_t fired = 0;
  for (uint8_t i = 0; i < set_.n; ++i) {
    const Zone& zone = set_.zones[i];
    if (outside_by(zone, metrics) != 0) continue;
    engaged_ |= 1u << i;
    if (zone.level_on_arm) {
      fired |= 1u << i;
      if (zone.fire == Fire::kOnce) can_fire_ &= ~(1u << i);
    }
  }
  return fired;
}

void ZoneEvaluator::disarm() {
  armed_ = false;
  engaged_ = 0;
  can_fire_ = 0;
}

uint32_t ZoneEvaluator::evaluate(const Metrics& metrics) {
  if (!armed_) return 0;
  uint32_t fired = 0;
  for (uint8_t i = 0; i < set_.n; ++i) {
    const Zone& zone = set_.zones[i];
    const uint32_t bit = 1u << i;
    const int64_t by = outside_by(zone, metrics);
    if (!(engaged_ & bit)) {
      if (by != 0) continue;
      engaged_ |= bit;
      if (can_fire_ & bit) {
        fired |= bit;
        if (zone.fire == Fire::kOnce) can_fire_ &= ~bit;
      }
    } else if (by != 0 && by >= zone.hysteresis) {
      engaged_ &= ~bit;
    }
  }
  return fired;
}

LineMask ZoneEvaluator::level_lines() const {
  if (!armed_) return 0;
  LineMask lines = 0;
  for (uint8_t i = 0; i < set_.n; ++i) {
    if ((engaged_ & (1u << i)) && set_.zones[i].action == Action::kLevel) {
      lines |= static_cast<LineMask>(1u << set_.zones[i].line);
    }
  }
  return lines;
}

}  // namespace mousewheeld
