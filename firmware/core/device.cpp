// SPDX-License-Identifier: AGPL-3.0-or-later
#include "device.h"

#include "hal.h"

namespace mousewheeld {

void Device::attach(const int32_t raw[kMaxAxes]) {
  for (uint8_t a = 0; a < kMaxAxes; ++a) axes_[a].attach(raw[a]);
}

Metrics Device::metrics() const {
  Metrics m;
  for (uint8_t a = 0; a < kMaxAxes; ++a) {
    m.displacement[a] = axes_[a].displacement();
    m.distance[a] = axes_[a].distance();
  }
  return m;
}

void Device::record_hits(uint32_t fired, Microseconds now) {
  for (uint8_t i = 0; fired != 0; ++i, fired >>= 1) {
    if (!(fired & 1u)) continue;
    const Zone& zone = zones_.zone(i);
    // The TTL is scheduled here, in the scan that saw the count. The record
    // that follows is only the record: a dropped zone_hit is a lost line in a
    // log, never a lost pulse.
    if (zone.action == Action::kPulse) lines_.pulse(zone.line, now, zone.pulse_ms);
    ZoneHitRecord hit;
    hit.seq = zone_hit_seq_++;
    hit.t_us = now;
    hit.arm_id = zones_.arm_id();
    hit.zone = i;
    for (uint8_t a = 0; a < kMaxAxes; ++a) hit.counts[a] = axes_[a].counts();
    zone_hits_.push(hit);
  }
}

void Device::scan(Microseconds now, const int32_t raw[kMaxAxes], uint32_t modulus) {
  if (last_scan_ != 0 && now - last_scan_ > kScanPeriodUs + kScanPeriodUs / 2) ++scan_overruns_;
  last_scan_ = now;

  bool moved = false;
  for (uint8_t a = 0; a < n_axes_; ++a) moved |= axes_[a].update(raw[a], modulus);

  // Velocity over a fixed window, whether or not anything moved: a wheel that
  // stopped has to read zero.
  if (now - window_start_ >= kVelocityWindowUs) {
    const int64_t elapsed = static_cast<int64_t>(now - window_start_);
    for (uint8_t a = 0; a < n_axes_; ++a) {
      velocity_[a] = (axes_[a].counts() - window_counts_[a]) * 1000000 / elapsed;
      window_counts_[a] = axes_[a].counts();
    }
    window_start_ = now;
  }

  // The stream runs at its rate whether or not anything moved: the host reads
  // time off it as well as counts.
  if (stream_rate_hz_ != 0) {
    stream_phase_ += stream_rate_hz_;
    if (stream_phase_ >= kScanHz) {
      stream_phase_ -= kScanHz;
      SampleRecord sample;
      // Consumed even when the ring is full, so the loss shows as a gap.
      sample.seq = sample_seq_++;
      sample.t_us = now;
      for (uint8_t a = 0; a < kMaxAxes; ++a) {
        sample.counts[a] = axes_[a].counts();
        sample.velocity[a] = velocity_[a];
      }
      samples_.push(sample);
    }
  }

  // Rule 4: an animal sitting still costs a compare per axis. Pulses still end
  // on time.
  if (moved && zones_.armed()) {
    record_hits(zones_.evaluate(metrics()), now);
    lines_.set_levels(zones_.level_lines());
  }
  lines_.tick(now);
}

Refusal Device::set_axes(const AxisSetting* axes, uint8_t n) {
  if (n == 0 || n > kMaxAxes) {
    return Refusal::because("bad_axis", "%u axes, and this board has %u", n, kMaxAxes);
  }
  for (uint8_t a = 0; a < n; ++a) {
    if (!axes[a].quadrature) {
      return Refusal::because("unsupported_source", "axis %u: only quadrature is implemented", a);
    }
  }
  if (zones_.armed()) return Refusal::because("busy", "disarm before changing the axes");
  ScanLock lock;
  n_axes_ = n;
  for (uint8_t a = 0; a < n; ++a) axes_[a].set_invert(axes[a].invert);
  return Refusal::none();
}

Refusal Device::set_line(uint8_t index, uint8_t pin, bool safe_high) {
  if (index >= kMaxLines) {
    return Refusal::because("bad_line", "line %u, and this board has %u", index, kMaxLines);
  }
  if (!hal::pin_is_output(pin)) {
    return Refusal::because("bad_pin", "pin %u cannot drive an output", pin);
  }
  if (zones_.armed()) return Refusal::because("busy", "disarm before changing the lines");
  ScanLock lock;
  lines_.configure(index, pin, safe_high);
  return Refusal::none();
}

Refusal Device::set_stream(uint32_t rate_hz, bool velocity) {
  if (rate_hz > kScanHz) {
    return Refusal::because("bad_rate", "%u Hz, above the %u Hz scan",
                            static_cast<unsigned>(rate_hz), static_cast<unsigned>(kScanHz));
  }
  ScanLock lock;
  stream_rate_hz_ = rate_hz;
  stream_phase_ = 0;
  velocity_on_samples_ = velocity;
  return Refusal::none();
}

Refusal Device::begin_upload(uint32_t version, uint32_t n, const char* name) {
  return upload_.begin(version, n, name);
}

Refusal Device::add_zone(uint32_t index, const Zone& zone) {
  if (Refusal refusal = check_zone(zone, n_axes_, lines_.configured())) return refusal;
  return upload_.add(index, zone);
}

Refusal Device::end_upload() { return upload_.commit(committed_); }

Refusal Device::arm(uint32_t arm_id, uint32_t version, bool origin_current, Microseconds now) {
  if (!committed_.committed) return Refusal::because("no_zone_set", "nothing has been uploaded");
  if (version != committed_.version) {
    return Refusal::because("version_mismatch", "the board holds version %u",
                            static_cast<unsigned>(committed_.version));
  }
  ScanLock lock;
  if (origin_current) {
    for (uint8_t a = 0; a < kMaxAxes; ++a) axes_[a].zero();
  }
  lines_.safe();
  record_hits(zones_.arm(committed_, arm_id, metrics()), now);
  lines_.set_levels(zones_.level_lines());
  lines_.tick(now);
  return Refusal::none();
}

Refusal Device::disarm(uint32_t arm_id) {
  if (zones_.armed() && arm_id != zones_.arm_id()) {
    return Refusal::because("wrong_arm_id", "armed as %u", static_cast<unsigned>(zones_.arm_id()));
  }
  ScanLock lock;
  zones_.disarm();
  lines_.safe();
  return Refusal::none();
}

Refusal Device::zero(uint32_t axes) {
  if (axes >> n_axes_) {
    return Refusal::because("bad_axis", "%u axes are configured", n_axes_);
  }
  if (axes == 0) axes = (1u << n_axes_) - 1;
  ScanLock lock;
  for (uint8_t a = 0; a < n_axes_; ++a) {
    if (axes & (1u << a)) axes_[a].zero();
  }
  return Refusal::none();
}

Refusal Device::set_analog(const AnalogConfig& config) {
  if (Refusal refusal = check_analog(config, n_axes_)) return refusal;
  analog_ = config;
  return Refusal::none();
}

uint16_t Device::analog_output() const {
  int64_t displacement;
  int64_t velocity;
  {
    ScanLock lock;
    displacement = axes_[analog_.axis].displacement();
    velocity = velocity_[analog_.axis];
  }
  return analog_value(analog_, displacement, velocity);
}

Snapshot Device::snapshot() const {
  Snapshot s{};
  ScanLock lock;
  s.n_axes = n_axes_;
  for (uint8_t a = 0; a < kMaxAxes; ++a) {
    s.counts[a] = axes_[a].counts();
    s.origin[a] = axes_[a].origin();
    s.distance[a] = axes_[a].distance();
    s.velocity[a] = velocity_[a];
  }
  s.stream_rate_hz = stream_rate_hz_;
  s.armed = zones_.armed();
  s.arm_id = zones_.arm_id();
  s.zone_set_version = zones_.version();
  s.zones_live = zones_.live();
  s.analog = analog_;
  s.ring_drops = samples_.drops();
  s.zone_hit_drops = zone_hits_.drops();
  s.scan_overruns = scan_overruns_;
  s.scan_max_us = scan_max_us_;
  return s;
}

}  // namespace mousewheeld
