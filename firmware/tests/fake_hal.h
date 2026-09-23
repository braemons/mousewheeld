// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// A HAL the tests hold the other end of: bytes in and out of the "serial
// port", the line levels as written, and a flash that is a vector.

#include <cstdint>
#include <deque>
#include <vector>

#include "config.h"

namespace mousewheeld::fake {

extern Microseconds now;
extern std::deque<uint8_t> rx;
extern std::vector<uint8_t> tx;
// Physical level per line, as the last write_outputs left it.
extern LineMask high;
extern uint8_t pins[kMaxLines];
extern std::vector<uint8_t> flash;
extern bool flash_fails;
extern uint16_t analog;
extern uint32_t modulus;

void reset();

}  // namespace mousewheeld::fake
