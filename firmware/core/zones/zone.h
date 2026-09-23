// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// A compiled zone, as the board holds it: integer counts, axis indices, a line
// index. The daemon compiled centimetres and names into these; the board never
// sees either.

#include <stdint.h>

#include "config.h"
#include "refusal.h"

namespace mousewheeld {

enum class Metric : uint8_t { kDisplacement = 0, kDistance = 1 };
enum class Fire : uint8_t { kOnce = 0, kRearm = 1 };
enum class Action : uint8_t { kPulse = 0, kLevel = 1 };

// One axis of a rect. An absent bound is open.
struct Interval {
  uint8_t axis = 0;
  bool has_lo = false;
  bool has_hi = false;
  int64_t lo = 0;
  int64_t hi = 0;
};

struct Zone {
  Interval intervals[kMaxAxes];
  uint8_t n_intervals = 0;
  Metric metric = Metric::kDisplacement;
  Fire fire = Fire::kOnce;
  Action action = Action::kPulse;
  // Fire on arming if already inside.
  bool level_on_arm = false;
  uint8_t line = 0;
  uint16_t pulse_ms = 0;
  // Period in counts, 0 for a straight track.
  int64_t wrap = 0;
  int64_t hysteresis = 0;
};

struct ZoneSet {
  Zone zones[kMaxZones];
  uint8_t n = 0;
  uint32_t version = 0;
  char name[kMaxNameLength + 1] = {};
  // False until the first commit: an empty committed set is still a set.
  bool committed = false;
};

// Whether the board can evaluate `zone` with `n_axes` axes and `lines`
// configured.
Refusal check_zone(const Zone& zone, uint8_t n_axes, LineMask lines);

// The chunked upload: begin, one zone per index, commit. The live set is
// untouched until commit, so a link that dies mid-upload leaves the board
// running what it was running.
class ZoneUpload {
 public:
  Refusal begin(uint32_t version, uint32_t n, const char* name);
  Refusal add(uint32_t index, const Zone& zone);
  // Moves the staged set into `live` if every index arrived.
  Refusal commit(ZoneSet& live);

 private:
  ZoneSet staging_;
  uint32_t received_ = 0;  // bit i: zone i arrived
  bool open_ = false;
};

}  // namespace mousewheeld
