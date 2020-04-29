#pragma once
#include <stddef.h>
#include <stdint.h>
#include <vector>

#include "../uint256/uint256.hpp"

namespace numeric {

inline std::vector<uint64_t> slice_input(const uint256_t input, const uint64_t base)
{
    uint256_t target = input;
    std::vector<uint64_t> slices;
    while (target > 0) {
        slices.push_back((target % base).data[0]);
        target /= base;
    }
    return slices;
}

inline std::vector<uint64_t> slice_input(const uint256_t input, const std::vector<uint64_t> bases)
{
    uint256_t target = input;
    std::vector<uint64_t> slices;
    for (size_t i = 0; i < bases.size(); ++i) {
        slices.push_back((target % bases[i]).data[0]);
        target /= bases[i];
    }
    return slices;
}

template <uint64_t base, uint64_t num_slices> constexpr std::array<uint256_t, num_slices> get_base_powers()
{
    std::array<uint256_t, num_slices> output{};
    output[0] = 1;
    for (size_t i = 1; i < num_slices; ++i) {
        output[i] = output[i - 1] * base;
    }
    return output;
}

template <uint64_t base> constexpr uint256_t map_into_sparse_form(const uint64_t input)
{
    uint256_t out = 0UL;
    uint64_t converted = (uint64_t)input;

    constexpr auto base_powers = get_base_powers<base, 32>();
    for (uint64_t i = 0; i < 32; ++i) {
        uint64_t sparse_bit = ((converted >> i) & 1ULL);
        if (sparse_bit) {
            out += base_powers[i];
        }
    }
    return out;
}

template <uint64_t base> constexpr uint64_t map_from_sparse_form(const uint256_t input)
{
    uint256_t target = input;
    uint64_t output = 0;

    constexpr auto bases = get_base_powers<base, 32>();

    for (uint64_t i = 0; i < 32; ++i) {
        const auto& base_power = bases[static_cast<size_t>(31 - i)];
        uint256_t prev_threshold = 0;
        for (uint64_t j = 1; j < base + 1; ++j) {
            const auto threshold = prev_threshold + base_power;
            if (target < threshold) {
                bool bit = ((j - 1) & 1);
                if (bit) {
                    output += (1ULL << (31ULL - i));
                }
                if (j > 1) {
                    target -= (prev_threshold);
                }
                break;
            }
            prev_threshold = threshold;
        }
    }

    return output;
}
} // namespace numeric