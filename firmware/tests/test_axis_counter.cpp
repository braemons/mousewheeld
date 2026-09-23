// SPDX-License-Identifier: AGPL-3.0-or-later
#include "doctest.h"
#include "position/axis_counter.h"

using mousewheeld::AxisCounter;

TEST_CASE("a 16-bit counter is extended across its wrap, both ways") {
  AxisCounter axis;
  axis.attach(32760);
  int32_t raw = 32760;
  // Up through the top: 32760 → -32768 … as the int16 reading goes.
  for (int i = 0; i < 20; ++i) {
    raw = static_cast<int16_t>(raw + 1);
    axis.update(raw, 65536);
  }
  CHECK(axis.counts() == 20);
  for (int i = 0; i < 50; ++i) {
    raw = static_cast<int16_t>(raw - 1);
    axis.update(raw, 65536);
  }
  CHECK(axis.counts() == -30);
  CHECK(axis.distance() == 70);
}

TEST_CASE("a counter that resets to zero at a limit is congruent modulo the limit") {
  // PCNT: counting up to +32000 resets to 0; down to -32000 resets to 0.
  const int32_t limit = 32000;
  AxisCounter axis;
  int32_t hw = 31990;
  axis.attach(hw);
  int64_t truth = 0;
  for (int i = 0; i < 25; ++i) {
    hw += 1;
    if (hw == limit) hw = 0;
    ++truth;
    axis.update(hw, limit);
  }
  CHECK(axis.counts() == truth);
  for (int i = 0; i < 64100; ++i) {
    hw -= 1;
    if (hw == -limit) hw = 0;
    --truth;
    axis.update(hw, limit);
  }
  CHECK(axis.counts() == truth);
}

TEST_CASE("several counts between two scans are folded in whole") {
  AxisCounter axis;
  axis.attach(0);
  CHECK(axis.update(1000, 65536));
  CHECK(axis.counts() == 1000);
  CHECK_FALSE(axis.update(1000, 65536));
}

TEST_CASE("invert flips the direction, and zero moves the origin, never the counts") {
  AxisCounter axis;
  axis.set_invert(true);
  axis.attach(0);
  axis.update(10, 65536);
  CHECK(axis.counts() == -10);
  axis.zero();
  CHECK(axis.counts() == -10);
  CHECK(axis.displacement() == 0);
  CHECK(axis.distance() == 0);
  axis.update(15, 65536);
  CHECK(axis.displacement() == -5);
  CHECK(axis.distance() == 5);
}
