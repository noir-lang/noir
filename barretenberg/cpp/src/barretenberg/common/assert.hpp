#pragma once

// NOLINTBEGIN
#if NDEBUG
// Compiler should optimize this out in release builds, without triggering an unused variable warning.
#define DONT_EVALUATE(expression)                                                                                      \
    {                                                                                                                  \
        true ? static_cast<void>(0) : static_cast<void>((expression));                                                 \
    }
#define ASSERT(expression) DONT_EVALUATE((expression))
#else
// cassert in wasi-sdk takes one second to compile, only include if needed
#include <cassert>
#include <iostream>
#include <stdexcept>
#include <string>
#define ASSERT(expression) assert((expression))
#endif // NDEBUG

// NOLINTEND