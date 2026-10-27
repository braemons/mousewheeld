// SPDX-License-Identifier: AGPL-3.0-or-later
#include <string>

#include "device.h"
#include "doctest.h"
#include "fake_hal.h"

using namespace mousewheeld;

namespace {

struct Bench {
  Device device;
  int32_t raw[kMaxAxes] = {};
  Microseconds now = 1000;

  Bench() {
    fake::reset();
    device.attach(raw);
  }

  void scans(int n, int32_t step = 0) {
    for (int i = 0; i < n; ++i) {
      now += kScanPeriodUs;
      raw[0] = static_cast<int16_t>(raw[0] + step);
      device.scan(now, raw, 65536);
    }
  }

  void goal_zone(int64_t lo, uint16_t ms = 10) {
    REQUIRE_FALSE(device.set_line(0, 12, false));
    REQUIRE_FALSE(device.begin_upload(3, 1, "goal"));
    Zone zone;
    zone.n_intervals = 1;
    zone.intervals[0] = {0, true, false, lo, 0};
    zone.pulse_ms = ms;
    REQUIRE_FALSE(device.add_zone(0, zone));
    REQUIRE_FALSE(device.end_upload());
  }
};

}  // namespace

TEST_CASE("the stream runs at its rate whether or not the wheel moves") {
  Bench b;
  REQUIRE_FALSE(b.device.set_stream(200, false));
  b.scans(static_cast<int>(kScanHz));  // one second
  size_t n = 0;
  uint64_t expected_seq = 0;
  SampleRecord sample;
  while (b.device.samples().pop(sample)) {
    CHECK(sample.seq == expected_seq++);
    ++n;
  }
  // The ring holds 256, and 200 went in.
  CHECK(n == 200);
}

TEST_CASE("a full ring drops the newest and the gap shows in seq") {
  Bench b;
  REQUIRE_FALSE(b.device.set_stream(kScanHz, false));
  b.scans(static_cast<int>(kSampleRing) + 10);
  CHECK(b.device.snapshot().ring_drops == 10);
  SampleRecord sample;
  for (size_t i = 0; i < kSampleRing; ++i) REQUIRE(b.device.samples().pop(sample));
  b.scans(1);
  REQUIRE(b.device.samples().pop(sample));
  CHECK(sample.seq == kSampleRing + 10);
}

TEST_CASE("a rate above the scan is refused") {
  Bench b;
  CHECK(b.device.set_stream(kScanHz + 1, false).code == std::string("bad_rate"));
}

TEST_CASE("samples carry cumulative counts, and velocity in counts per second") {
  Bench b;
  REQUIRE_FALSE(b.device.set_stream(100, true));
  // 2 counts per scan at 5 kHz: 10000 counts/s.
  b.scans(static_cast<int>(kScanHz), 2);
  SampleRecord sample, last{};
  while (b.device.samples().pop(sample)) last = sample;
  CHECK(last.counts[0] == 2 * static_cast<int64_t>(kScanHz));
  CHECK(last.velocity[0] == 2 * static_cast<int64_t>(kScanHz));
}

TEST_CASE("arming with origin current zeroes displacement, and the zone pulses from the scan") {
  Bench b;
  b.goal_zone(100, 10);
  b.scans(1, 500);  // somewhere else on the track first
  REQUIRE_FALSE(b.device.arm(9, 3, true, b.now));
  CHECK(b.device.axis(0).displacement() == 0);
  CHECK(b.device.axis(0).counts() == 500);

  b.scans(99, 1);
  CHECK(fake::high == 0);
  b.scans(1, 1);  // displacement 100: in
  CHECK(fake::high == 0b1);
  ZoneHitRecord hit;
  REQUIRE(b.device.zone_hits().pop(hit));
  CHECK(hit.arm_id == 9);
  CHECK(hit.zone == 0);
  CHECK(hit.seq == 0);
  CHECK(hit.counts[0] == 600);

  // The pulse ends 10 ms later, from the scan, with the wheel standing still.
  b.scans(static_cast<int>(10000 / kScanPeriodUs) - 1);
  CHECK(fake::high == 0b1);
  b.scans(1);
  CHECK(fake::high == 0);
}

TEST_CASE("a velocity zone's line falls when the wheel stops, with nothing moving") {
  Bench b;
  REQUIRE_FALSE(b.device.set_line(0, 12, false));
  REQUIRE_FALSE(b.device.begin_upload(4, 1, "speed"));
  Zone zone;
  zone.n_intervals = 1;
  // 2 counts per scan is 2 * kScanHz counts/s; ask for half of it.
  zone.intervals[0] = {0, true, false, static_cast<int64_t>(kScanHz), 0};
  zone.metric = Metric::kVelocity;
  zone.fire = Fire::kRearm;
  zone.action = Action::kLevel;
  zone.hysteresis = static_cast<int64_t>(kScanHz) / 10;
  REQUIRE_FALSE(b.device.add_zone(0, zone));
  REQUIRE_FALSE(b.device.end_upload());
  REQUIRE_FALSE(b.device.arm(1, 4, true, b.now));

  const int window = static_cast<int>(kVelocityWindowUs / kScanPeriodUs);
  b.scans(2 * window, 2);
  CHECK(fake::high == 0b1);
  b.scans(2 * window);
  CHECK(fake::high == 0);
}

