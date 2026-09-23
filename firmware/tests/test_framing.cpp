// SPDX-License-Identifier: AGPL-3.0-or-later
#include <cstring>
#include <string>
#include <vector>

#include "doctest.h"
#include "protocol/cobs.h"
#include "protocol/crc16.h"
#include "protocol/framing.h"

using namespace mousewheeld::protocol;

namespace {

std::vector<uint8_t> round_trip(const std::vector<uint8_t>& in) {
  std::vector<uint8_t> encoded(cobs_max_encoded(in.size()));
  const size_t n = cobs_encode(in.data(), in.size(), encoded.data());
  REQUIRE(n <= encoded.size());
  for (size_t i = 0; i < n; ++i) REQUIRE(encoded[i] != 0);
  std::vector<uint8_t> decoded(n);
  const size_t m = cobs_decode(encoded.data(), n, decoded.data());
  decoded.resize(m);
  return decoded;
}

FrameReader::Result feed(FrameReader& reader, const uint8_t* bytes, size_t len) {
  FrameReader::Result result = FrameReader::Result::kPending;
  for (size_t i = 0; i < len; ++i) result = reader.push(bytes[i]);
  return result;
}

}  // namespace

TEST_CASE("the crc matches the reference vector the daemon pins") {
  const char* check = "123456789";
  CHECK(crc16(reinterpret_cast<const uint8_t*>(check), 9) == 0x29B1);
}

TEST_CASE("cobs round-trips zeros, runs and the 254-byte boundary") {
  CHECK(round_trip({1, 2, 3}) == std::vector<uint8_t>{1, 2, 3});
  CHECK(round_trip({0}) == std::vector<uint8_t>{0});
  CHECK(round_trip({0, 0, 1, 0}) == std::vector<uint8_t>{0, 0, 1, 0});
  for (size_t len : {253u, 254u, 255u, 508u, 600u}) {
    std::vector<uint8_t> run(len, 0x42);
    CHECK(round_trip(run) == run);
    run[len / 2] = 0;
    CHECK(round_trip(run) == run);
  }
}

TEST_CASE("cobs refuses a code that runs past the end") {
  const uint8_t bad[] = {5, 1, 2};
  uint8_t out[8];
  CHECK(cobs_decode(bad, sizeof(bad), out) == 0);
}

TEST_CASE("a sealed frame comes back out of the reader intact") {
  const uint8_t payload[] = {0x08, 0x00, 0x2A, 0x00, 0xFF};
  uint8_t frame[mousewheeld::kMaxFrame];
  const size_t len = encode_frame(payload, sizeof(payload), frame);
  CHECK(frame[len - 1] == 0);
  for (size_t i = 0; i + 1 < len; ++i) CHECK(frame[i] != 0);

  FrameReader reader;
  REQUIRE(feed(reader, frame, len) == FrameReader::Result::kFrame);
  CHECK(reader.size() == sizeof(payload));
  CHECK(memcmp(reader.payload(), payload, sizeof(payload)) == 0);
}

TEST_CASE("a flipped byte is refused by crc, never handed on") {
  const uint8_t payload[] = {1, 2, 3, 4};
  uint8_t frame[mousewheeld::kMaxFrame];
  const size_t len = encode_frame(payload, sizeof(payload), frame);
  frame[2] ^= 0x01;
  FrameReader reader;
  REQUIRE(feed(reader, frame, len) == FrameReader::Result::kRefused);
  CHECK(std::string(reader.refusal()) == "bad_crc");
}

TEST_CASE("a frame over the limit is refused by length, and the next one is read") {
  FrameReader reader;
  std::vector<uint8_t> junk(mousewheeld::kMaxFrame + 10, 0x55);
  junk.push_back(0);
  REQUIRE(feed(reader, junk.data(), junk.size()) == FrameReader::Result::kRefused);
  CHECK(std::string(reader.refusal()) == "frame_too_long");

  const uint8_t payload[] = {7};
  uint8_t frame[mousewheeld::kMaxFrame];
  const size_t len = encode_frame(payload, sizeof(payload), frame);
  CHECK(feed(reader, frame, len) == FrameReader::Result::kFrame);
}

TEST_CASE("the tail of a frame cut off by a reset resynchronises on the next delimiter") {
  const uint8_t payload[] = {9, 8, 7, 6, 5};
  uint8_t frame[mousewheeld::kMaxFrame];
  const size_t len = encode_frame(payload, sizeof(payload), frame);

  FrameReader reader;
  // The second half of a frame, as a board that booted mid-transmission sees it.
  CHECK(feed(reader, frame + 3, len - 3) == FrameReader::Result::kRefused);
  CHECK(feed(reader, frame, len) == FrameReader::Result::kFrame);
}

TEST_CASE("back-to-back delimiters are not frames") {
  FrameReader reader;
  CHECK(reader.push(0) == FrameReader::Result::kPending);
  CHECK(reader.push(0) == FrameReader::Result::kPending);
}

TEST_CASE("the largest payload fits a frame") {
  std::vector<uint8_t> payload(kMaxPayload, 0);
  uint8_t frame[mousewheeld::kMaxFrame];
  const size_t len = encode_frame(payload.data(), payload.size(), frame);
  CHECK(len > 0);
  CHECK(len <= mousewheeld::kMaxFrame);
  CHECK(encode_frame(payload.data(), kMaxPayload + 1, frame) == 0);
}
