// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// The hardware, behind the smallest interface the core needs. One
// implementation per board in hal/, each guarded by its architecture macro.
//
// The core never includes a vendor header. That is what lets it build under
// CMake on the host, where the tests are.

#include <stddef.h>
#include <stdint.h>

#include "config.h"

namespace mousewheeld::hal {

// Outputs to their safe level before anything else, then counters, then the
// link. Does not start the scan.
void init();

Microseconds micros_now();

// The hardware counter for `axis`, **congruent to the true count modulo
// counter_modulus()**. It may wrap or reset at a limit — the core extends it to
// 64 bits by differencing successive reads, which is exact as long as an axis
// moves by less than half the modulus between two scans.
int32_t read_counter(uint8_t axis);
uint32_t counter_modulus();

// Whether `pin` can drive an output line on this board.
bool pin_is_output(uint8_t pin);
// Bind output line `index` to `pin` and drive it to `level`.
void configure_line(uint8_t index, uint8_t pin, bool level);
// Physical levels: lines in `high` go high, lines in `low` go low, the rest
// are untouched.
void write_outputs(LineMask high, LineMask low);

// 0 is the bottom of the range and 65535 the top; a board scales to its DAC.
void write_analog(uint16_t value);

size_t serial_read(uint8_t* buf, size_t len);
size_t serial_write(const uint8_t* buf, size_t len);
// Bytes serial_write() would take now without blocking.
size_t serial_writable();

// The settings blob (see core/settings.h). load returns the length read, 0 if
// there is none.
size_t flash_load(uint8_t* buf, size_t len);
bool flash_save(const uint8_t* buf, size_t len);

// Call `scan` at kScanHz from a timer, until stop_scan().
void start_scan(void (*scan)());
void stop_scan();

// Exclude the scan. loop() holds it while it changes what the scan reads; the
// scan holds it for its whole body. Short, and never across I/O.
void lock_scan();
void unlock_scan();

}  // namespace mousewheeld::hal

namespace mousewheeld {

class ScanLock {
 public:
  ScanLock() { hal::lock_scan(); }
  ~ScanLock() { hal::unlock_scan(); }
  ScanLock(const ScanLock&) = delete;
  ScanLock& operator=(const ScanLock&) = delete;
};

}  // namespace mousewheeld
