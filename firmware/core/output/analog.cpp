// SPDX-License-Identifier: AGPL-3.0-or-later
#include "output/analog.h"

namespace mousewheeld {
namespace {

// Keeps value × 65535 inside an int64.
constexpr int64_t kMaxSpan = int64_t{1} << 40;

uint16_t scale(int64_t value, int64_t lo, int64_t hi) {
  if (value <= lo) return 0;
  if (value >= hi) return 65535;
  return static_cast<uint16_t>((value - lo) * 65535 / (hi - lo));
}

}  // namespace

Refusal check_analog(const AnalogConfig& config, uint8_t n_axes) {
  if (config.mode == AnalogMode::kOff) return Refusal::none();
  if (config.axis >= n_axes) {
    return Refusal::because("bad_axis", "axis %u, and %u are configured", config.axis, n_axes);
  }
  if (config.mode == AnalogMode::kWrap) {
    if (config.range_counts <= 0 || config.range_counts > kMaxSpan) {
      return Refusal::because("bad_analog", "wrap needs 0 < range_counts <= 2^40");
    }
    return Refusal::none();
  }
  if (config.hi <= config.lo || config.hi - config.lo > kMaxSpan) {
    return Refusal::because("bad_analog", "needs lo < hi, at most 2^40 apart");
  }
  return Refusal::none();
}

uint16_t analog_value(const AnalogConfig& config, int64_t displacement, int64_t velocity) {
  switch (config.mode) {
    case AnalogMode::kWrap: {
      int64_t r = displacement % config.range_counts;
      if (r < 0) r += config.range_counts;
      return static_cast<uint16_t>(r * 65535 / config.range_counts);
    }
    case AnalogMode::kClamp:
      return scale(displacement, config.lo, config.hi);
    case AnalogMode::kVelocity:
      return scale(velocity, config.lo, config.hi);
    case AnalogMode::kOff:
      break;
  }
  return 32768;
}

}  // namespace mousewheeld
