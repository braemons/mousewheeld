// SPDX-License-Identifier: AGPL-3.0-or-later
//
// ESP32 (classic WROOM-32): PCNT for the counters, the built-in DAC for the
// analog output, a hardware timer for the scan, NVS for the settings blob.
//
// ESP32-S3 and -C3 have no DAC and a different PCNT; this file is the classic
// part only.

#if defined(ARDUINO_ARCH_ESP32)

#include <Arduino.h>
#include <Preferences.h>
#include <driver/dac.h>
#include <driver/gpio.h>
#include <driver/pcnt.h>
#include <esp_timer.h>
#include <soc/gpio_struct.h>
#include <soc/pcnt_struct.h>

#include "hal.h"

#ifndef MOUSEWHEELD_BAUD
#define MOUSEWHEELD_BAUD 1000000
#endif

namespace mousewheeld::hal {
namespace {

// The wiring of the rig this was first built for: axis 0 is the wheel, on the
// pins the preliminary sketch used.
struct EncoderPins {
  gpio_num_t a;
  gpio_num_t b;
};
constexpr EncoderPins kEncoderPins[] = {
    {GPIO_NUM_32, GPIO_NUM_21},
    {GPIO_NUM_33, GPIO_NUM_27},
};
static_assert(sizeof(kEncoderPins) / sizeof(kEncoderPins[0]) >= kMaxAxes, "a pin pair per axis");

// DAC channel 1. Channel 2 (GPIO26) stays free for a line.
constexpr gpio_num_t kAnalogPin = GPIO_NUM_25;

// PCNT counts between these and resets to 0 on reaching either, so a reading is
// the true count modulo the limit. The core extends it (see hal.h).
constexpr int16_t kCounterLimit = 32000;

// Glitch filter, in APB cycles at 80 MHz: 1 µs. A 4096-count wheel at 3 rev/s
// edges every ~80 µs, so nothing real is this short.
constexpr uint16_t kFilterCycles = 80;

constexpr size_t kTxBuffer = 4096;
constexpr size_t kRxBuffer = 1024;

portMUX_TYPE scan_mux = portMUX_INITIALIZER_UNLOCKED;
hw_timer_t* scan_timer = nullptr;
void (*scan_fn)() = nullptr;

uint8_t line_pins[kMaxLines] = {};
LineMask lines_configured = 0;

Preferences preferences;
constexpr const char* kNamespace = "mousewheeld";
constexpr const char* kSettingsKey = "settings";

void configure_counter(uint8_t axis) {
  const pcnt_unit_t unit = static_cast<pcnt_unit_t>(PCNT_UNIT_0 + axis);
  const EncoderPins pins = kEncoderPins[axis];

  // ×4 quadrature on one unit: channel 0 counts A's edges with B as the
  // direction, channel 1 counts B's edges with A as the direction.
  pcnt_config_t config = {};
  config.unit = unit;
  config.counter_h_lim = kCounterLimit;
  config.counter_l_lim = -kCounterLimit;

  config.channel = PCNT_CHANNEL_0;
  config.pulse_gpio_num = pins.a;
  config.ctrl_gpio_num = pins.b;
  config.pos_mode = PCNT_COUNT_DEC;
  config.neg_mode = PCNT_COUNT_INC;
  config.lctrl_mode = PCNT_MODE_REVERSE;
  config.hctrl_mode = PCNT_MODE_KEEP;
  pcnt_unit_config(&config);

  config.channel = PCNT_CHANNEL_1;
  config.pulse_gpio_num = pins.b;
  config.ctrl_gpio_num = pins.a;
  config.pos_mode = PCNT_COUNT_INC;
  config.neg_mode = PCNT_COUNT_DEC;
  pcnt_unit_config(&config);

  // Encoders are commonly open-collector.
  gpio_pullup_en(pins.a);
  gpio_pullup_en(pins.b);

  pcnt_set_filter_value(unit, kFilterCycles);
  pcnt_filter_enable(unit);
  pcnt_counter_pause(unit);
  pcnt_counter_clear(unit);
  pcnt_counter_resume(unit);
}

void IRAM_ATTR on_timer() {
  if (scan_fn != nullptr) scan_fn();
}

bool is_reserved(uint8_t pin) {
  if (pin == 1 || pin == 3) return true;               // UART0: the link
  if (pin >= 6 && pin <= 11) return true;              // the flash
  if (pin == static_cast<uint8_t>(kAnalogPin)) return true;
  for (uint8_t a = 0; a < kMaxAxes; ++a) {
    if (pin == kEncoderPins[a].a || pin == kEncoderPins[a].b) return true;
  }
  return false;
}

}  // namespace

void init() {
  // Nothing is configured as an output until the host or the flash says so,
  // so there is nothing to make safe first.
  Serial.setRxBufferSize(kRxBuffer);
  Serial.setTxBufferSize(kTxBuffer);
  Serial.begin(MOUSEWHEELD_BAUD);

  for (uint8_t a = 0; a < kMaxAxes; ++a) configure_counter(a);

  dac_output_enable(DAC_CHANNEL_1);
  dac_output_voltage(DAC_CHANNEL_1, 128);

  preferences.begin(kNamespace, false);
}

Microseconds micros_now() { return static_cast<Microseconds>(esp_timer_get_time()); }

int32_t read_counter(uint8_t axis) {
  // The register, not pcnt_get_counter_value(): this is the hot path (rule 5).
  return static_cast<int16_t>(PCNT.cnt_unit[axis].cnt_val);
}

uint32_t counter_modulus() { return kCounterLimit; }

bool pin_is_output(uint8_t pin) {
  return pin <= 33 && GPIO_IS_VALID_OUTPUT_GPIO(pin) && !is_reserved(pin);
}

void configure_line(uint8_t index, uint8_t pin, bool level) {
  line_pins[index] = pin;
  lines_configured |= static_cast<LineMask>(1u << index);
  // Level first, then direction, so the pin never glitches to the other level.
  digitalWrite(pin, level ? HIGH : LOW);
  pinMode(pin, OUTPUT);
}

void write_outputs(LineMask high, LineMask low) {
  uint32_t set0 = 0, clear0 = 0, set1 = 0, clear1 = 0;
  for (uint8_t i = 0; i < kMaxLines; ++i) {
    const LineMask bit = static_cast<LineMask>(1u << i);
    if (!(lines_configured & bit)) continue;
    const uint8_t pin = line_pins[i];
    if (high & bit) {
      if (pin < 32) set0 |= 1u << pin; else set1 |= 1u << (pin - 32);
    } else if (low & bit) {
      if (pin < 32) clear0 |= 1u << pin; else clear1 |= 1u << (pin - 32);
    }
  }
  // Every line in one register write per bank, so lines that change together
  // change together.
  if (set0) GPIO.out_w1ts = set0;
  if (clear0) GPIO.out_w1tc = clear0;
  if (set1) GPIO.out1_w1ts.val = set1;
  if (clear1) GPIO.out1_w1tc.val = clear1;
}

void write_analog(uint16_t value) { dac_output_voltage(DAC_CHANNEL_1, value >> 8); }

size_t serial_read(uint8_t* buf, size_t len) {
  size_t n = 0;
  while (n < len && Serial.available() > 0) buf[n++] = static_cast<uint8_t>(Serial.read());
  return n;
}

size_t serial_write(const uint8_t* buf, size_t len) { return Serial.write(buf, len); }

size_t serial_writable() {
  const int room = Serial.availableForWrite();
  return room > 0 ? static_cast<size_t>(room) : 0;
}

size_t flash_load(uint8_t* buf, size_t len) {
  const size_t stored = preferences.getBytesLength(kSettingsKey);
  if (stored == 0 || stored > len) return stored > len ? stored : 0;
  return preferences.getBytes(kSettingsKey, buf, len);
}

bool flash_save(const uint8_t* buf, size_t len) {
  // The scan's timer interrupt is not in IRAM, so it is held off while the
  // flash cache is disabled for the write. The counters keep counting in
  // PCNT; the next scan folds in what happened meanwhile.
  return preferences.putBytes(kSettingsKey, buf, len) == len;
}

void start_scan(void (*scan)()) {
  scan_fn = scan;
  // Timer 0 at 1 MHz (80 MHz APB / 80), alarming every scan period.
  scan_timer = timerBegin(0, 80, true);
  timerAttachInterrupt(scan_timer, &on_timer, true);
  timerAlarmWrite(scan_timer, kScanPeriodUs, true);
  timerAlarmEnable(scan_timer);
}

void stop_scan() {
  if (scan_timer == nullptr) return;
  timerAlarmDisable(scan_timer);
  timerDetachInterrupt(scan_timer);
  timerEnd(scan_timer);
  scan_timer = nullptr;
}

void lock_scan() {
  if (xPortInIsrContext()) {
    portENTER_CRITICAL_ISR(&scan_mux);
  } else {
    portENTER_CRITICAL(&scan_mux);
  }
}

void unlock_scan() {
  if (xPortInIsrContext()) {
    portEXIT_CRITICAL_ISR(&scan_mux);
  } else {
    portEXIT_CRITICAL(&scan_mux);
  }
}

}  // namespace mousewheeld::hal

#endif  // ARDUINO_ARCH_ESP32
