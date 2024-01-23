#pragma once

#include "./get_msb.hpp"
#include <cstdint>

namespace bb::numeric {
constexpr uint64_t pow64(const uint64_t input, const uint64_t exponent)
{
    if (input == 0) {
        return 0;
    }
    if (exponent == 0) {
        return 1;
    }

    uint64_t accumulator = input;
    uint64_t to_mul = input;
    const uint64_t maximum_set_bit = get_msb64(exponent);

    for (int i = static_cast<int>(maximum_set_bit) - 1; i >= 0; --i) {
        accumulator *= accumulator;
        if (((exponent >> i) & 1) != 0U) {
            accumulator *= to_mul;
        }
    }
    return accumulator;
}

constexpr bool is_power_of_two(uint64_t x)
{
    return (x != 0U) && ((x & (x - 1)) == 0U);
}

} // namespace bb::numeric