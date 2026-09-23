// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// A voltage proportional to position, with no daemon running: the debugging
// tool that needs nothing but a scope. Updated from loop() at a fixed rate,
// never from the scan.

#include <stdint.h>

#include "config.h"
#include "refusal.h"

namespace mousewheeld {

enum class AnalogMode : uint8_t { kOff = 0, kWrap = 1, kClamp = 2, kVelocity = 3 };

struct AnalogConfig {
  uint8_t axis = 0;
  AnalogMode mode = AnalogMode::kOff;
  int64_t range_counts = 0;
  int64_t lo = 0;
  int64_t hi = 0;
};

Refusal check_analog(const AnalogConfig& config, uint8_t n_axes);

// 0..65535 across the configured range. kOff is mid-scale.
uint16_t analog_value(const AnalogConfig& config, int64_t displacement, int64_t velocity);

}  // namespace mousewheeld