TEST_CASE("arm refuses a version the board does not hold") {
  Bench b;
  CHECK(b.device.arm(1, 3, true, b.now).code == std::string("no_zone_set"));
  b.goal_zone(100);
  CHECK(b.device.arm(1, 2, true, b.now).code == std::string("version_mismatch"));
}

TEST_CASE("disarm drives the lines safe, and names a stranger's arm_id") {
  Bench b;
  REQUIRE_FALSE(b.device.set_line(1, 13, true));  // idles high
  CHECK(fake::high == 0b10);
  REQUIRE_FALSE(b.device.begin_upload(1, 1, "level"));
  Zone zone;
  zone.n_intervals = 1;
  zone.intervals[0] = {0, true, false, 10, 0};
  zone.action = Action::kLevel;
  zone.line = 1;
  REQUIRE_FALSE(b.device.add_zone(0, zone));
  REQUIRE_FALSE(b.device.end_upload());
  REQUIRE_FALSE(b.device.arm(4, 1, true, b.now));
  b.scans(1, 20);
  CHECK(fake::high == 0);  // active is low on a line that idles high
  CHECK(b.device.disarm(5).code == std::string("wrong_arm_id"));
  REQUIRE_FALSE(b.device.disarm(4));
  CHECK(fake::high == 0b10);
}

TEST_CASE("an unsupported source is refused naming the axis") {
  Bench b;
  AxisSetting axes[2];
  axes[1].quadrature = false;
  const Refusal refusal = b.device.set_axes(axes, 2);
  CHECK(refusal.code == std::string("unsupported_source"));
  CHECK(std::string(refusal.detail).find("axis 1") != std::string::npos);
}

TEST_CASE("zero moves the origin of the named axes only") {
  Bench b;
  AxisSetting axes[2];
  REQUIRE_FALSE(b.device.set_axes(axes, 2));
  b.raw[1] = 50;
  b.scans(1, 7);
  REQUIRE_FALSE(b.device.zero(0b10));
  CHECK(b.device.axis(0).displacement() == 7);
  CHECK(b.device.axis(1).displacement() == 0);
  CHECK(b.device.axis(1).counts() == 50);
  CHECK(b.device.zero(0b100).code == std::string("bad_axis"));
}

TEST_CASE("injected motion is counted, fires the zones, says so on the samples, and stops on time") {
  Bench b;
  b.goal_zone(500);
  REQUIRE_FALSE(b.device.arm(1, 3, true, b.now));
  REQUIRE_FALSE(b.device.set_stream(10, true));
  // 1500 counts a second for half a second, with the wheel itself still.
  REQUIRE_FALSE(b.device.inject(0, 1500, 500, b.now));
  b.scans(static_cast<int>(kScanHz) / 2 + 5);
  ZoneHitRecord hit;
  REQUIRE(b.device.zone_hits().pop(hit));
  CHECK(hit.counts[0] == 500);
  b.scans(10);
  const int64_t counted = b.device.snapshot().counts[0];
  CHECK(counted >= 745);
  CHECK(counted <= 755);
  b.scans(static_cast<int>(kScanHz) / 2);
  CHECK(b.device.snapshot().counts[0] == counted);

  SampleRecord sample;
  REQUIRE(b.device.samples().pop(sample));
  CHECK(sample.injecting == 0b1);
  SampleRecord last = sample;
  while (b.device.samples().pop(sample)) last = sample;
  CHECK(last.injecting == 0);
  CHECK(last.velocity[0] == 0);
}

TEST_CASE("an injection can be ended, backwards is backwards, and nonsense is refused") {
  Bench b;
  REQUIRE_FALSE(b.device.inject(0, -2000, 10000, b.now));
  b.scans(static_cast<int>(kScanHz) / 10);
  CHECK(b.device.snapshot().counts[0] < -150);
  REQUIRE_FALSE(b.device.inject(0, 0, 0, b.now));
  const int64_t stopped = b.device.snapshot().counts[0];
  b.scans(100);
  CHECK(b.device.snapshot().counts[0] == stopped);

  CHECK(b.device.inject(1, 100, 100, b.now).code == std::string("bad_axis"));
  CHECK(b.device.inject(0, 2000000, 100, b.now).code == std::string("bad_injection"));
  CHECK(b.device.inject(0, 100, 600001, b.now).code == std::string("bad_injection"));
}
