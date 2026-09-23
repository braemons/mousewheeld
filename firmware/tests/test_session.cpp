// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The session end to end: frames in through the fake port, frames out, decoded
// with the same generated code the daemon's side is generated from.

#include <cstring>
#include <string>
#include <vector>

#include <pb_decode.h>
#include <pb_encode.h>

#include "device.h"
#include "doctest.h"
#include "fake_hal.h"
#include "protocol/framing.h"
#include "protocol/session.h"
#include "settings.h"

using namespace mousewheeld;
using namespace mousewheeld::protocol;

namespace {

using Host = mousewheeld_link_v1_HostMessage;
using Reply = mousewheeld_link_v1_DeviceMessage;

struct Link {
  Device device;
  Session session{device};
  uint32_t next_id = 100;
  int32_t raw[kMaxAxes] = {};

  Link() {
    fake::reset();
    fake::now = 1000;
    device.attach(raw);
  }

  void send(Host message) {
    message.message_id = next_id++;
    uint8_t payload[kMaxPayload];
    pb_ostream_t stream = pb_ostream_from_buffer(payload, sizeof(payload));
    REQUIRE(pb_encode(&stream, mousewheeld_link_v1_HostMessage_fields, &message));
    uint8_t frame[kMaxFrame];
    const size_t len = encode_frame(payload, stream.bytes_written, frame);
    fake::rx.insert(fake::rx.end(), frame, frame + len);
  }

  void send_raw(const std::vector<uint8_t>& bytes) {
    fake::rx.insert(fake::rx.end(), bytes.begin(), bytes.end());
  }

  // Poll until the port is quiet, and decode every frame the board wrote.
  std::vector<Reply> replies() {
    for (int i = 0; i < 64; ++i) session.poll(fake::now);
    std::vector<Reply> out;
    FrameReader reader;
    for (uint8_t byte : fake::tx) {
      if (reader.push(byte) != FrameReader::Result::kFrame) continue;
      Reply reply = mousewheeld_link_v1_DeviceMessage_init_zero;
      pb_istream_t stream = pb_istream_from_buffer(reader.payload(), reader.size());
      REQUIRE(pb_decode(&stream, mousewheeld_link_v1_DeviceMessage_fields, &reply));
      out.push_back(reply);
    }
    fake::tx.clear();
    return out;
  }

  Reply only_reply() {
    auto all = replies();
    REQUIRE(all.size() == 1);
    return all[0];
  }

  void scans(int n, int32_t step = 0) {
    for (int i = 0; i < n; ++i) {
      fake::now += kScanPeriodUs;
      raw[0] = static_cast<int16_t>(raw[0] + step);
      device.scan(fake::now, raw, 65536);
    }
  }

  void expect_ok(const Host& message) {
    send(message);
    const Reply reply = only_reply();
    if (reply.which_body == mousewheeld_link_v1_DeviceMessage_error_tag) {
      FAIL(reply.body.error.code << ": " << reply.body.error.detail);
    }
    REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_ok_tag);
    CHECK(reply.body.ok.answers == next_id - 1);
  }

  void upload_goal(uint32_t version, int64_t lo) {
    Host lines = mousewheeld_link_v1_HostMessage_init_zero;
    lines.which_body = mousewheeld_link_v1_HostMessage_lines_tag;
    lines.body.lines.lines_count = 1;
    lines.body.lines.lines[0] = {0, 12, false};
    expect_ok(lines);

    Host begin = mousewheeld_link_v1_HostMessage_init_zero;
    begin.which_body = mousewheeld_link_v1_HostMessage_zones_begin_tag;
    begin.body.zones_begin.zone_set_version = version;
    begin.body.zones_begin.n = 1;
    strcpy(begin.body.zones_begin.name, "goal");
    send(begin);

    Host zone = mousewheeld_link_v1_HostMessage_init_zero;
    zone.which_body = mousewheeld_link_v1_HostMessage_zone_tag;
    zone.body.zone.index = 0;
    zone.body.zone.intervals_count = 1;
    zone.body.zone.intervals[0].axis = 0;
    zone.body.zone.intervals[0].has_lo = true;
    zone.body.zone.intervals[0].lo = lo;
    zone.body.zone.pulse_ms = 10;
    send(zone);

    Host end = mousewheeld_link_v1_HostMessage_init_zero;
    end.which_body = mousewheeld_link_v1_HostMessage_zones_end_tag;
    expect_ok(end);
  }
};

