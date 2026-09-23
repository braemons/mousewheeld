// SPDX-License-Identifier: AGPL-3.0-or-later
#pragma once

// Why a command was not carried out, by name. Every refusal goes back to the
// host as an Error with this code; nothing is refused silently.

#include <stdarg.h>
#include <stdio.h>

namespace mousewheeld {

struct Refusal {
  // nullptr: nothing was refused.
  const char* code = nullptr;
  char detail[64] = {};

  explicit operator bool() const { return code != nullptr; }

  static Refusal none() { return {}; }

  static Refusal because(const char* code, const char* format, ...)
#if defined(__GNUC__)
      __attribute__((format(printf, 2, 3)))
#endif
  {
    Refusal refusal;
    refusal.code = code;
    va_list args;
    va_start(args, format);
    vsnprintf(refusal.detail, sizeof(refusal.detail), format, args);
    va_end(args);
    return refusal;
  }
};

}  // namespace mousewheeld
