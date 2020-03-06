/** @file
 *****************************************************************************

 Wrapper around assert macro, prevents unused variable warnings in release builds.

 *****************************************************************************/

#pragma once
#include "assert.h"

// compiler should optimize this out in release builds, without triggering
// an unused variable warning
#define DONT_EVALUATE(expression)                                                                                      \
    {                                                                                                                  \
        true ? static_cast<void>(0) : static_cast<void>((expression));                                                 \
    }

#if NDEBUG
#define ASSERT(expression) DONT_EVALUATE((expression))
#else
#define ASSERT(expression) assert((expression))
#endif // NDEBUG