Host of(pb_size_t tag) {
  Host message = mousewheeld_link_v1_HostMessage_init_zero;
  message.which_body = tag;
  return message;
}

}  // namespace

TEST_CASE("boot says hello unasked, so the host can tell the board reset") {
  Link link;
  link.session.boot(fake::now);
  const Reply reply = link.only_reply();
  REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_hello_ack_tag);
  CHECK(reply.message_id == 0);
  CHECK(reply.body.hello_ack.protocol_version == kProtocolVersion);
  CHECK(reply.body.hello_ack.max_frame == kMaxFrame);
  CHECK(reply.body.hello_ack.max_zones == kMaxZones);
  CHECK(reply.body.hello_ack.scan_hz == kScanHz);
  CHECK_FALSE(reply.body.hello_ack.has_flashed);
}

TEST_CASE("ping is answered by pong naming it, with the board's own message_id") {
  Link link;
  link.send(of(mousewheeld_link_v1_HostMessage_ping_tag));
  link.send(of(mousewheeld_link_v1_HostMessage_ping_tag));
  const auto replies = link.replies();
  REQUIRE(replies.size() == 2);
  CHECK(replies[0].body.pong.answers == 100);
  CHECK(replies[1].body.pong.answers == 101);
  CHECK(replies[1].message_id == replies[0].message_id + 1);
}

TEST_CASE("a body the board cannot name is refused by name, never dropped") {
  Link link;
  // HostMessage{message_id: 5, field 99: varint 1} — a message from a newer host.
  const uint8_t payload[] = {0x08, 0x05, 0x98, 0x06, 0x01};
  uint8_t frame[kMaxFrame];
  const size_t len = encode_frame(payload, sizeof(payload), frame);
  link.send_raw({frame, frame + len});
  const Reply reply = link.only_reply();
  REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_error_tag);
  CHECK(std::string(reply.body.error.code) == "unknown_type");
  CHECK(reply.body.error.has_answers);
  CHECK(reply.body.error.answers == 5);
}

TEST_CASE("a corrupted frame is refused and not acted on") {
  Link link;
  Host stream = of(mousewheeld_link_v1_HostMessage_stream_tag);
  stream.body.stream.rate_hz = 200;
  link.send(stream);
  // Flip a byte in the middle of the frame.
  fake::rx[2] ^= 0x10;
  const Reply reply = link.only_reply();
  REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_error_tag);
  CHECK(std::string(reply.body.error.code) == "bad_crc");
  CHECK_FALSE(reply.body.error.has_answers);
  CHECK(link.device.snapshot().stream_rate_hz == 0);
  CHECK(link.session.frames_refused() == 1);
}

TEST_CASE("upload, arm, and a zone hit with its pulse") {
  Link link;
  link.upload_goal(3, 100);

  Host arm = of(mousewheeld_link_v1_HostMessage_arm_tag);
  arm.body.arm.arm_id = 42;
  arm.body.arm.zone_set_version = 3;
  link.send(arm);
  const Reply armed = link.only_reply();
  REQUIRE(armed.which_body == mousewheeld_link_v1_DeviceMessage_armed_tag);
  CHECK(armed.body.armed.arm_id == 42);
  CHECK(armed.body.armed.zone_set_version == 3);
  REQUIRE(armed.body.armed.origin_count == 1);
  CHECK(armed.body.armed.origin[0] == 0);

  link.scans(100, 1);
  CHECK(fake::high == 0b1);
  const Reply hit = link.only_reply();
  REQUIRE(hit.which_body == mousewheeld_link_v1_DeviceMessage_zone_hit_tag);
  CHECK(hit.body.zone_hit.arm_id == 42);
  CHECK(hit.body.zone_hit.zone == 0);
  CHECK(hit.body.zone_hit.c_count == 1);
  CHECK(hit.body.zone_hit.c[0] == 100);
}

