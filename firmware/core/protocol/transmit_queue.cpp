// SPDX-License-Identifier: AGPL-3.0-or-later
#include "protocol/transmit_queue.h"

namespace mousewheeld::protocol {

bool TransmitQueue::push(const uint8_t* bytes, size_t len) {
  if (len > free()) return false;
  for (size_t i = 0; i < len; ++i) {
    buffer_[head_] = bytes[i];
    head_ = (head_ + 1) % kCapacity;
  }
  used_ += len;
  return true;
}

}  // namespace mousewheeld::protocol
