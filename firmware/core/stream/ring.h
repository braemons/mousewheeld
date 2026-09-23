// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// Single producer — the scan — and single consumer — loop(). A full ring drops
// the newest entry and counts it, which is what keeps it lock-free with an
// interrupt at one end: the producer never waits. The drop is visible to the
// host twice, in StateReport and as a gap in `seq`.

#include <atomic>
#include <stddef.h>
#include <stdint.h>

namespace mousewheeld {

template <typename T, size_t N>
class Ring {
  static_assert(N >= 2 && (N & (N - 1)) == 0, "a power of two");

 public:
  // Producer only.
  bool push(const T& item) {
    const size_t head = head_.load(std::memory_order_relaxed);
    if (head - tail_.load(std::memory_order_acquire) == N) {
      drops_.store(drops_.load(std::memory_order_relaxed) + 1, std::memory_order_relaxed);
      return false;
    }
    items_[head & (N - 1)] = item;
    head_.store(head + 1, std::memory_order_release);
    return true;
  }

  // Consumer only.
  bool pop(T& item) {
    const size_t tail = tail_.load(std::memory_order_relaxed);
    if (tail == head_.load(std::memory_order_acquire)) return false;
    item = items_[tail & (N - 1)];
    tail_.store(tail + 1, std::memory_order_release);
    return true;
  }

  bool empty() const {
    return tail_.load(std::memory_order_acquire) == head_.load(std::memory_order_acquire);
  }

  uint32_t drops() const { return drops_.load(std::memory_order_relaxed); }

 private:
  T items_[N];
  std::atomic<size_t> head_{0};
  std::atomic<size_t> tail_{0};
  std::atomic<uint32_t> drops_{0};
};

}  // namespace mousewheeld
