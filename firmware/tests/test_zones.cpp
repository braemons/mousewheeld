// SPDX-License-Identifier: AGPL-3.0-or-later
#include <initializer_list>
#include <string>

#include "doctest.h"
#include "zones/zone.h"
#include "zones/zone_evaluator.h"

using namespace mousewheeld;

namespace {

Zone rect(int64_t lo, int64_t hi, Fire fire = Fire::kOnce) {
  Zone zone;
  zone.n_intervals = 1;
  zone.intervals[0] = {0, true, true, lo, hi};
  zone.fire = fire;
  zone.action = Action::kPulse;
  zone.pulse_ms = 10;
  return zone;
}

ZoneSet set_of(std::initializer_list<Zone> zones) {
  ZoneSet set;
  for (const Zone& zone : zones) set.zones[set.n++] = zone;
  set.version = 1;
  set.committed = true;
  return set;
}

Metrics at(int64_t displacement, int64_t distance = 0) {
  Metrics m;
  m.displacement[0] = displacement;
  m.distance[0] = distance;
  return m;
}

}  // namespace

TEST_CASE("a zone fires on entry, not on the level") {
  ZoneEvaluator zones;
  CHECK(zones.arm(set_of({rect(100, 200)}), 1, at(0)) == 0);
  CHECK(zones.evaluate(at(99)) == 0);
  CHECK(zones.evaluate(at(100)) == 1);
  CHECK(zones.evaluate(at(150)) == 0);
}

TEST_CASE("once fires one time per arm") {
  ZoneEvaluator zones;
  zones.arm(set_of({rect(100, 200, Fire::kOnce)}), 1, at(0));
  CHECK(zones.evaluate(at(150)) == 1);
  CHECK(zones.evaluate(at(0)) == 0);
  CHECK(zones.evaluate(at(150)) == 0);
  CHECK(zones.live() == 0);
}

TEST_CASE("rearm fires again only after leaving by the hysteresis") {
  Zone zone = rect(100, 200, Fire::kRearm);
  zone.hysteresis = 10;
  ZoneEvaluator zones;
  zones.arm(set_of({zone}), 1, at(0));
  CHECK(zones.evaluate(at(100)) == 1);
  // Jitter at the boundary: out by less than the hysteresis and back.
  CHECK(zones.evaluate(at(95)) == 0);
  CHECK(zones.evaluate(at(100)) == 0);
  // Out by the hysteresis, then back in.
  CHECK(zones.evaluate(at(90)) == 0);
  CHECK(zones.evaluate(at(100)) == 1);
}

TEST_CASE("arming inside does not fire, unless level_on_arm") {
  ZoneEvaluator zones;
  CHECK(zones.arm(set_of({rect(0, 200)}), 1, at(50)) == 0);
  Zone eager = rect(0, 200);
  eager.level_on_arm = true;
  CHECK(zones.arm(set_of({eager}), 2, at(50)) == 1);
  CHECK(zones.evaluate(at(60)) == 0);
}

TEST_CASE("an open bound is open") {
  Zone zone = rect(1000, 0);
  zone.intervals[0].has_hi = false;
  ZoneEvaluator zones;
  zones.arm(set_of({zone}), 1, at(0));
  CHECK(zones.evaluate(at(1'000'000)) == 1);
}

TEST_CASE("a wrapped track evaluates modulo the period, and hysteresis goes the short way") {
  Zone zone = rect(0, 10, Fire::kRearm);
  zone.wrap = 300;
  zone.hysteresis = 5;
  ZoneEvaluator zones;
  zones.arm(set_of({zone}), 1, at(150));
  CHECK(zones.evaluate(at(305)) == 1);  // 305 mod 300 = 5
  // 297 is 3 below 0 the short way round: inside the hysteresis, still engaged.
  CHECK(zones.evaluate(at(297)) == 0);
  CHECK(zones.evaluate(at(301)) == 0);
  CHECK(zones.evaluate(at(290)) == 0);  // 10 away: disengaged
  CHECK(zones.evaluate(at(600)) == 1);
}

