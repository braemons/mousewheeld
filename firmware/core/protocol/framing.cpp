// SPDX-License-Identifier: AGPL-3.0-or-later
#include "protocol/framing.h"

#include <string.h>

#include "protocol/crc16.h"

namespace mousewheeld::protocol {

size_t encode_frame(const uint8_t* payload, size_t len, uint8_t* out) {
  uint8_t sealed[kMaxPayload + 2];
  if (len > kMaxPayload) return 0;
  memcpy(sealed, payload, len);
  const uint16_t crc = crc16(payload, len);
  sealed[len] = static_cast<uint8_t>(crc >> 8);
  sealed[len + 1] = static_cast<uint8_t>(crc & 0xFF);
  const size_t encoded = cobs_encode(sealed, len + 2, out);
  out[encoded] = 0;
  return encoded + 1;
}

FrameReader::Result FrameReader::push(uint8_t byte) {
  if (byte != 0) {
    if (fill_ < sizeof(buffer_)) {
      buffer_[fill_++] = byte;
    } else {
      overflowed_ = true;
    }
    return Result::kPending;
  }

  // A delimiter: whatever was collected is one frame.
  const size_t collected = fill_;
  const bool overflowed = overflowed_;
  fill_ = 0;
  overflowed_ = false;

  // Back-to-back delimiters are how a sender resynchronises a receiver; an
  // empty frame is not an error.
  if (collected == 0 && !overflowed) return Result::kPending;
  if (overflowed) {
    refusal_ = "frame_too_long";
    return Result::kRefused;
  }
  const size_t decoded = cobs_decode(buffer_, collected, buffer_);
  if (decoded < 2) {
    refusal_ = "bad_cobs";
    return Result::kRefused;
  }
  size_ = decoded - 2;
  const uint16_t sent = static_cast<uint16_t>((buffer_[size_] << 8) | buffer_[size_ + 1]);
  if (crc16(buffer_, size_) != sent) {
    refusal_ = "bad_crc";
    return Result::kRefused;
  }
  return Result::kFrame;
}

}  // namespace mousewheeld::protocol
