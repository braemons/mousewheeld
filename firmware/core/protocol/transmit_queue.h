// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// Encoded frames waiting for the serial port. loop() fills it and drains it
// into hal::serial_write() as fast as the port takes bytes, so a frame is never
// written from anywhere that could block.

#include <stddef.h>
#include <stdint.h>

namespace mousewheeld::protocol {

class TransmitQueue {
 public:
  static constexpr size_t kCapacity = 2048;

  size_t free() const { return kCapacity - used_; }

  // All of `bytes`, or none of them.
  bool push(const uint8_t* bytes, size_t len);

  // Hand queued bytes to `write` (which returns how many it took) until it
  // takes fewer than offered.
  template <typename Write>
  void drain(Write write) {
    while (used_ != 0) {
      const size_t to_end = kCapacity - tail_;
      const size_t chunk = used_ < to_end ? used_ : to_end;
      const size_t took = write(buffer_ + tail_, chunk);
      tail_ = (tail_ + took) % kCapacity;
      used_ -= took;
      if (took < chunk) return;
    }
  }

 private:
  uint8_t buffer_[kCapacity];
  size_t head_ = 0;  // where the next byte goes
  size_t tail_ = 0;  // the oldest byte not yet written
  size_t used_ = 0;
};

}  // namespace mousewheeld::protocol
