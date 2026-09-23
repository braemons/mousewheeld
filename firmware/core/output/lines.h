// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// The output lines: which pin, which level is safe, and what is driving them
// now. Pulses start and end in the scan, so a TTL's width is set by the scan
// clock and never by how busy loop() is.
//
// A line is *active* at the level that is not its safe one. Everything above
// this class thinks in active/inactive; only apply() knows about polarity.

#include <stdint.h>

#include "config.h"

namespace mousewheeld {

class Lines {
 public:
  // Bind `index` to `pin` and drive it safe.
  void configure(uint8_t index, uint8_t pin, bool safe_high);
  LineMask configured() const { return configured_; }
  uint8_t pin(uint8_t index) const { return pins_[index]; }
  bool safe_high(uint8_t index) const { return safe_high_ & (1u << index); }

  // Active for `ms` from `now`. A pulse on a line already pulsing extends it.
  void pulse(uint8_t line, Microseconds now, uint16_t ms);
  // The lines level zones hold active.
  void set_levels(LineMask active) { levels_ = active; }
  // End pulses that are due, and write whatever changed.
  void tick(Microseconds now);
  // Every line to its safe level, pulses and levels forgotten.
  void safe();

  LineMask active() const { return applied_; }

 private:
  void apply(LineMask active);

  Microseconds pulse_end_[kMaxLines] = {};
  uint8_t pins_[kMaxLines] = {};
  LineMask configured_ = 0;
  LineMask safe_high_ = 0;
  LineMask pulses_ = 0;
  LineMask levels_ = 0;
  LineMask applied_ = 0;
};

}  // namespace mousewheeld
