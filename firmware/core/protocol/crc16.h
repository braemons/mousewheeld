// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

#include <stddef.h>
#include <stdint.h>

namespace mousewheeld::protocol {

// CRC-16/CCITT-FALSE: polynomial 0x1021, initial 0xFFFF, no reflection, no
// final XOR. The daemon's link uses the same parameters.
constexpr uint16_t kCrcInit = 0xFFFF;

uint16_t crc16_update(uint16_t crc, const uint8_t* bytes, size_t len);

inline uint16_t crc16(const uint8_t* bytes, size_t len) {
  return crc16_update(kCrcInit, bytes, len);
}

}  // namespace mousewheeld::protocol
