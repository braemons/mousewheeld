// SPDX-License-Identifier: AGPL-3.0-or-later
#include "output/lines.h"

#include "hal.h"

namespace mousewheeld {

void Lines::configure(uint8_t index, uint8_t pin, bool safe_high) {
  const LineMask bit = static_cast<LineMask>(1u << index);
  pins_[index] = pin;
  configured_ |= bit;
  safe_high_ = safe_high ? (safe_high_ | bit) : (safe_high_ & ~bit);
  pulses_ &= ~bit;
  levels_ &= ~bit;
  applied_ &= ~bit;
  hal::configure_line(index, pin, safe_high);
}

void Lines::pulse(uint8_t line, Microseconds now, uint16_t ms) {
  const LineMask bit = static_cast<LineMask>(1u << line);
  const Microseconds end = now + static_cast<Microseconds>(ms) * 1000;
  if (!(pulses_ & bit) || end > pulse_end_[line]) pulse_end_[line] = end;
  pulses_ |= bit;
}

void Lines::tick(Microseconds now) {
  if (pulses_) {
    for (uint8_t i = 0; i < kMaxLines; ++i) {
      if ((pulses_ & (1u << i)) && now >= pulse_end_[i]) {
        pulses_ &= static_cast<LineMask>(~(1u << i));
      }
    }
  }
  const LineMask active = (pulses_ | levels_) & configured_;
  if (active != applied_) apply(active);
}

void Lines::safe() {
  pulses_ = 0;
  levels_ = 0;
  apply(0);
}

void Lines::apply(LineMask active) {
  // Active is the level that is not safe: high for a line that idles low.
  const LineMask high = ((active & ~safe_high_) | (~active & safe_high_)) & configured_;
  const LineMask low = static_cast<LineMask>(~high) & configured_;
  hal::write_outputs(high, low);
  applied_ = active;
}

}  // namespace mousewheeld
