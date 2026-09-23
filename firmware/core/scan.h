// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// One scan, as the timer runs it: read every counter, hand the readings to the
// Device, and time it. Shared by the board and the native build so they scan
// identically.

#include "device.h"
#include "hal.h"

namespace mousewheeld {

inline void scan_once(Device& device) {
  ScanLock lock;
  const Microseconds now = hal::micros_now();
  int32_t raw[kMaxAxes];
  for (uint8_t a = 0; a < kMaxAxes; ++a) raw[a] = hal::read_counter(a);
  device.scan(now, raw, hal::counter_modulus());
  device.note_scan_duration(static_cast<uint32_t>(hal::micros_now() - now));
}

// Before start_scan(): the counters' current readings are where counting
// starts.
inline void attach_counters(Device& device) {
  int32_t raw[kMaxAxes];
  for (uint8_t a = 0; a < kMaxAxes; ++a) raw[a] = hal::read_counter(a);
  device.attach(raw);
}

}  // namespace mousewheeld