TEST_CASE("arming the wrong version is refused, naming the one the board holds") {
  Link link;
  link.upload_goal(3, 100);
  Host arm = of(mousewheeld_link_v1_HostMessage_arm_tag);
  arm.body.arm.zone_set_version = 2;
  link.send(arm);
  const Reply reply = link.only_reply();
  REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_error_tag);
  CHECK(std::string(reply.body.error.code) == "version_mismatch");
  CHECK(std::string(reply.body.error.detail).find("3") != std::string::npos);
}

TEST_CASE("a zone refused mid-upload is answered, and the commit then fails whole") {
  Link link;
  Host begin = of(mousewheeld_link_v1_HostMessage_zones_begin_tag);
  begin.body.zones_begin.n = 1;
  begin.body.zones_begin.zone_set_version = 1;
  link.send(begin);
  Host zone = of(mousewheeld_link_v1_HostMessage_zone_tag);
  zone.body.zone.intervals_count = 1;
  zone.body.zone.line = 0;  // not configured
  zone.body.zone.pulse_ms = 10;
  link.send(zone);
  link.send(of(mousewheeld_link_v1_HostMessage_zones_end_tag));
  const auto replies = link.replies();
  REQUIRE(replies.size() == 2);
  CHECK(std::string(replies[0].body.error.code) == "bad_line");
  CHECK(replies[0].body.error.answers == 101);
  CHECK(std::string(replies[1].body.error.code) == "incomplete_upload");
  CHECK_FALSE(link.device.committed().committed);
}

TEST_CASE("samples go out with the axes configured, and velocity only when asked") {
  Link link;
  Host stream = of(mousewheeld_link_v1_HostMessage_stream_tag);
  stream.body.stream.rate_hz = 1000;
  link.expect_ok(stream);
  link.scans(25, 3);
  auto replies = link.replies();
  REQUIRE(replies.size() == 5);
  for (size_t i = 0; i < replies.size(); ++i) {
    REQUIRE(replies[i].which_body == mousewheeld_link_v1_DeviceMessage_sample_tag);
    CHECK(replies[i].body.sample.seq == i);
    CHECK(replies[i].body.sample.c_count == 1);
    CHECK(replies[i].body.sample.v_count == 0);
  }
  CHECK(replies.back().body.sample.c[0] == 75);
}

TEST_CASE("the drain is bounded per pass and keeps room for replies") {
  Link link;
  Host stream = of(mousewheeld_link_v1_HostMessage_stream_tag);
  stream.body.stream.rate_hz = kScanHz;
  link.expect_ok(stream);
  link.scans(200);
  link.session.poll(fake::now);
  // One pass: at most kDrainPerLoop samples.
  FrameReader reader;
  size_t frames = 0;
  for (uint8_t byte : fake::tx) frames += reader.push(byte) == FrameReader::Result::kFrame;
  CHECK(frames == kDrainPerLoop);
  fake::tx.clear();
  // A command in the middle of a backlog is still answered.
  link.send(of(mousewheeld_link_v1_HostMessage_ping_tag));
  bool ponged = false;
  for (const Reply& reply : link.replies()) {
    ponged |= reply.which_body == mousewheeld_link_v1_DeviceMessage_pong_tag;
  }
  CHECK(ponged);
}

