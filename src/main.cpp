#include <Arduino.h>
#include <AiEsp32RotaryEncoder.h>
#include <driver/dac.h>

/*
connecting Rotary encoder

Rotary encoder side    MICROCONTROLLER side
-------------------    ---------------------------------------------------------------------
CLK (A pin)            any microcontroler intput pin with interrupt -> in this example pin 32
DT (B pin)             any microcontroler intput pin with interrupt -> in this example pin 21
SW (button pin)        any microcontroler intput pin with interrupt -> in this example pin 25
GND - to microcontroler GND
VCC                    microcontroler VCC (then set ROTARY_ENCODER_VCC_PIN -1)

***OR in case VCC pin is not free you can cheat and connect:***
VCC                    any microcontroler output pin - but set also ROTARY_ENCODER_VCC_PIN 25
                        in this example pin 25

*/
#if defined(ESP8266)
#define ROTARY_ENCODER_A_PIN D6
#define ROTARY_ENCODER_B_PIN D5
#define ROTARY_ENCODER_BUTTON_PIN D7
#else
#define ROTARY_ENCODER_A_PIN 32
#define ROTARY_ENCODER_B_PIN 21
#define ROTARY_ENCODER_BUTTON_PIN 25
#endif
#define ROTARY_ENCODER_VCC_PIN -1 /* 27 put -1 of Rotary encoder Vcc is connected directly to 3,3V; else you can use declared output pin for powering rotary encoder */

constexpr dac_channel_t ANALOG_OUTPUT_PIN = DAC_CHANNEL_1;

// depending on your encoder - try 1,2 or 4 to get expected behaviour
// #define ROTARY_ENCODER_STEPS 1
// #define ROTARY_ENCODER_STEPS 2
#define ROTARY_ENCODER_STEPS 4

// instead of changing here, rather change numbers above
AiEsp32RotaryEncoder rotaryEncoder = AiEsp32RotaryEncoder(ROTARY_ENCODER_A_PIN, ROTARY_ENCODER_B_PIN, ROTARY_ENCODER_BUTTON_PIN, ROTARY_ENCODER_VCC_PIN, ROTARY_ENCODER_STEPS);

void rotary_onButtonClick()
{
  static unsigned long lastTimePressed = 0;
  // ignore multiple press in that time milliseconds
  if (millis() - lastTimePressed < 500)
  {
    return;
  }
  lastTimePressed = millis();
  Serial.print("button pressed ");
  Serial.print(millis());
  Serial.println(" milliseconds after restart");
}

void IRAM_ATTR readEncoderISR()
{
  rotaryEncoder.readEncoder_ISR();
}

constexpr bool circleValues = true;
constexpr int minValue = 0;
constexpr int maxValue = 4096;

void setup()
{
  Serial.begin(115200);

  // we must initialize rotary encoder
  rotaryEncoder.begin();
  rotaryEncoder.setup(readEncoderISR);

  rotaryEncoder.setBoundaries(minValue, maxValue, circleValues);
  rotaryEncoder.disableAcceleration();
  rotaryEncoder.setAcceleration(0);
}

int64_t previousPosition = 0;
int64_t previousStep = 0;
int64_t previousTimeMS = 0;
constexpr int minimumOutputIntervalMS = 1;

enum Commands : int8_t
{
  kNoCommand = -1,
  kStartAcquisition = 0,
  kStopAcquisition = 1,
  kGetCurrentPosition = 2,

};
int incomingByte = 0; // for incoming serial data

struct DataPoint
{
  u_int64_t currentTimeMS;
  int64_t position;
  int64_t step;
  int64_t dt;
} output;

void loop()
{

  incomingByte = kNoCommand;
  if (Serial.available() > 0)
  {
    // read the incoming byte:
    incomingByte = Serial.read();
  }

  u_int64_t currentTimeMS = 0;
  // int64_t step = rotaryEncoder.encoderChanged();

  int64_t currentPosition = rotaryEncoder.readEncoder();
  uint8_t currentAnalogValue = map(currentPosition, minValue, maxValue, 0, 255);
  dac_output_voltage((dac_channel_t)0, currentAnalogValue);
  dac_output_voltage((dac_channel_t)1, currentAnalogValue);

  currentTimeMS = millis();

  if (incomingByte > 0)
  {
    int64_t step = currentPosition - previousPosition;
    int64_t dt = currentTimeMS - previousTimeMS;

    // print out the current you read:
    Serial.print(currentTimeMS);
    Serial.print(",");
    Serial.print(previousTimeMS);
    Serial.print(",");
    Serial.print(dt);
    Serial.print(",");
    Serial.print(currentPosition);
    Serial.print(",");
    Serial.print(previousPosition);
    Serial.print(",");
    Serial.print(step);
    Serial.print(",");
    Serial.print(currentAnalogValue);
    Serial.print("\n");

    previousPosition = currentPosition;
    previousTimeMS = currentTimeMS;
  }

  delay(minimumOutputIntervalMS);
}