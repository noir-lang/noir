#pragma once
#include <stddef.h>
#include <stdint.h>

#include "../uint256/uint256.hpp"

namespace numeric {
template <uint64_t base> constexpr uint256_t map_into_sparse_form(const uint64_t input)
{
    uint256_t out = 0UL;
    uint64_t converted = (uint64_t)input;
    uint256_t base_accumulator = 1;
    for (uint64_t i = 0; i < 32; ++i) {
        uint64_t sparse_bit = ((converted >> i) & 1ULL);
        out += (uint256_t(sparse_bit) * base_accumulator);
        base_accumulator *= base;
    }
    return out;
}

template <uint64_t base> constexpr uint64_t map_from_sparse_form(const uint256_t input)
{
    uint256_t target = input;
    uint64_t output = 0;

    uint64_t count = 0;
    while (target > 0) {
        uint64_t slice = (target % base).data[0];
        uint64_t bit = slice & 1ULL;
        output += (bit << count);
        ++count;
        target -= slice;
        target = target / base;
    }

    return output;
}
}