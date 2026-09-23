// SPDX-License-Identifier: AGPL-3.0-or-later
#include "protocol/session.h"

#include <stdio.h>
#include <string.h>

#include <pb_decode.h>
#include <pb_encode.h>

#include "hal.h"
#include "settings.h"

namespace mousewheeld::protocol {
namespace {

// The generated structs are sized by core/proto/link.options, and the core by
// config.h. They are two files saying one thing, so the build checks they do.
template <typename T, size_t N>
constexpr size_t count_of(T (&)[N]) {
  return N;
}
constexpr mousewheeld_link_v1_Sample kSample{};
constexpr mousewheeld_link_v1_Zone kZone{};
constexpr mousewheeld_link_v1_Axes kAxes{};
constexpr mousewheeld_link_v1_Lines kLines{};
constexpr mousewheeld_link_v1_ZonesBegin kZonesBegin{};
static_assert(count_of(kSample.c) == kMaxAxes, "link.options: Sample.c max_count is kMaxAxes");
static_assert(count_of(kZone.intervals) == kMaxAxes, "link.options: Zone.intervals is kMaxAxes");
static_assert(count_of(kAxes.axes) == kMaxAxes, "link.options: Axes.axes is kMaxAxes");
static_assert(count_of(kLines.lines) == kMaxLines, "link.options: Lines.lines is kMaxLines");
static_assert(sizeof(kZonesBegin.name) == kMaxNameLength + 1, "link.options: name is kMaxNameLength");
static_assert(mousewheeld_link_v1_HostMessage_size <= kMaxPayload, "a HostMessage fits a frame");
static_assert(mousewheeld_link_v1_DeviceMessage_size <= kMaxPayload, "a DeviceMessage fits a frame");

// A reply must always fit, so records stop being queued while less than this is
// free. Replies come one per command and commands one per frame read.
constexpr size_t kReplyHeadroom = 2 * kMaxFrame;

constexpr Microseconds kAnalogPeriodUs = 1000;
constexpr Microseconds kDebugPeriodUs = 100000;

#if defined(MOUSEWHEELD_BOARD_NAME)
constexpr const char* kBoard = MOUSEWHEELD_BOARD_NAME;
#else
constexpr const char* kBoard = "unknown";
#endif

void copy_string(char* to, size_t size, const char* from) {
  strncpy(to, from, size - 1);
  to[size - 1] = '\0';
}

Zone zone_from_wire(const mousewheeld_link_v1_Zone& wire, Refusal& refusal) {
  Zone zone;
  zone.n_intervals = static_cast<uint8_t>(wire.intervals_count);
  for (pb_size_t i = 0; i < wire.intervals_count; ++i) {
    const auto& interval = wire.intervals[i];
    if (interval.axis >= kMaxAxes) {
      refusal = Refusal::because("bad_axis", "axis %u", static_cast<unsigned>(interval.axis));
      return zone;
    }
    zone.intervals[i].axis = static_cast<uint8_t>(interval.axis);
    zone.intervals[i].has_lo = interval.has_lo;
    zone.intervals[i].lo = interval.lo;
    zone.intervals[i].has_hi = interval.has_hi;
    zone.intervals[i].hi = interval.hi;
  }
  if (wire.metric > mousewheeld_link_v1_Metric_METRIC_DISTANCE ||
      wire.fire > mousewheeld_link_v1_Fire_FIRE_REARM ||
      wire.action > mousewheeld_link_v1_Action_ACTION_LEVEL) {
    // A value from a newer protocol. Evaluating half of a zone is worse than
    // refusing it.
    refusal = Refusal::because("bad_zone", "a metric, fire or action this board does not know");
    return zone;
  }
  if (wire.line >= kMaxLines || wire.pulse_ms > 0xFFFF) {
    refusal = Refusal::because("bad_zone", "line or pulse_ms out of range");
    return zone;
  }
  zone.metric = static_cast<Metric>(wire.metric);
  zone.fire = static_cast<Fire>(wire.fire);
  zone.action = static_cast<Action>(wire.action);
  zone.level_on_arm = wire.level_on_arm;
  zone.line = static_cast<uint8_t>(wire.line);
  zone.pulse_ms = static_cast<uint16_t>(wire.pulse_ms);
  zone.wrap = wire.wrap;
  zone.hysteresis = wire.hysteresis;
  return zone;
}

mousewheeld_link_v1_Analog analog_to_wire(const AnalogConfig& config) {
  mousewheeld_link_v1_Analog wire = mousewheeld_link_v1_Analog_init_zero;
  wire.axis = config.axis;
  wire.mode = static_cast<mousewheeld_link_v1_AnalogMode>(config.mode);
  wire.range_counts = config.range_counts;
  wire.lo = config.lo;
  wire.hi = config.hi;
  return wire;
}

}  // namespace

// ------------------------------------------------------------------ boot ---

void Session::boot(Microseconds now) {
  static uint8_t blob[sizeof(Settings)];
  const size_t len = hal::flash_load(blob, sizeof(blob));
  const char* restored = nullptr;
  char text[sizeof(mousewheeld_link_v1_Log{}.text)];

  if (len != 0 && !settings_valid(blob, len)) {
    restored = "saved settings are from another firmware; booting unconfigured";
  } else if (len != 0) {
    Settings settings;
    memcpy(static_cast<void*>(&settings), blob, sizeof(settings));
    if (settings.zones.committed) {
      flashed_ = true;
      flashed_version_ = settings.zones.version;
      copy_string(flashed_name_, sizeof(flashed_name_), settings.zones.name);
    }
    const Refusal refusal = restore_settings(device_, settings, now);
    if (refusal) {
      snprintf(text, sizeof(text), "saved settings refused: %s, %s", refusal.code, refusal.detail);
    } else {
      snprintf(text, sizeof(text), "restored saved settings%s%s%s",
               settings.zones.committed ? ", zone set " : "",
               settings.zones.committed ? settings.zones.name : "",
               settings.armed ? ", armed" : "");
    }
    restored = text;
  }

  send_hello_ack();
  if (restored != nullptr) send_log(restored);
}

// ------------------------------------------------------------------ loop ---

void Session::poll(Microseconds now) {
  read_link(now);

  if (debug_) {
    // The records are not wanted in debug mode, but the rings must not fill
    // and count drops that were never drops on the wire.
    SampleRecord sample;
    while (device_.samples().pop(sample)) {
    }
    ZoneHitRecord hit;
    while (device_.zone_hits().pop(hit)) {
    }
    write_debug(now);
  } else {
    drain_rings();
  }

  if (device_.analog().mode != AnalogMode::kOff && now >= next_analog_) {
    next_analog_ = now + kAnalogPeriodUs;
    hal::write_analog(device_.analog_output());
  }

  queue_.drain([](const uint8_t* bytes, size_t len) {
    const size_t room = hal::serial_writable();
    return hal::serial_write(bytes, len < room ? len : room);
  });
}

void Session::read_link(Microseconds now) {
  uint8_t byte;
  while (queue_.free() >= kReplyHeadroom && hal::serial_read(&byte, 1) == 1) {
    switch (reader_.push(byte)) {
      case FrameReader::Result::kPending:
        break;
      case FrameReader::Result::kRefused:
        // Never acted on, not even partially. There is no message_id to name:
        // nothing in a frame that failed its check can be believed.
        ++frames_refused_;
        if (!debug_) send_error(reader_.refusal(), "the frame did not survive the wire", nullptr);
        break;
      case FrameReader::Result::kFrame: {
        mousewheeld_link_v1_HostMessage message = mousewheeld_link_v1_HostMessage_init_zero;
        pb_istream_t stream = pb_istream_from_buffer(reader_.payload(), reader_.size());
        if (!pb_decode(&stream, mousewheeld_link_v1_HostMessage_fields, &message)) {
          ++frames_refused_;
          send_error("bad_message", PB_GET_ERROR(&stream), nullptr);
          break;
        }
        handle(message, now);
        break;
      }
    }
  }
}

void Session::handle(const mousewheeld_link_v1_HostMessage& message, Microseconds now) {
  const uint32_t id = message.message_id;
  const auto& body = message.body;

  switch (message.which_body) {
    case mousewheeld_link_v1_HostMessage_hello_tag:
      // Hello always leaves debug mode: it is how a host that finds a terminal
      // on the other end gets the protocol back.
      debug_ = false;
      send_hello_ack();
      return;

    case mousewheeld_link_v1_HostMessage_ping_tag: {
      mousewheeld_link_v1_DeviceMessage reply = mousewheeld_link_v1_DeviceMessage_init_zero;
      reply.which_body = mousewheeld_link_v1_DeviceMessage_pong_tag;
      reply.body.pong.answers = id;
      send(reply);
      return;
    }

    case mousewheeld_link_v1_HostMessage_state_tag:
      send_state_report();
      return;

    case mousewheeld_link_v1_HostMessage_axes_tag: {
      AxisSetting axes[kMaxAxes];
      for (pb_size_t a = 0; a < body.axes.axes_count; ++a) {
        axes[a].quadrature =
            body.axes.axes[a].source == mousewheeld_link_v1_AxisSource_AXIS_SOURCE_QUADRATURE;
        axes[a].invert = body.axes.axes[a].invert;
      }
      answer(device_.set_axes(axes, static_cast<uint8_t>(body.axes.axes_count)), id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_lines_tag: {
      // Checked whole before any line changes, so a refused Lines leaves the
      // map as it was.
      for (pb_size_t i = 0; i < body.lines.lines_count; ++i) {
        const auto& line = body.lines.lines[i];
        if (line.index >= kMaxLines || line.pin > 0xFF || !hal::pin_is_output(line.pin)) {
          answer(Refusal::because("bad_line", "line %u on pin %u", static_cast<unsigned>(line.index),
                                  static_cast<unsigned>(line.pin)),
                 id);
          return;
        }
      }
      for (pb_size_t i = 0; i < body.lines.lines_count; ++i) {
        const auto& line = body.lines.lines[i];
        if (Refusal refusal = device_.set_line(static_cast<uint8_t>(line.index),
                                               static_cast<uint8_t>(line.pin), line.safe_high)) {
          answer(refusal, id);
          return;
        }
      }
      send_ok(id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_stream_tag:
      answer(device_.set_stream(body.stream.rate_hz, body.stream.velocity), id);
      return;

    case mousewheeld_link_v1_HostMessage_zones_begin_tag: {
      // Answered only when refused: a clean upload is one Ok, at the end.
      const Refusal refusal = device_.begin_upload(body.zones_begin.zone_set_version,
                                                   body.zones_begin.n, body.zones_begin.name);
      if (refusal) answer(refusal, id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_zone_tag: {
      Refusal refusal;
      const Zone zone = zone_from_wire(body.zone, refusal);
      if (!refusal) refusal = device_.add_zone(body.zone.index, zone);
      if (refusal) answer(refusal, id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_zones_end_tag:
      answer(device_.end_upload(), id);
      return;

    case mousewheeld_link_v1_HostMessage_arm_tag: {
      const bool current = body.arm.origin != mousewheeld_link_v1_Origin_ORIGIN_ABSOLUTE;
      if (Refusal refusal = device_.arm(body.arm.arm_id, body.arm.zone_set_version, current, now)) {
        answer(refusal, id);
        return;
      }
      mousewheeld_link_v1_DeviceMessage reply = mousewheeld_link_v1_DeviceMessage_init_zero;
      reply.which_body = mousewheeld_link_v1_DeviceMessage_armed_tag;
      reply.body.armed.arm_id = body.arm.arm_id;
      reply.body.armed.zone_set_version = body.arm.zone_set_version;
      const Snapshot s = device_.snapshot();
      reply.body.armed.origin_count = s.n_axes;
      for (uint8_t a = 0; a < s.n_axes; ++a) reply.body.armed.origin[a] = s.origin[a];
      send(reply);
      return;
    }

    case mousewheeld_link_v1_HostMessage_disarm_tag:
      answer(device_.disarm(body.disarm.arm_id), id);
      return;

    case mousewheeld_link_v1_HostMessage_zero_tag: {
      uint32_t axes = 0;
      for (pb_size_t i = 0; i < body.zero.axes_count; ++i) {
        if (body.zero.axes[i] >= kMaxAxes) {
          answer(Refusal::because("bad_axis", "axis %u", static_cast<unsigned>(body.zero.axes[i])), id);
          return;
        }
        axes |= 1u << body.zero.axes[i];
      }
      answer(device_.zero(axes), id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_analog_tag: {
      if (body.analog.mode > mousewheeld_link_v1_AnalogMode_ANALOG_MODE_VELOCITY ||
          body.analog.axis >= kMaxAxes) {
        answer(Refusal::because("bad_analog", "a mode or axis this board does not have"), id);
        return;
      }
      AnalogConfig config;
      config.axis = static_cast<uint8_t>(body.analog.axis);
      config.mode = static_cast<AnalogMode>(body.analog.mode);
      config.range_counts = body.analog.range_counts;
      config.lo = body.analog.lo;
      config.hi = body.analog.hi;
      answer(device_.set_analog(config), id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_save_tag: {
      const Settings settings = capture_settings(device_);
      if (!hal::flash_save(reinterpret_cast<const uint8_t*>(&settings), sizeof(settings))) {
        answer(Refusal::because("flash_failed", "the settings were not written"), id);
        return;
      }
      flashed_ = settings.zones.committed;
      flashed_version_ = settings.zones.version;
      copy_string(flashed_name_, sizeof(flashed_name_), settings.zones.name);
      send_ok(id);
      return;
    }

    case mousewheeld_link_v1_HostMessage_debug_tag:
      // Ok first, in the protocol, so the host knows the text that follows was
      // asked for.
      send_ok(id);
      debug_ = body.debug.on;
      next_debug_ = now;
      return;

    default: {
      // A body this firmware cannot name: newer than it, or not ours.
      char detail[32];
      snprintf(detail, sizeof(detail), "field %u", static_cast<unsigned>(message.which_body));
      send_error("unknown_type", detail, &id);
      return;
    }
  }
}

void Session::drain_rings() {
  // Zone hits first: fewer, and the ones somebody is waiting on.
  for (size_t n = 0; n < kDrainPerLoop && queue_.free() >= kReplyHeadroom + kMaxFrame; ++n) {
    ZoneHitRecord hit;
    if (!device_.zone_hits().pop(hit)) break;
    mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
    message.which_body = mousewheeld_link_v1_DeviceMessage_zone_hit_tag;
    auto& wire = message.body.zone_hit;
    wire.seq = hit.seq;
    wire.arm_id = hit.arm_id;
    wire.zone = hit.zone;
    wire.t_us = hit.t_us;
    wire.c_count = device_.n_axes();
    for (uint8_t a = 0; a < device_.n_axes(); ++a) wire.c[a] = hit.counts[a];
    send(message);
  }

  for (size_t n = 0; n < kDrainPerLoop && queue_.free() >= kReplyHeadroom + kMaxFrame; ++n) {
    SampleRecord sample;
    if (!device_.samples().pop(sample)) break;
    mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
    message.which_body = mousewheeld_link_v1_DeviceMessage_sample_tag;
    auto& wire = message.body.sample;
    wire.seq = sample.seq;
    wire.t_us = sample.t_us;
    wire.c_count = device_.n_axes();
    for (uint8_t a = 0; a < device_.n_axes(); ++a) wire.c[a] = sample.counts[a];
    if (device_.velocity_on_samples()) {
      wire.v_count = device_.n_axes();
      for (uint8_t a = 0; a < device_.n_axes(); ++a) wire.v[a] = sample.velocity[a];
    }
    send(message);
  }
}

void Session::write_debug(Microseconds now) {
  if (now < next_debug_ || queue_.free() < kReplyHeadroom) return;
  next_debug_ = now + kDebugPeriodUs;
  const Snapshot s = device_.snapshot();
  char line[160];
  int len = snprintf(line, sizeof(line), "t_us=%llu", static_cast<unsigned long long>(now));
  for (uint8_t a = 0; a < s.n_axes && len > 0 && static_cast<size_t>(len) < sizeof(line); ++a) {
    len += snprintf(line + len, sizeof(line) - len, " | axis %u c=%lld d=%lld v=%lld", a,
                    static_cast<long long>(s.counts[a]),
                    static_cast<long long>(s.counts[a] - s.origin[a]),
                    static_cast<long long>(s.velocity[a]));
  }
  if (len > 0 && static_cast<size_t>(len) < sizeof(line)) {
    len += snprintf(line + len, sizeof(line) - len, " | %s lines=0x%02X\r\n",
                    s.armed ? "armed" : "disarmed", device_.lines().active());
  }
  if (len <= 0) return;
  const size_t n = static_cast<size_t>(len) < sizeof(line) ? static_cast<size_t>(len) : sizeof(line) - 1;
  queue_.push(reinterpret_cast<const uint8_t*>(line), n);
}

// ----------------------------------------------------------------- send ---

bool Session::send(mousewheeld_link_v1_DeviceMessage& message) {
  message.message_id = message_id_++;
  uint8_t payload[kMaxPayload];
  pb_ostream_t stream = pb_ostream_from_buffer(payload, sizeof(payload));
  if (!pb_encode(&stream, mousewheeld_link_v1_DeviceMessage_fields, &message)) return false;
  uint8_t frame[kMaxFrame];
  const size_t len = encode_frame(payload, stream.bytes_written, frame);
  if (len == 0 || !queue_.push(frame, len)) {
    ++replies_dropped_;
    return false;
  }
  return true;
}

void Session::send_hello_ack() {
  mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
  message.which_body = mousewheeld_link_v1_DeviceMessage_hello_ack_tag;
  auto& ack = message.body.hello_ack;
  copy_string(ack.board, sizeof(ack.board), kBoard);
  copy_string(ack.firmware, sizeof(ack.firmware), kFirmwareVersion);
  ack.protocol_version = kProtocolVersion;
  ack.n_axes = kMaxAxes;
  ack.max_zones = kMaxZones;
  ack.max_lines = kMaxLines;
  ack.max_frame = kMaxFrame;
  ack.scan_hz = kScanHz;
  ack.has_flashed = flashed_;
  if (flashed_) {
    copy_string(ack.flashed.name, sizeof(ack.flashed.name), flashed_name_);
    ack.flashed.version = flashed_version_;
  }
  send(message);
}

void Session::send_state_report() {
  const Snapshot s = device_.snapshot();
  mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
  message.which_body = mousewheeld_link_v1_DeviceMessage_state_report_tag;
  auto& report = message.body.state_report;
  report.c_count = report.origin_count = report.distance_count = report.v_count = s.n_axes;
  for (uint8_t a = 0; a < s.n_axes; ++a) {
    report.c[a] = s.counts[a];
    report.origin[a] = s.origin[a];
    report.distance[a] = s.distance[a];
    report.v[a] = s.velocity[a];
  }
  report.n_axes = s.n_axes;
  report.stream_rate_hz = s.stream_rate_hz;
  report.has_armed = s.armed;
  report.armed.arm_id = s.arm_id;
  report.armed.zone_set_version = s.zone_set_version;
  report.zones_live = s.zones_live;
  report.has_analog = true;
  report.analog = analog_to_wire(s.analog);
  report.ring_drops = s.ring_drops;
  report.zone_hit_drops = s.zone_hit_drops;
  report.scan_overruns = s.scan_overruns;
  report.scan_max_us = s.scan_max_us;
  report.frames_refused = frames_refused_;
  send(message);
}

void Session::send_ok(uint32_t answers) {
  mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
  message.which_body = mousewheeld_link_v1_DeviceMessage_ok_tag;
  message.body.ok.answers = answers;
  send(message);
}

void Session::send_error(const char* code, const char* detail, const uint32_t* answers) {
  mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
  message.which_body = mousewheeld_link_v1_DeviceMessage_error_tag;
  auto& error = message.body.error;
  copy_string(error.code, sizeof(error.code), code);
  copy_string(error.detail, sizeof(error.detail), detail);
  if (answers != nullptr) {
    error.has_answers = true;
    error.answers = *answers;
  }
  send(message);
}

void Session::send_log(const char* text) {
  mousewheeld_link_v1_DeviceMessage message = mousewheeld_link_v1_DeviceMessage_init_zero;
  message.which_body = mousewheeld_link_v1_DeviceMessage_log_tag;
  copy_string(message.body.log.text, sizeof(message.body.log.text), text);
  send(message);
}

void Session::answer(const Refusal& refusal, uint32_t answers) {
  if (refusal) {
    send_error(refusal.code, refusal.detail, &answers);
  } else {
    send_ok(answers);
  }
}

}  // namespace mousewheeld::protocol
