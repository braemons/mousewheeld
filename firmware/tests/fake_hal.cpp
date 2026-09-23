// SPDX-License-Identifier: AGPL-3.0-or-later
#include "fake_hal.h"

#include <cstring>

#include "hal.h"

namespace mousewheeld::fake {

Microseconds now = 0;
std::deque<uint8_t> rx;
std::vector<uint8_t> tx;
LineMask high = 0;
uint8_t pins[kMaxLines] = {};
std::vector<uint8_t> flash;
bool flash_fails = false;
uint16_t analog = 0;
uint32_t modulus = 65536;

void reset() {
  now = 0;
  rx.clear();
  tx.clear();
  high = 0;
  memset(pins, 0, sizeof(pins));
  flash.clear();
  flash_fails = false;
  analog = 0;
  modulus = 65536;
}

}  // namespace mousewheeld::fake

namespace mousewheeld::hal {

void init() {}
Microseconds micros_now() { return fake::now; }
int32_t read_counter(uint8_t) { return 0; }
uint32_t counter_modulus() { return fake::modulus; }
bool pin_is_output(uint8_t pin) { return pin < 40; }

void configure_line(uint8_t index, uint8_t pin, bool level) {
  fake::pins[index] = pin;
  if (level) {
    fake::high |= static_cast<LineMask>(1u << index);
  } else {
    fake::high &= static_cast<LineMask>(~(1u << index));
  }
}

void write_outputs(LineMask high, LineMask low) {
  fake::high = static_cast<LineMask>((fake::high | high) & ~low);
}

void write_analog(uint16_t value) { fake::analog = value; }

size_t serial_read(uint8_t* buf, size_t len) {
  size_t n = 0;
  while (n < len && !fake::rx.empty()) {
    buf[n++] = fake::rx.front();
    fake::rx.pop_front();
  }
  return n;
}

size_t serial_write(const uint8_t* buf, size_t len) {
  fake::tx.insert(fake::tx.end(), buf, buf + len);
  return len;
}

size_t serial_writable() { return 1 << 20; }

size_t flash_load(uint8_t* buf, size_t len) {
  if (fake::flash.empty()) return 0;
  if (fake::flash.size() > len) return fake::flash.size();
  memcpy(buf, fake::flash.data(), fake::flash.size());
  return fake::flash.size();
}

bool flash_save(const uint8_t* buf, size_t len) {
  if (fake::flash_fails) return false;
  fake::flash.assign(buf, buf + len);
  return true;
}

void start_scan(void (*)()) {}
void stop_scan() {}
void lock_scan() {}
void unlock_scan() {}

}  // namespace mousewheeld::hal
