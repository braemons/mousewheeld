// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// The board: axes, zones, outputs and the rings, and the scan that moves them.
//
// Two contexts touch this object and the split is the design:
//
// - **scan()** runs from a timer at kScanHz. It reads counters, evaluates the
//   armed zones, starts and ends pulses, and pushes fixed-size records into
//   the rings. It formats nothing and never waits.
// - **Everything else** runs from loop(). A command changes what the scan
//   reads under a ScanLock, briefly, and never across I/O.

#include <stdint.h>

#include "config.h"
#include "output/analog.h"
#include "output/lines.h"
#include "position/axis_counter.h"
#include "refusal.h"
#include "stream/records.h"
#include "stream/ring.h"
#include "zones/zone.h"
#include "zones/zone_evaluator.h"

namespace mousewheeld {

struct AxisSetting {
  // Only quadrature is implemented; anything else is refused by name.
  bool quadrature = true;
  bool invert = false;
};

// What StateReport carries, copied out under the lock.
struct Snapshot {
  uint8_t n_axes;
  int64_t counts[kMaxAxes];
  int64_t origin[kMaxAxes];
  int64_t distance[kMaxAxes];
  int64_t velocity[kMaxAxes];
  uint32_t stream_rate_hz;
  bool armed;
  uint32_t arm_id;
  uint32_t zone_set_version;
  uint32_t zones_live;
  AnalogConfig analog;
  uint64_t ring_drops;
  uint64_t zone_hit_drops;
  uint64_t scan_overruns;
  uint32_t scan_max_us;
};

using SampleRing = Ring<SampleRecord, kSampleRing>;
using ZoneHitRing = Ring<ZoneHitRecord, kZoneHitRing>;

class Device {
 public:
  // Take the counters' current readings as the reference. Before the scan
  // starts.
  void attach(const int32_t raw[kMaxAxes]);

  // ------------------------------------------------------------ the scan ---
  void scan(Microseconds now, const int32_t raw[kMaxAxes], uint32_t modulus);
  // The wrapper that timed the scan says how long it took.
  void note_scan_duration(uint32_t us) {
    if (us > scan_max_us_) scan_max_us_ = us;
  }

  // ---------------------------------------------------- loop(): commands ---
  Refusal set_axes(const AxisSetting* axes, uint8_t n);
  Refusal set_line(uint8_t index, uint8_t pin, bool safe_high);
  Refusal set_stream(uint32_t rate_hz, bool velocity);
  Refusal begin_upload(uint32_t version, uint32_t n, const char* name);
  Refusal add_zone(uint32_t index, const Zone& zone);
  Refusal end_upload();
  Refusal arm(uint32_t arm_id, uint32_t version, bool origin_current, Microseconds now);
  Refusal disarm(uint32_t arm_id);
  // Bit i: axis i. 0 zeroes every axis.
  Refusal zero(uint32_t axes);
  Refusal set_analog(const AnalogConfig& config);

  // ---------------------------------------------------- loop(): the rest ---
  // The value for the DAC, from the latest counts.
  uint16_t analog_output() const;
  Snapshot snapshot() const;

  SampleRing& samples() { return samples_; }
  ZoneHitRing& zone_hits() { return zone_hits_; }

  uint8_t n_axes() const { return n_axes_; }
  bool velocity_on_samples() const { return velocity_on_samples_; }
  const ZoneSet& committed() const { return committed_; }
  const Lines& lines() const { return lines_; }
  const AxisCounter& axis(uint8_t index) const { return axes_[index]; }
  const AnalogConfig& analog() const { return analog_; }
  bool armed() const { return zones_.armed(); }
  uint32_t arm_id() const { return zones_.arm_id(); }

 private:
  Metrics metrics() const;
  void record_hits(uint32_t fired, Microseconds now);

  // Read by the scan; written by loop() under the lock.
  AxisCounter axes_[kMaxAxes];
  uint8_t n_axes_ = 1;
  ZoneEvaluator zones_;
  Lines lines_;
  uint32_t stream_rate_hz_ = 0;
  bool velocity_on_samples_ = false;

  // The scan's own.
  uint32_t stream_phase_ = 0;
  uint64_t sample_seq_ = 0;
  uint64_t zone_hit_seq_ = 0;
  Microseconds last_scan_ = 0;
  Microseconds window_start_ = 0;
  int64_t window_counts_[kMaxAxes] = {};
  int64_t velocity_[kMaxAxes] = {};
  uint64_t scan_overruns_ = 0;
  uint32_t scan_max_us_ = 0;

  // loop()'s own: the scan never reads these.
  ZoneUpload upload_;
  ZoneSet committed_;
  AnalogConfig analog_;

  SampleRing samples_;
  ZoneHitRing zone_hits_;
};

}  // namespace mousewheeld
