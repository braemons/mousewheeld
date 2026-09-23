// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// The link, from the board's side: frames in, commands carried out on the
// Device, replies and the rings' records out. Everything here runs in loop().
//
// One pass of poll():
//
//   1. read what the port has, and carry out each complete frame — only while
//      the transmit queue has room for the reply, so a reply is never dropped
//      for want of space; unread bytes wait in the port's own buffer;
//   2. turn at most kDrainPerLoop ring records into frames, and only while the
//      queue keeps that headroom (rule 3: a backlog drains in bounded steps);
//   3. update the analog output, at most once a millisecond;
//   4. hand the queue to the port.

#include <stdint.h>

#include "config.h"
#include "device.h"
#include "mousewheeld/link/v1/link.pb.h"
#include "protocol/framing.h"
#include "protocol/transmit_queue.h"

namespace mousewheeld::protocol {

class Session {
 public:
  explicit Session(Device& device) : device_(device) {}

  // Restore a saved configuration, then say hello unasked: a hello_ack nobody
  // requested is how the host learns the board reset.
  void boot(Microseconds now);

  void poll(Microseconds now);

  bool debug() const { return debug_; }
  uint64_t frames_refused() const { return frames_refused_; }
  uint64_t replies_dropped() const { return replies_dropped_; }

 private:
  void read_link(Microseconds now);
  void handle(const mousewheeld_link_v1_HostMessage& message, Microseconds now);
  void drain_rings();
  void write_debug(Microseconds now);

  // Assigns the message_id, encodes and queues. False if it did not fit.
  bool send(mousewheeld_link_v1_DeviceMessage& message);
  void send_hello_ack();
  void send_state_report();
  void send_ok(uint32_t answers);
  void send_error(const char* code, const char* detail, const uint32_t* answers);
  void send_log(const char* text);
  void answer(const Refusal& refusal, uint32_t answers);

  Device& device_;
  FrameReader reader_;
  TransmitQueue queue_;
  uint16_t message_id_ = 0;
  bool debug_ = false;
  Microseconds next_analog_ = 0;
  Microseconds next_debug_ = 0;
  uint64_t frames_refused_ = 0;
  uint64_t replies_dropped_ = 0;
  // What is in flash, for HelloAck.flashed.
  bool flashed_ = false;
  uint32_t flashed_version_ = 0;
  char flashed_name_[kMaxNameLength + 1] = {};
};

}  // namespace mousewheeld::protocol
