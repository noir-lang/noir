#pragma once
#include <cstdint>
#include <numeric/bitop/get_msb.hpp>
#include <iostream>

namespace barretenberg {
namespace wnaf {
constexpr size_t SCALAR_BITS = 127;

#define WNAF_SIZE(x) ((wnaf::SCALAR_BITS + x - 1) / (x))

constexpr size_t get_optimal_bucket_width(const size_t num_points)
{
    if (num_points >= 14617149) {
        return 21;
    }
    if (num_points >= 1139094) {
        return 18;
    }
    // if (num_points >= 100000)
    if (num_points >= 155975) {
        return 15;
    }
    if (num_points >= 144834)
    // if (num_points >= 100000)
    {
        return 14;
    }
    if (num_points >= 25067) {
        return 12;
    }
    if (num_points >= 13926) {
        return 11;
    }
    if (num_points >= 7659) {
        return 10;
    }
    if (num_points >= 2436) {
        return 9;
    }
    if (num_points >= 376) {
        return 7;
    }
    if (num_points >= 231) {
        return 6;
    }
    if (num_points >= 97) {
        return 5;
    }
    if (num_points >= 35) {
        return 4;
    }
    if (num_points >= 10) {
        return 3;
    }
    if (num_points >= 2) {
        return 2;
    }
    return 1;
}
constexpr size_t get_num_buckets(const size_t num_points)
{
    const size_t bits_per_bucket = get_optimal_bucket_width(num_points / 2);
    return 1UL << bits_per_bucket;
}

constexpr size_t get_num_rounds(const size_t num_points)
{
    const size_t bits_per_bucket = get_optimal_bucket_width(num_points / 2);
    return WNAF_SIZE(bits_per_bucket + 1);
}

template <size_t bits, size_t bit_position> inline uint64_t get_wnaf_bits_const(const uint64_t* scalar) noexcept
{
    if constexpr (bits == 0) {
        return 0ULL;
    } else {
        /**
         *  we want to take a 128 bit scalar and shift it down by (bit_position).
         * We then wish to mask out `bits` number of bits.
         * Low limb contains first 64 bits, so we wish to shift this limb by (bit_position mod 64), which is also
         * (bit_position & 63) If we require bits from the high limb, these need to be shifted left, not right. Actual
         * bit position of bit in high limb = `b`. Desired position = 64 - (amount we shifted low limb by) = 64 -
         * (bit_position & 63)
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
}

inline uint64_t get_wnaf_bits(const uint64_t* scalar, const uint64_t bits, const uint64_t bit_position) noexcept
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
    const size_t lo_limb_idx = static_cast<size_t>(bit_position >> 6);
    const size_t hi_limb_idx = static_cast<size_t>((bit_position + bits - 1) >> 6);
    const uint64_t lo_shift = bit_position & 63UL;
    const uint64_t bit_mask = (1UL << static_cast<uint64_t>(bits)) - 1UL;

    const uint64_t lo = (scalar[lo_limb_idx] >> lo_shift);
    const uint64_t hi_shift = bit_position ? 64UL - (bit_position & 63UL) : 0;
    const uint64_t hi = ((scalar[hi_limb_idx] << (hi_shift)));
    const uint64_t hi_mask = bit_mask & (0ULL - (lo_limb_idx != hi_limb_idx));

    return (lo & bit_mask) | (hi & hi_mask);
}

inline void fixed_wnaf_packed(
    const uint64_t* scalar, uint64_t* wnaf, bool& skew_map, const uint64_t point_index, const size_t wnaf_bits) noexcept
{
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits(scalar, wnaf_bits, 0) + (uint64_t)skew_map;
    const size_t wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;

    for (size_t round_i = 1; round_i < wnaf_entries - 1; ++round_i) {
        uint64_t slice = get_wnaf_bits(scalar, wnaf_bits, round_i * wnaf_bits);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i)] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);
        previous = slice + predicate;
    }
    size_t final_bits = SCALAR_BITS - (wnaf_bits * (wnaf_entries - 1));
    uint64_t slice = get_wnaf_bits(scalar, final_bits, (wnaf_entries - 1) * wnaf_bits);
    uint64_t predicate = ((slice & 1UL) == 0UL);

    wnaf[1] = ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
              (point_index);
    wnaf[0] = ((slice + predicate) >> 1UL) | (point_index);
}

inline void fixed_wnaf(const uint64_t* scalar,
                       uint64_t* wnaf,
                       bool& skew_map,
                       const uint64_t point_index,
                       const uint64_t num_points,
                       const size_t wnaf_bits) noexcept
{
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits(scalar, wnaf_bits, 0) + (uint64_t)skew_map;
    const size_t wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;

    for (size_t round_i = 1; round_i < wnaf_entries - 1; ++round_i) {
        uint64_t slice = get_wnaf_bits(scalar, wnaf_bits, round_i * wnaf_bits);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i) * num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);
        previous = slice + predicate;
    }
    size_t final_bits = SCALAR_BITS - (wnaf_bits * (wnaf_entries - 1));
    uint64_t slice = get_wnaf_bits(scalar, final_bits, (wnaf_entries - 1) * wnaf_bits);
    uint64_t predicate = ((slice & 1UL) == 0UL);

    wnaf[num_points] =
        ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
        (point_index);
    wnaf[0] = ((slice + predicate) >> 1UL) | (point_index);
}

/**
 * Current flow...
 *
 * If a wnaf entry is even, we add +1 to it, and subtract 32 from the previous entry.
 * This works if the previous entry is odd. If we recursively apply this process, starting at the least significant
 *window, this will always be the case.
 *
 * However, we want to skip over windows that are 0, which poses a problem.
 *
 * Scenario 1:  even window followed by 0 window followed by any window 'x'
 *
 *   We can't add 1 to the even window and subtract 32 from the 0 window, as we don't have a bucket that maps to -32
 *   This means that we have to identify whether we are going to borrow 32 from 'x', requiring us to look at least 2
 *steps ahead
 *
 * Scenario 2: <even> <0> <0> <x>
 *
 *   This problem proceeds indefinitely - if we have adjacent 0 windows, we do not know whether we need to track a
 *borrow flag until we identify the next non-zero window
 *
 * Scenario 3: <odd> <0>
 *
 *   This one works...
 *
 * Ok, so we should be a bit more limited with when we don't include window entries.
 * The goal here is to identify short scalars, so we want to identify the most significant non-zero window
 **/
inline uint64_t get_num_scalar_bits(const uint64_t* scalar)
{
    const uint64_t msb_1 = numeric::get_msb(scalar[1]);
    const uint64_t msb_0 = numeric::get_msb(scalar[0]);

    const uint64_t scalar_1_mask = (0ULL - (scalar[1] > 0));
    const uint64_t scalar_0_mask = (0ULL - (scalar[0] > 0)) & ~scalar_1_mask;

    const uint64_t msb = (scalar_1_mask & (msb_1 + 64)) | (scalar_0_mask & (msb_0));
    return msb;
}

/**
 * How to compute an x-bit wnaf slice?
 *
 * Iterate over number of slices in scalar.
 * For each slice, if slice is even, ADD +1 to current slice and SUBTRACT 2^x from previous slice.
 * (for 1st slice we instead add +1 and set the scalar's 'skew' value to 'true' (i.e. need to subtract 1 from it at the
 * end of our scalar mul algo))
 *
 * In *wnaf we store the following:
 *  1. bits 0-30: ABSOLUTE value of wnaf (i.e. -3 goes to 3)
 *  2. bit 31: 'predicate' bool (i.e. does the wnaf value need to be negated?)
 *  3. bits 32-63: position in a point array that describes the elliptic curve point this wnaf slice is referencing
 *
 * N.B. IN OUR STDLIB ALGORITHMS THE SKEW VALUE REPRESENTS AN ADDITION NOT A SUBTRACTION (i.e. we add +1 at the end of
 * the scalar mul algo we don't sub 1) (this is to eliminate situations which could produce the point at infinity as an
 * output as our circuit logic cannot accomodate this edge case).
 *
 * Credits: Zac W.
 *
 * @param scalar Pointer to the 128-bit non-montgomery scalar that is supposed to be transformed into wnaf
 * @param wnaf Pointer to output array that needs to accomodate enough 64-bit WNAF entries
 * @param skew_map Reference to output skew value, which if true shows that the point should be added once at the end of
 * computation
 * @param wnaf_round_counts Pointer to output array specifying the number of points participating in each round
 * @param point_index The index of the point that should be multiplied by this scalar in the point array
 * @param num_points Total points in the MSM (2*num_initial_points)
 *
 */
inline void fixed_wnaf_with_counts(const uint64_t* scalar,
                                   uint64_t* wnaf,
                                   bool& skew_map,
                                   uint64_t* wnaf_round_counts,
                                   const uint64_t point_index,
                                   const uint64_t num_points,
                                   const size_t wnaf_bits) noexcept
{
    const size_t max_wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;
    if ((scalar[0] | scalar[1]) == 0ULL) {
        skew_map = false;
        for (size_t round_i = 0; round_i < max_wnaf_entries; ++round_i) {
            wnaf[(round_i)*num_points] = 0xffffffffffffffffULL;
        }
        return;
    }
    const size_t current_scalar_bits = static_cast<size_t>(get_num_scalar_bits(scalar) + 1);
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits(scalar, wnaf_bits, 0) + (uint64_t)skew_map;
    const size_t wnaf_entries = static_cast<size_t>((current_scalar_bits + wnaf_bits - 1) / wnaf_bits);

    if (wnaf_entries == 1) {
        wnaf[(max_wnaf_entries - 1) * num_points] = (previous >> 1UL) | (point_index);
        ++wnaf_round_counts[max_wnaf_entries - 1];
        for (size_t j = wnaf_entries; j < max_wnaf_entries; ++j) {
            wnaf[(max_wnaf_entries - 1 - j) * num_points] = 0xffffffffffffffffULL;
        }
        return;
    }

    // If there are several windows
    for (size_t round_i = 1; round_i < wnaf_entries - 1; ++round_i) {

        // Get a bit slice
        uint64_t slice = get_wnaf_bits(scalar, wnaf_bits, round_i * wnaf_bits);

        // Get the predicate (last bit is zero)
        uint64_t predicate = ((slice & 1UL) == 0UL);

        // Update round count
        ++wnaf_round_counts[max_wnaf_entries - round_i];

        // Calculate entry value
        // If the last bit of current slice is 1, we simply put the previous value with the point index
        // If the last bit of the current slice is 0, we negate everything, so that we subtract from the WNAF form and
        // make it 0
        wnaf[(max_wnaf_entries - round_i) * num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);

        // Update the previous value to the next windows
        previous = slice + predicate;
    }
    // The final iteration for top bits
    size_t final_bits = static_cast<size_t>(current_scalar_bits - (wnaf_bits * (wnaf_entries - 1)));
    uint64_t slice = get_wnaf_bits(scalar, final_bits, (wnaf_entries - 1) * wnaf_bits);
    uint64_t predicate = ((slice & 1UL) == 0UL);

    ++wnaf_round_counts[(max_wnaf_entries - wnaf_entries + 1)];
    wnaf[((max_wnaf_entries - wnaf_entries + 1) * num_points)] =
        ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
        (point_index);

    // Saving top bits
    ++wnaf_round_counts[max_wnaf_entries - wnaf_entries];
    wnaf[(max_wnaf_entries - wnaf_entries) * num_points] = ((slice + predicate) >> 1UL) | (point_index);

    // Fill all unused slots with -1
    for (size_t j = wnaf_entries; j < max_wnaf_entries; ++j) {
        wnaf[(max_wnaf_entries - 1 - j) * num_points] = 0xffffffffffffffffULL;
    }
}

template <size_t num_points, size_t wnaf_bits, size_t round_i>
inline void wnaf_round(uint64_t* scalar, uint64_t* wnaf, const uint64_t point_index, const uint64_t previous) noexcept
{
    constexpr size_t wnaf_entries = (SCALAR_BITS + wnaf_bits - 1) / wnaf_bits;
    constexpr size_t log2_num_points = static_cast<uint64_t>(numeric::get_msb(static_cast<uint32_t>(num_points)));

    if constexpr (round_i < wnaf_entries - 1) {
        uint64_t slice = get_wnaf_bits(scalar, wnaf_bits, round_i * wnaf_bits);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i) << log2_num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf_round<num_points, wnaf_bits, round_i + 1>(scalar, wnaf, point_index, slice + predicate);
    } else {
        constexpr size_t final_bits = SCALAR_BITS - (SCALAR_BITS / wnaf_bits) * wnaf_bits;
        uint64_t slice = get_wnaf_bits(scalar, final_bits, (wnaf_entries - 1) * wnaf_bits);
        // uint64_t slice = get_wnaf_bits_const<final_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
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
        constexpr size_t final_bits = ((scalar_bits / wnaf_bits) * wnaf_bits == scalar_bits)
                                          ? wnaf_bits
                                          : scalar_bits - (scalar_bits / wnaf_bits) * wnaf_bits;
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
        uint64_t slice = get_wnaf_bits(scalar, wnaf_bits, round_i * wnaf_bits);
        // uint64_t slice = get_wnaf_bits_const<wnaf_bits, round_i * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i)] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index);
        wnaf_round_packed<wnaf_bits, round_i + 1>(scalar, wnaf, point_index, slice + predicate);
    } else {
        constexpr size_t final_bits = SCALAR_BITS - (SCALAR_BITS / wnaf_bits) * wnaf_bits;
        uint64_t slice = get_wnaf_bits(scalar, final_bits, (wnaf_entries - 1) * wnaf_bits);
        // uint64_t slice = get_wnaf_bits_const<final_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
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

template <size_t scalar_bits, size_t num_points, size_t wnaf_bits, size_t round_i>
inline void wnaf_round_with_restricted_first_slice(uint64_t* scalar,
                                                   uint64_t* wnaf,
                                                   const uint64_t point_index,
                                                   const uint64_t previous) noexcept
{
    constexpr size_t wnaf_entries = (scalar_bits + wnaf_bits - 1) / wnaf_bits;
    constexpr size_t log2_num_points = static_cast<uint64_t>(numeric::get_msb(static_cast<uint32_t>(num_points)));
    constexpr size_t bits_in_first_slice = scalar_bits % wnaf_bits;
    if constexpr (round_i == 1) {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, (round_i - 1) * wnaf_bits + bits_in_first_slice>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);

        wnaf[(wnaf_entries - round_i) << log2_num_points] =
            ((((previous - (predicate << (bits_in_first_slice /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) |
             (predicate << 31UL)) |
            (point_index << 32UL);
        if (round_i == 1) {
            std::cerr << "writing value " << std::hex << wnaf[(wnaf_entries - round_i) << log2_num_points] << std::dec
                      << " at index " << ((wnaf_entries - round_i) << log2_num_points) << std::endl;
        }
        wnaf_round_with_restricted_first_slice<scalar_bits, num_points, wnaf_bits, round_i + 1>(
            scalar, wnaf, point_index, slice + predicate);

    } else if constexpr (round_i < wnaf_entries - 1) {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, (round_i - 1) * wnaf_bits + bits_in_first_slice>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[(wnaf_entries - round_i) << log2_num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf_round_with_restricted_first_slice<scalar_bits, num_points, wnaf_bits, round_i + 1>(
            scalar, wnaf, point_index, slice + predicate);
    } else {
        uint64_t slice = get_wnaf_bits_const<wnaf_bits, (wnaf_entries - 1) * wnaf_bits>(scalar);
        uint64_t predicate = ((slice & 1UL) == 0UL);
        wnaf[num_points] =
            ((((previous - (predicate << (wnaf_bits /*+ 1*/))) ^ (0UL - predicate)) >> 1UL) | (predicate << 31UL)) |
            (point_index << 32UL);
        wnaf[0] = ((slice + predicate) >> 1UL) | (point_index << 32UL);
    }
}

template <size_t num_bits, size_t num_points, size_t wnaf_bits>
inline void fixed_wnaf_with_restricted_first_slice(uint64_t* scalar,
                                                   uint64_t* wnaf,
                                                   bool& skew_map,
                                                   const size_t point_index) noexcept
{
    constexpr size_t bits_in_first_slice = num_bits % wnaf_bits;
    std::cerr << "bits in first slice = " << bits_in_first_slice << std::endl;
    skew_map = ((scalar[0] & 1) == 0);
    uint64_t previous = get_wnaf_bits_const<bits_in_first_slice, 0>(scalar) + (uint64_t)skew_map;
    std::cerr << "previous = " << previous << std::endl;
    wnaf_round_with_restricted_first_slice<num_bits, num_points, wnaf_bits, 1UL>(scalar, wnaf, point_index, previous);
}

// template <size_t wnaf_bits>
// inline void fixed_wnaf_packed(const uint64_t* scalar,
//                               uint64_t* wnaf,
//                               bool& skew_map,
//                               const uint64_t point_index) noexcept
// {
//     skew_map = ((scalar[0] & 1) == 0);
//     uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
//     wnaf_round_packed<wnaf_bits, 1UL>(scalar, wnaf, point_index, previous);
// }

// template <size_t wnaf_bits>
// inline constexpr std::array<uint32_t, WNAF_SIZE(wnaf_bits)> fixed_wnaf(const uint64_t *scalar) const noexcept
// {
//     bool skew_map = ((scalar[0] * 1) == 0);
//     uint64_t previous = get_wnaf_bits_const<wnaf_bits, 0>(scalar) + (uint64_t)skew_map;
//     std::array<uint32_t, WNAF_SIZE(wnaf_bits)> result;
// }
} // namespace wnaf
} // namespace barretenberg
