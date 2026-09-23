# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Host build of the portable core, for the unit tests, and of the native
# device the daemon's integration tests talk to.
#
# The core is plain C++17 with no Arduino.h, so it compiles here exactly as it
# does on a board. Board builds go through PlatformIO (platformio.ini); this
# file never sees a board HAL.
cmake_minimum_required(VERSION 3.16)
project(mousewheeld_firmware LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(MOUSEWHEELD_SANITIZE "Build with ASan and UBSan" OFF)
if(MOUSEWHEELD_SANITIZE)
  add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g)
  add_link_options(-fsanitize=address,undefined)
endif()

# Vendored, zlib-licensed, and exactly the version core/proto/ was generated
# with — generated code and runtime must agree.
add_library(nanopb STATIC
  third_party/nanopb/pb_common.c
  third_party/nanopb/pb_decode.c
  third_party/nanopb/pb_encode.c
)
target_include_directories(nanopb PUBLIC third_party/nanopb)

add_library(mousewheeld_core STATIC
  core/device.cpp
  core/settings.cpp
  core/output/analog.cpp
  core/output/lines.cpp
  core/protocol/cobs.cpp
  core/protocol/crc16.cpp
  core/protocol/framing.cpp
  core/protocol/session.cpp
  core/protocol/transmit_queue.cpp
  core/proto/mousewheeld/link/v1/link.pb.c
  core/zones/zone.cpp
  core/zones/zone_evaluator.cpp
)
target_include_directories(mousewheeld_core PUBLIC core core/proto)
target_compile_definitions(mousewheeld_core PUBLIC MOUSEWHEELD_BOARD_NAME="native")
target_link_libraries(mousewheeld_core PUBLIC nanopb)
target_compile_options(mousewheeld_core PRIVATE
  $<$<COMPILE_LANGUAGE:CXX>:-Wall -Wextra -Wpedantic -Werror>)

# The firmware on this machine, on stdin/stdout.
find_package(Threads REQUIRED)
add_executable(mousewheeld_native_device
  native/mousewheeld_native_device.cpp
  hal/native.cpp
)
target_link_libraries(mousewheeld_native_device PRIVATE mousewheeld_core Threads::Threads)
target_compile_options(mousewheeld_native_device PRIVATE -Wall -Wextra -Wpedantic -Werror)

enable_testing()

add_library(mousewheeld_test_support STATIC tests/doctest_main.cpp tests/fake_hal.cpp)
target_include_directories(mousewheeld_test_support PUBLIC tests tests/third_party)
target_link_libraries(mousewheeld_test_support PUBLIC mousewheeld_core)

foreach(t framing axis_counter zones device session)
  add_executable(test_${t} tests/test_${t}.cpp)
  target_link_libraries(test_${t} PRIVATE mousewheeld_test_support)
  target_compile_options(test_${t} PRIVATE -Wall -Wextra)
  add_test(NAME ${t} COMMAND test_${t})
endforeach()
