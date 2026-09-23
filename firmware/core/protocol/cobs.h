// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// Consistent Overhead Byte Stuffing. A frame is COBS-encoded so it contains no
// zero byte, and a zero ends it. That is what lets a receiver that lost bytes —
// a USB re-enumeration, a reset mid-frame — find the start of the next frame
// without parsing anything: it waits for a zero.

#include <stddef.h>
#include <stdint.h>

namespace mousewheeld::protocol {

// The most `cobs_encode` can produce from `len` bytes, not counting the
// delimiter.
constexpr size_t cobs_max_encoded(size_t len) { return len + len / 254 + 1; }

// Encode `len` bytes of `in` into `out`, which holds at least
// cobs_max_encoded(len). Returns the encoded length. Writes no delimiter.
size_t cobs_encode(const uint8_t* in, size_t len, uint8_t* out);

// Decode `len` bytes (no delimiter) into `out`, which may be `in`. Returns the
// decoded length, or 0 if the input is not valid COBS.
size_t cobs_decode(const uint8_t* in, size_t len, uint8_t* out);

}  // namespace mousewheeld::protocol
