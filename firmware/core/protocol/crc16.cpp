// SPDX-License-Identifier: AGPL-3.0-or-later
#include "protocol/crc16.h"

namespace mousewheeld::protocol {

uint16_t crc16_update(uint16_t crc, const uint8_t* bytes, size_t len) {
  // Bitwise rather than a table: 512 bytes of table buys nothing at 256-byte
  // frames, and this runs in loop(), never in the scan.
  for (size_t i = 0; i < len; ++i) {
    crc ^= static_cast<uint16_t>(bytes[i]) << 8;
    for (int bit = 0; bit < 8; ++bit) {
      crc = (crc & 0x8000) ? static_cast<uint16_t>((crc << 1) ^ 0x1021)
                           : static_cast<uint16_t>(crc << 1);
    }
  }
  return crc;
}

}  // namespace mousewheeld::protocol
