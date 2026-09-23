// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// The armed set, evaluated in the scan.
//
// Semantics, which are wire-visible and specified in dev/PLAN.md:
//
// - A zone is **engaged** on entry — the scan that sees the metric inside every
//   interval — and disengaged once it is outside by at least `hysteresis`. So
//   encoder jitter at a boundary is one engagement, not a pulse train.
// - A zone **fires** when it engages and can still fire. `once` can fire one
//   time per arm; `rearm` fires on every engagement.
// - Arming a zone that is already inside engages it without firing, unless
//   `level_on_arm`.
// - A `level` action is active while its zone is engaged, for as long as the
//   set is armed; `once` limits the zone_hit and the pulse, not the level.
// - Zones are evaluated in declaration order, so two firing in one scan are
//   reported in that order.

#include <stdint.h>

#include "config.h"
#include "zones/zone.h"

namespace mousewheeld {

struct Metrics {
  int64_t displacement[kMaxAxes] = {};
  int64_t distance[kMaxAxes] = {};
};

class ZoneEvaluator {
 public:
  // Take a copy of `set` — the scan must never read a set loop() may replace —
  // and engage what is already inside. Returns the zones that fire on arming.
  uint32_t arm(const ZoneSet& set, uint32_t arm_id, const Metrics& metrics);
  void disarm();

  // One scan. Returns the zones that fired, bit i for zone i.
  uint32_t evaluate(const Metrics& metrics);

  bool armed() const { return armed_; }
  uint32_t arm_id() const { return arm_id_; }
  uint32_t version() const { return set_.version; }
  const Zone& zone(uint8_t index) const { return set_.zones[index]; }
  uint8_t size() const { return set_.n; }

  // Zones that can still fire.
  uint32_t live() const { return armed_ ? can_fire_ : 0; }
  // Lines a level zone is holding active.
  LineMask level_lines() const;

 private:
  ZoneSet set_;
  uint32_t engaged_ = 0;
  uint32_t can_fire_ = 0;
  uint32_t arm_id_ = 0;
  bool armed_ = false;
};

}  // namespace mousewheeld
