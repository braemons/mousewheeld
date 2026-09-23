// SPDX-License-Identifier: AGPL-3.0-or-later
#include "settings.h"

#include <stddef.h>
#include <string.h>

#include "protocol/crc16.h"

namespace mousewheeld {
namespace {

constexpr uint32_t kMagic = 0x4D574431;  // "MWD1"
// Bump on any change to Settings or to a struct inside it.
constexpr uint16_t kFormat = 1;

uint16_t checksum(const Settings& settings) {
  return protocol::crc16(reinterpret_cast<const uint8_t*>(&settings), offsetof(Settings, crc));
}

}  // namespace

Settings capture_settings(const Device& device) {
  Settings s;
  // Padding included, so the CRC is over defined bytes.
  memset(static_cast<void*>(&s), 0, sizeof(s));
  s.magic = kMagic;
  s.format = kFormat;
  s.size = sizeof(Settings);
  s.n_axes = device.n_axes();
  for (uint8_t a = 0; a < kMaxAxes; ++a) s.invert[a] = device.axis(a).invert();
  s.lines_configured = device.lines().configured();
  for (uint8_t i = 0; i < kMaxLines; ++i) {
    if (device.lines().safe_high(i)) s.lines_safe_high |= static_cast<LineMask>(1u << i);
    s.line_pins[i] = device.lines().pin(i);
  }
  s.analog = device.analog();
  s.zones = device.committed();
  s.armed = device.armed();
  s.arm_id = device.arm_id();
  s.crc = checksum(s);
  return s;
}

bool settings_valid(const uint8_t* bytes, size_t len) {
  if (len != sizeof(Settings)) return false;
  Settings s;
  memcpy(static_cast<void*>(&s), bytes, sizeof(s));
  return s.magic == kMagic && s.format == kFormat && s.size == sizeof(Settings) &&
         s.crc == checksum(s);
}

Refusal restore_settings(Device& device, const Settings& s, Microseconds now) {
  AxisSetting axes[kMaxAxes];
  for (uint8_t a = 0; a < kMaxAxes; ++a) axes[a].invert = s.invert[a];
  if (Refusal refusal = device.set_axes(axes, s.n_axes)) return refusal;
  for (uint8_t i = 0; i < kMaxLines; ++i) {
    if (!(s.lines_configured & (1u << i))) continue;
    if (Refusal refusal = device.set_line(i, s.line_pins[i], s.lines_safe_high & (1u << i))) {
      return refusal;
    }
  }
  if (Refusal refusal = device.set_analog(s.analog)) return refusal;
  if (!s.zones.committed) return Refusal::none();

  // Through the upload, so a flashed set is checked exactly as a sent one is.
  if (Refusal refusal = device.begin_upload(s.zones.version, s.zones.n, s.zones.name)) {
    return refusal;
  }
  for (uint8_t i = 0; i < s.zones.n; ++i) {
    if (Refusal refusal = device.add_zone(i, s.zones.zones[i])) return refusal;
  }
  if (Refusal refusal = device.end_upload()) return refusal;
  if (!s.armed) return Refusal::none();
  return device.arm(s.arm_id, s.zones.version, true, now);
}

}  // namespace mousewheeld
