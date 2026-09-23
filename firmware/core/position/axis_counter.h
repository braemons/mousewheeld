// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// One axis: a wrapping hardware counter extended to a 64-bit accumulator, an
// origin, and an odometer.
//
// The accumulator only ever moves by what the wheel did. `zero` and arming move
// the origin — a different number — so the counts the host differences never
// step, and a host that missed a sample loses resolution but never distance.

#include <stdint.h>

namespace mousewheeld {

class AxisCounter {
 public:
  // Take `raw` as the reference for the next update, without moving.
  void attach(int32_t raw) { last_raw_ = raw; }

  // Fold a new hardware reading in. Returns whether the axis moved.
  bool update(int32_t raw, uint32_t modulus) {
    int64_t delta = static_cast<int64_t>(raw) - last_raw_;
    last_raw_ = raw;
    if (delta == 0) return false;
    // The reading is only known modulo `modulus`: take the representative
    // nearest zero. Exact while an axis moves less than half the modulus per
    // scan, which at 5 kHz and a 32000 modulus is 80 million counts a second.
    const int64_t m = modulus;
    delta %= m;
    if (delta >= m / 2) delta -= m;
    if (delta < -m / 2) delta += m;
    if (delta == 0) return false;
    if (invert_) delta = -delta;
    counts_ += delta;
    distance_ += delta < 0 ? -delta : delta;
    return true;
  }

  void set_invert(bool invert) { invert_ = invert; }
  bool invert() const { return invert_; }

  int64_t counts() const { return counts_; }
  int64_t origin() const { return origin_; }
  int64_t displacement() const { return counts_ - origin_; }
  int64_t distance() const { return distance_; }

  // Displacement and distance are zero here from now on.
  void zero() {
    origin_ = counts_;
    distance_ = 0;
  }

 private:
  int64_t counts_ = 0;
  int64_t origin_ = 0;
  int64_t distance_ = 0;
  int32_t last_raw_ = 0;
  bool invert_ = false;
};

}  // namespace mousewheeld
