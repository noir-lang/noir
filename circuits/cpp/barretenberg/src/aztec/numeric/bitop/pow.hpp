#pragma once

#include <cstdint>
#include "./get_msb.hpp"

namespace numeric {
constexpr uint64_t pow64(const uint64_t input, const uint64_t exponent)
{
    if (input == 0) {
        return 0;
    } else if (exponent == 0) {
        return 1;
    }

    uint64_t accumulator = input;
    uint64_t to_mul = input;
    const uint64_t maximum_set_bit = get_msb64(exponent);

    for (int i = static_cast<int>(maximum_set_bit) - 1; i >= 0; --i) {
        accumulator *= accumulator;
        if ((exponent >> i) & 1) {
            accumulator *= to_mul;
        }
    }
    return accumulator;
}

} // namespace numeric