TEST_CASE("a distance zone reads the odometer") {
  Zone zone = rect(500, 0);
  zone.intervals[0].has_hi = false;
  zone.metric = Metric::kDistance;
  ZoneEvaluator zones;
  zones.arm(set_of({zone}), 1, at(0, 0));
  CHECK(zones.evaluate(at(-100, 499)) == 0);
  CHECK(zones.evaluate(at(-200, 500)) == 1);
}

TEST_CASE("a two-axis rect needs both axes inside") {
  Zone zone = rect(0, 10);
  zone.n_intervals = 2;
  zone.intervals[1] = {1, true, true, 20, 30};
  ZoneEvaluator zones;
  Metrics m;
  zones.arm(set_of({zone}), 1, m);
  m.displacement[0] = 5;
  CHECK(zones.evaluate(m) == 0);
  m.displacement[1] = 25;
  CHECK(zones.evaluate(m) == 1);
}

TEST_CASE("two zones in one scan both fire, in declaration order") {
  ZoneEvaluator zones;
  zones.arm(set_of({rect(0, 100), rect(50, 60)}), 1, at(-10));
  CHECK(zones.evaluate(at(55)) == 0b11);
}

TEST_CASE("a level zone holds its line while engaged") {
  Zone zone = rect(100, 200, Fire::kOnce);
  zone.action = Action::kLevel;
  zone.line = 3;
  ZoneEvaluator zones;
  zones.arm(set_of({zone}), 1, at(0));
  CHECK(zones.level_lines() == 0);
  zones.evaluate(at(150));
  CHECK(zones.level_lines() == 0b1000);
  zones.evaluate(at(250));
  CHECK(zones.level_lines() == 0);
  // Once limits the hit, not the level.
  zones.evaluate(at(150));
  CHECK(zones.level_lines() == 0b1000);
  zones.disarm();
  CHECK(zones.level_lines() == 0);
}

TEST_CASE("an upload commits only whole") {
  ZoneUpload upload;
  ZoneSet live;
  CHECK(upload.commit(live).code == std::string("no_upload"));
  REQUIRE_FALSE(upload.begin(7, 2, "goal"));
  REQUIRE_FALSE(upload.add(0, rect(0, 1)));
  const Refusal refusal = upload.commit(live);
  CHECK(refusal.code == std::string("incomplete_upload"));
  CHECK_FALSE(live.committed);
  REQUIRE_FALSE(upload.begin(7, 2, "goal"));
  REQUIRE_FALSE(upload.add(1, rect(0, 1)));
  REQUIRE_FALSE(upload.add(0, rect(0, 1)));
  REQUIRE_FALSE(upload.commit(live));
  CHECK(live.committed);
  CHECK(live.n == 2);
  CHECK(live.version == 7);
  CHECK(std::string(live.name) == "goal");
}

TEST_CASE("an upload that does not fit is refused naming the capacity") {
  ZoneUpload upload;
  const Refusal refusal = upload.begin(1, kMaxZones + 1, "big");
  CHECK(refusal.code == std::string("zone_table_full"));
  CHECK(std::string(refusal.detail).find("16") != std::string::npos);
}

TEST_CASE("check_zone refuses what the board cannot evaluate") {
  CHECK(check_zone(rect(0, 1), 1, 0b1).code == nullptr);
  CHECK(check_zone(rect(0, 1), 1, 0b0).code == std::string("bad_line"));
  CHECK(check_zone(rect(2, 1), 1, 0b1).code == std::string("bad_zone"));
  Zone other_axis = rect(0, 1);
  other_axis.intervals[0].axis = 1;
  CHECK(check_zone(other_axis, 1, 0b1).code == std::string("bad_axis"));
  Zone wrapped = rect(0, 400);
  wrapped.wrap = 300;
  CHECK(check_zone(wrapped, 1, 0b1).code == std::string("bad_zone"));
  Zone instant = rect(0, 1);
  instant.pulse_ms = 0;
  CHECK(check_zone(instant, 1, 0b1).code == std::string("bad_zone"));
}
