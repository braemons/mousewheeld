// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// What the scan hands loop(): fixed-size, already numbered. Formatting them is
// loop()'s job — nothing that builds a frame runs in the scan.

#include <stdint.h>

#include "config.h"

namespace mousewheeld {

struct SampleRecord {
  uint64_t seq;
  Microseconds t_us;
  int64_t counts[kMaxAxes];
  int64_t velocity[kMaxAxes];
};

struct ZoneHitRecord {
  uint64_t seq;
  Microseconds t_us;
  uint32_t arm_id;
  uint8_t zone;
  int64_t counts[kMaxAxes];
};

}  // namespace mousewheeld
