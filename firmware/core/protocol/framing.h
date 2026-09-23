// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// One frame on the wire, both directions:
//
//     COBS( protobuf message ‖ CRC-16 big-endian ) ‖ 0x00
//
// The CRC covers the protobuf bytes. A frame whose CRC does not match is
// refused whole and never acted on, not even partially — a truncated zone
// upload is exactly what it exists to catch.

#include <stddef.h>
#include <stdint.h>

#include "config.h"
#include "protocol/cobs.h"

namespace mousewheeld::protocol {

// The largest protobuf message a frame of kMaxFrame bytes can carry.
constexpr size_t kMaxPayload = kMaxFrame - 1 /* delimiter */ - 2 /* crc */ - (kMaxFrame / 254 + 1);

static_assert(cobs_max_encoded(kMaxPayload + 2) + 1 <= kMaxFrame, "kMaxPayload fits a frame");

// Seal `len` bytes of protobuf into a frame in `out` (kMaxFrame bytes).
// Returns the frame length including the delimiter.
size_t encode_frame(const uint8_t* payload, size_t len, uint8_t* out);

// Bytes in, checked payloads out. Resynchronises on the next delimiter after
// anything it refuses.
class FrameReader {
 public:
  enum class Result : uint8_t { kPending, kFrame, kRefused };

  Result push(uint8_t byte);

  // Valid after kFrame, until the next push().
  const uint8_t* payload() const { return buffer_; }
  size_t size() const { return size_; }

  // Valid after kRefused: "bad_crc", "bad_cobs" or "frame_too_long".
  const char* refusal() const { return refusal_; }

 private:
  uint8_t buffer_[kMaxFrame];
  size_t fill_ = 0;
  size_t size_ = 0;
  bool overflowed_ = false;
  const char* refusal_ = "";
};

}  // namespace mousewheeld::protocol
