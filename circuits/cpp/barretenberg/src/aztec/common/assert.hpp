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

#if NDEBUG
#define ASSERT(expression) DONT_EVALUATE((expression))
#else
#define ASSERT(expression) assert((expression))
#endif // NDEBUG

namespace barretenberg {
namespace errors {
inline void throw_or_abort [[noreturn]] (std::string const& err)
{
#ifndef __wasm__
    throw std::runtime_error(err);
#else
    std::cout << err << std::endl;
    std::abort();
#endif
}
} // namespace errors
} // namespace barretenberg
