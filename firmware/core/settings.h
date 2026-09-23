// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// What `Save` writes to flash and boot restores, so a board with nothing but
// power comes up counting, driving its lines and — if it was saved armed —
// running its zones.
//
// The blob is this struct's bytes, framed by a magic, a format number, its size
// and a CRC. A blob from another firmware layout is refused whole and the board
// boots unconfigured, saying so in a Log: a half-read zone set is worse than
// none.

#include <stdint.h>

#include "config.h"
#include "device.h"
#include "output/analog.h"
#include "zones/zone.h"

namespace mousewheeld {

struct Settings {
  uint32_t magic;
  uint16_t format;
  uint16_t size;

  uint8_t n_axes;
  bool invert[kMaxAxes];
  LineMask lines_configured;
  LineMask lines_safe_high;
  uint8_t line_pins[kMaxLines];
  AnalogConfig analog;
  ZoneSet zones;
  bool armed;
  uint32_t arm_id;

  // Over every byte before it.
  uint16_t crc;
};

Settings capture_settings(const Device& device);

// Whether `bytes` is a blob this firmware wrote.
bool settings_valid(const uint8_t* bytes, size_t len);

// Apply to a freshly booted device. Arms with the origin at the current
// position — the only origin a board that just reset has.
Refusal restore_settings(Device& device, const Settings& settings, Microseconds now);

}  // namespace mousewheeld
