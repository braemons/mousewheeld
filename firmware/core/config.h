// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// Capacities and unit aliases. Everything here is compile-time and the ones a
// host can overflow are reported in HelloAck, so a zone set that does not fit
// is refused at upload naming what overflowed — never in the middle of a trial.
//
// The repeated-field limits in core/proto/link.options must agree with these;
// core/protocol/session.cpp static_asserts that they do.

#include <stddef.h>
#include <stdint.h>

#ifndef MOUSEWHEELD_MAX_AXES
#define MOUSEWHEELD_MAX_AXES 2
#endif

#ifndef MOUSEWHEELD_SCAN_HZ
#define MOUSEWHEELD_SCAN_HZ 5000
#endif

#ifndef MOUSEWHEELD_FIRMWARE_VERSION
#define MOUSEWHEELD_FIRMWARE_VERSION ""
#endif

namespace mousewheeld {

using Microseconds = uint64_t;
// Bit i is output line i.
using LineMask = uint8_t;

constexpr uint8_t kMaxAxes = MOUSEWHEELD_MAX_AXES;
constexpr uint8_t kMaxZones = 16;
constexpr uint8_t kMaxLines = 8;
// A zone-set name, without its terminator.
constexpr size_t kMaxNameLength = 32;

constexpr uint32_t kScanHz = MOUSEWHEELD_SCAN_HZ;
constexpr Microseconds kScanPeriodUs = 1000000 / kScanHz;

// Device velocity: counts over this window, reported as counts/s.
constexpr Microseconds kVelocityWindowUs = 20000;

// The longest frame on the wire in either direction, encoded, including the
// delimiter. Checked against the generated message sizes in framing.h.
constexpr size_t kMaxFrame = 256;

// Rings between the scan and loop(). Powers of two.
constexpr size_t kSampleRing = 256;
constexpr size_t kZoneHitRing = 32;

// How many ring entries one loop() pass may turn into frames. Rule 3: a backlog
// is drained in bounded steps, interleaved with the transmit drain.
constexpr size_t kDrainPerLoop = 8;

constexpr uint32_t kProtocolVersion = 1;

// A value no board firmware reports, so an unversioned build is recognisable.
constexpr const char* kFirmwareVersion =
    MOUSEWHEELD_FIRMWARE_VERSION[0] != '\0' ? MOUSEWHEELD_FIRMWARE_VERSION : "0.0.0";

static_assert(kMaxLines <= 8 * sizeof(LineMask), "a LineMask holds every line");
static_assert(kMaxZones <= 32, "StateReport.zones_live is a 32-bit mask");
static_assert(kMaxAxes >= 1, "a board with no axes measures nothing");

}  // namespace mousewheeld
