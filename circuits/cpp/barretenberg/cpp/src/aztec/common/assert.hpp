#pragma once
#include "assert.h"
#include <string>
#include <stdexcept>
#include <iostream>

// Compiler should optimize this out in release builds, without triggering an unused variable warning.
#define DONT_EVALUATE(expression)                                                                                      \
    {                                                                                                                  \
        true ? static_cast<void>(0) : static_cast<void>((expression));                                                 \
    }

// NOLINTBEGIN

#if NDEBUG
#define ASSERT(expression) DONT_EVALUATE((expression))
#else
#define ASSERT(expression) assert((expression))
#endif // NDEBUG

// NOLINTEND