TEST_CASE("state_report carries counts, origin and the armed set") {
  Link link;
  link.upload_goal(5, 1000);
  link.scans(1, 30);
  Host arm = of(mousewheeld_link_v1_HostMessage_arm_tag);
  arm.body.arm.arm_id = 7;
  arm.body.arm.zone_set_version = 5;
  link.send(arm);
  link.replies();
  link.scans(1, 12);
  link.send(of(mousewheeld_link_v1_HostMessage_state_tag));
  const Reply reply = link.only_reply();
  REQUIRE(reply.which_body == mousewheeld_link_v1_DeviceMessage_state_report_tag);
  const auto& report = reply.body.state_report;
  CHECK(report.c[0] == 42);
  CHECK(report.origin[0] == 30);
  CHECK(report.distance[0] == 12);
  REQUIRE(report.has_armed);
  CHECK(report.armed.arm_id == 7);
  CHECK(report.zones_live == 1);
}

TEST_CASE("save, then a reboot comes up with the set, armed, and says so") {
  std::vector<uint8_t> saved;
  {
    Link link;
    link.upload_goal(9, 100);
    Host arm = of(mousewheeld_link_v1_HostMessage_arm_tag);
    arm.body.arm.arm_id = 1;
    arm.body.arm.zone_set_version = 9;
    link.send(arm);
    link.replies();
    link.expect_ok(of(mousewheeld_link_v1_HostMessage_save_tag));
    saved = fake::flash;
  }

  Link rebooted;
  fake::flash = saved;
  rebooted.session.boot(fake::now);
  const auto replies = rebooted.replies();
  REQUIRE(replies.size() == 2);
  REQUIRE(replies[0].which_body == mousewheeld_link_v1_DeviceMessage_hello_ack_tag);
  REQUIRE(replies[0].body.hello_ack.has_flashed);
  CHECK(std::string(replies[0].body.hello_ack.flashed.name) == "goal");
  CHECK(replies[0].body.hello_ack.flashed.version == 9);
  REQUIRE(replies[1].which_body == mousewheeld_link_v1_DeviceMessage_log_tag);
  CHECK(std::string(replies[1].body.log.text).find("armed") != std::string::npos);
  CHECK(rebooted.device.armed());

  rebooted.scans(100, 1);
  CHECK(fake::high == 0b1);
}

TEST_CASE("a blob from another firmware is refused whole, and the board says so") {
  Link link;
  fake::flash.assign(sizeof(Settings), 0xAB);
  link.session.boot(fake::now);
  const auto replies = link.replies();
  REQUIRE(replies.size() == 2);
  CHECK_FALSE(replies[0].body.hello_ack.has_flashed);
  CHECK(std::string(replies[1].body.log.text).find("another firmware") != std::string::npos);
  CHECK_FALSE(link.device.committed().committed);
}

TEST_CASE("a failed flash write is refused, not reported as saved") {
  Link link;
  fake::flash_fails = true;
  link.send(of(mousewheeld_link_v1_HostMessage_save_tag));
  const Reply reply = link.only_reply();
  CHECK(std::string(reply.body.error.code) == "flash_failed");
}

TEST_CASE("debug mode writes text, and hello brings the protocol back") {
  Link link;
  Host debug = of(mousewheeld_link_v1_HostMessage_debug_tag);
  debug.body.debug.on = true;
  link.expect_ok(debug);
  // expect_ok() read away the first line; the next is due a period later.
  fake::now += 100000;
  link.session.poll(fake::now);
  const std::string text(fake::tx.begin(), fake::tx.end());
  CHECK(text.find("axis 0 c=0") != std::string::npos);
  fake::tx.clear();
  link.send(of(mousewheeld_link_v1_HostMessage_hello_tag));
  const Reply reply = link.only_reply();
  CHECK(reply.which_body == mousewheeld_link_v1_DeviceMessage_hello_ack_tag);
  CHECK_FALSE(link.session.debug());
}

TEST_CASE("the analog output follows displacement in wrap mode") {
  Link link;
  Host analog = of(mousewheeld_link_v1_HostMessage_analog_tag);
  analog.body.analog.mode = mousewheeld_link_v1_AnalogMode_ANALOG_MODE_WRAP;
  analog.body.analog.range_counts = 1000;
  link.expect_ok(analog);
  link.scans(1, 250);
  link.session.poll(fake::now + 5000);
  CHECK(fake::analog == 250 * 65535 / 1000);
}
