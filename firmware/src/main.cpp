// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Thin on purpose: the board is core/ and hal/. This file owns the two objects,
// starts the scan, and hands loop() to the session.

#include <Arduino.h>

#include "device.h"
#include "hal.h"
#include "protocol/session.h"
#include "scan.h"

namespace {

mousewheeld::Device device;
mousewheeld::protocol::Session session(device);

void scan() { mousewheeld::scan_once(device); }

}  // namespace

void setup() {
  using namespace mousewheeld;
  hal::init();
  attach_counters(device);
  session.boot(hal::micros_now());
  hal::start_scan(&scan);
}

void loop() { session.poll(mousewheeld::hal::micros_now()); }
