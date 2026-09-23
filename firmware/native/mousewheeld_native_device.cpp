// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The firmware as a host process, speaking the link on stdin/stdout. Run it on
// the far end of a pty and the daemon cannot tell it from a board — which is
// the point: the integration tests then exercise this firmware, not a model of
// it. src/main.cpp, without Arduino.

#include <chrono>
#include <thread>

#include "device.h"
#include "hal.h"
#include "protocol/session.h"
#include "scan.h"

namespace {

mousewheeld::Device device;
mousewheeld::protocol::Session session(device);

void scan() { mousewheeld::scan_once(device); }

}  // namespace

int main() {
  using namespace mousewheeld;
  hal::init();
  attach_counters(device);
  session.boot(hal::micros_now());
  hal::start_scan(&scan);
  for (;;) {
    session.poll(hal::micros_now());
    std::this_thread::sleep_for(std::chrono::microseconds(200));
  }
}
