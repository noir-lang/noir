#pragma once
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <memory.h>
#include <numeric/bitop/get_msb.hpp>
#include <vector>

namespace barretenberg {
namespace wnaf {
constexpr size_t SCALAR_BITS = 127;

#define WNAF_SIZE(x) ((wnaf::SCALAR_BITS + x - 1) / (x))

template <size_t bits, size_t bit_position> inline uint64_t get_wnaf_bits_const(const uint64_t* scalar) noexcept
{
    /**
     *  we want to take a 128 bit scalar and shift it down by (bit_position).
     * We then wish to mask out `bits` number of bits.
     * Low limb contains first 64 bits, so we wish to shift this limb by (bit_position mod 64), which is also
     * (bit_position & 63) If we require bits from the high limb, these need to be shifted left, not right. Actual bit
     * position of bit in high limb = `b`. Desired position = 64 - (amount we shifted low limb by) = 64 - (bit_position
     * & 63)
     *
     * So, step 1:
     * get low limb and shift right by (bit_position & 63)
     * get high limb and shift left by (64 - (bit_position & 63))
     *
     */
    constexpr size_t lo_limb_idx = bit_position / 64;
    constexpr size_t hi_limb_idx = (bit_position + bits - 1) / 64;
    constexpr uint64_t lo_shift = bit_position & 63UL;
    constexpr uint64_t bit_mask = (1UL << static_cast<uint64_t>(bits)) - 1UL;

    uint64_t lo = (scalar[lo_limb_idx] >> lo_shift);
    if constexpr (lo_limb_idx == hi_limb_idx) {
        return lo & bit_mask;
    } else {
        constexpr uint64_t hi_shift = 64UL - (bit_position & 63UL);
        uint64_t hi = ((scalar[hi_limb_idx] << (hi_shift)));
        return (lo | hi) & bit_mask;
    }
}

template <size_t num_points, size_t wnaf_bits, size_t round_i>
inline void wnaf_round(uint64_t* scalar, uint64_t* wnaf, const uint64_t point_index, const uint64_t previous) noexcept
{
    constexpr size_t wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;
    constexpr size_t log2_num_points = static_cast<uint64_t>(numeric::get_msb(static_cast<uint32_t>(num_points)));

    if constexpr (round_i < wnaf_entries - 1) {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, round_i * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i) << log2_num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf_round<num_points, wnaf_bits, round_i + 1>(scalar, wnaf, point_index, slice + predicate);
    } else {
        constexpr size_t final_bits = SCALAR_BITS - (SCALAR_BITS / wnaf_bits) * wnaf_bits;
        uint64_t slice = get_wnaf_bits_const<final_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf[0] = ((slice + predicate) >> 1UL) | (point_index << 32UL);
    }
}

template <size_t scalar_bits, size_t num_points, size_t wnaf_bits, size_t round_i>
inline void wnaf_round(uint64_t* scalar, uint64_t* wnaf, const uint64_t point_index, const uint64_t previous) noexcept
{
    constexpr size_t wnaf_entries = (scalar_bits + wnaf_bits - 1) / wnaf_bits;
    constexpr size_t log2_num_points = static_cast<uint64_t>(numeric::get_msb(static_cast<uint32_t>(num_points)));

    if constexpr (round_i < wnaf_entries - 1) {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, round_i * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i) << log2_num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf_round<scalar_bits, num_points, wnaf_bits, round_i + 1>(scalar, wnaf, point_index, slice + predicate);
    } else {
        constexpr size_t final_bits = scalar_bits - (scalar_bits / wnaf_bits) * wnaf_bits;
        uint64_t slice = get_wnaf_bits_const<final_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf[0] = ((slice + predicate) >> 1UL) | (point_index << 32UL);
    }
}

template <size_t wnaf_bits, size_t round_i>
inline void wnaf_round_packed(const uint64_t* scalar,
                              uint64_t* wnaf,
                              const uint64_t point_index,
                              const uint64_t previous) noexcept
{
    constexpr size_t wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;

    if constexpr (round_i < wnaf_entries - 1) {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, round_i * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i)] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);
        wnaf_round_packed<wnaf_bits, round_i + 1>(scalar, wnaf, point_index, slice + predicate);
    } else {
        constexpr size_t final_bits = SCALAR_BITS - (SCALAR_BITS / wnaf_bits) * wnaf_bits;
        uint64_t slice = get_wnaf_bits_const<final_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[1] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);
        wnaf[0] = ((slice + predicate) >> 1UL) | (point_index);
    }
}

template <size_t num_points, size_t wnaf_bits>
inline void fixed_wnaf(uint64_t* scalar, uint64_t* wnaf, bool& skew_map, const size_t point_index) noexcept
{
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
    wnaf_round<num_points, wnaf_bits, 1UL>(scalar, wnaf, point_index, previous);
}

template <size_t num_bits, size_t num_points, size_t wnaf_bits>
inline void fixed_wnaf(uint64_t* scalar, uint64_t* wnaf, bool& skew_map, const size_t point_index) noexcept
{
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
    wnaf_round<num_bits, num_points, wnaf_bits, 1UL>(scalar, wnaf, point_index, previous);
}

template <size_t wnaf_bits>
inline void fixed_wnaf_packed(const uint64_t* scalar,
                              uint64_t* wnaf,
                              bool& skew_map,
                              const uint64_t point_index) noexcept
{
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
    wnaf_round_packed<wnaf_bits, 1UL>(scalar, wnaf, point_index, previous);
}

// template <size_t wnaf_bits>
// inline constexpr std::array<uint32_t, WNAF_SIZE(wnaf_bits)> fixed_wnaf(const uint64_t *scalar) const noexcept
// {
//     bool skew_map = ((scalar[0] * 1) == 0);
//     uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
//     std::array<uint32_t, WNAF_SIZE(wnaf_bits)> result;
// }
} // namespace wnaf
} // namespace barretenberg
