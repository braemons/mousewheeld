// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The board on this machine: the link on stdin/stdout, a wheel that turns at
// MOUSEWHEELD_NATIVE_SPEED counts/s (default 0), the settings blob in the file
// MOUSEWHEELD_NATIVE_FLASH (default: none, so every save fails honestly), and
// the scan on a thread. It is what the daemon's integration tests talk to, so
// they exercise the firmware's own refusals rather than a stand-in's.

#if !defined(ARDUINO)

#include <fcntl.h>
#include <unistd.h>

#include <atomic>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <mutex>
#include <thread>

#include "hal.h"

namespace mousewheeld::hal {
namespace {

using Clock = std::chrono::steady_clock;

const Clock::time_point boot = Clock::now();
std::mutex scan_mutex;
std::thread scan_thread;
std::atomic<bool> scanning{false};
double speed_counts_s = 0;

}  // namespace

void init() {
  fcntl(STDIN_FILENO, F_SETFL, fcntl(STDIN_FILENO, F_GETFL) | O_NONBLOCK);
  if (const char* speed = getenv("MOUSEWHEELD_NATIVE_SPEED")) speed_counts_s = atof(speed);
}

Microseconds micros_now() {
  return static_cast<Microseconds>(
      std::chrono::duration_cast<std::chrono::microseconds>(Clock::now() - boot).count());
}

int32_t read_counter(uint8_t axis) {
  if (axis != 0) return 0;
  // A 16-bit counter, as the cheapest hardware has, so the extension is
  // exercised on every run that turns the wheel far enough.
  const auto counts = static_cast<int64_t>(speed_counts_s * static_cast<double>(micros_now()) / 1e6);
  return static_cast<int16_t>(counts & 0xFFFF);
}

uint32_t counter_modulus() { return 65536; }

bool pin_is_output(uint8_t pin) { return pin < 64; }
void configure_line(uint8_t, uint8_t, bool) {}
void write_outputs(LineMask, LineMask) {}
void write_analog(uint16_t) {}

size_t serial_read(uint8_t* buf, size_t len) {
  const ssize_t n = read(STDIN_FILENO, buf, len);
  if (n == 0) {
    // The host closed the link: the board is unplugged.
    stop_scan();
    exit(0);
  }
  return n > 0 ? static_cast<size_t>(n) : 0;
}

size_t serial_write(const uint8_t* buf, size_t len) {
  const ssize_t n = write(STDOUT_FILENO, buf, len);
  return n > 0 ? static_cast<size_t>(n) : 0;
}

size_t serial_writable() { return 4096; }

size_t flash_load(uint8_t* buf, size_t len) {
  const char* path = getenv("MOUSEWHEELD_NATIVE_FLASH");
  if (path == nullptr) return 0;
  FILE* file = fopen(path, "rb");
  if (file == nullptr) return 0;
  const size_t n = fread(buf, 1, len, file);
  // Longer than the buffer is another firmware's blob, and says so by length.
  const bool more = fgetc(file) != EOF;
  fclose(file);
  return more ? len + 1 : n;
}

bool flash_save(const uint8_t* buf, size_t len) {
  const char* path = getenv("MOUSEWHEELD_NATIVE_FLASH");
  if (path == nullptr) return false;
  FILE* file = fopen(path, "wb");
  if (file == nullptr) return false;
  const bool ok = fwrite(buf, 1, len, file) == len;
  return fclose(file) == 0 && ok;
}

void start_scan(void (*scan)()) {
  scanning = true;
  scan_thread = std::thread([scan] {
    auto next = Clock::now();
    while (scanning) {
      next += std::chrono::microseconds(kScanPeriodUs);
      std::this_thread::sleep_until(next);
      scan();
    }
  });
}

void stop_scan() {
  scanning = false;
  if (scan_thread.joinable()) scan_thread.join();
}

void lock_scan() { scan_mutex.lock(); }
void unlock_scan() { scan_mutex.unlock(); }

}  // namespace mousewheeld::hal

#endif  // !ARDUINO
