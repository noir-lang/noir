#pragma once
#include "../bitop/get_msb.hpp"
#include "./uint256.hpp"
#include "barretenberg/common/assert.hpp"
namespace bb::numeric {

constexpr std::pair<uint64_t, uint64_t> uint256_t::mul_wide(const uint64_t a, const uint64_t b)
{
    const uint64_t a_lo = a & 0xffffffffULL;
    const uint64_t a_hi = a >> 32ULL;
    const uint64_t b_lo = b & 0xffffffffULL;
    const uint64_t b_hi = b >> 32ULL;

    const uint64_t lo_lo = a_lo * b_lo;
    const uint64_t hi_lo = a_hi * b_lo;
    const uint64_t lo_hi = a_lo * b_hi;
    const uint64_t hi_hi = a_hi * b_hi;

    const uint64_t cross = (lo_lo >> 32ULL) + (hi_lo & 0xffffffffULL) + lo_hi;

    return { (cross << 32ULL) | (lo_lo & 0xffffffffULL), (hi_lo >> 32ULL) + (cross >> 32ULL) + hi_hi };
}

// compute a + b + carry, returning the carry
constexpr std::pair<uint64_t, uint64_t> uint256_t::addc(const uint64_t a, const uint64_t b, const uint64_t carry_in)
{
    const uint64_t sum = a + b;
    const auto carry_temp = static_cast<uint64_t>(sum < a);
    const uint64_t r = sum + carry_in;
    const uint64_t carry_out = carry_temp + static_cast<uint64_t>(r < carry_in);
    return { r, carry_out };
}

constexpr uint64_t uint256_t::addc_discard_hi(const uint64_t a, const uint64_t b, const uint64_t carry_in)
{
    return a + b + carry_in;
}

constexpr std::pair<uint64_t, uint64_t> uint256_t::sbb(const uint64_t a, const uint64_t b, const uint64_t borrow_in)
{
    const uint64_t t_1 = a - (borrow_in >> 63ULL);
    const auto borrow_temp_1 = static_cast<uint64_t>(t_1 > a);
    const uint64_t t_2 = t_1 - b;
    const auto borrow_temp_2 = static_cast<uint64_t>(t_2 > t_1);

    return { t_2, 0ULL - (borrow_temp_1 | borrow_temp_2) };
}

constexpr uint64_t uint256_t::sbb_discard_hi(const uint64_t a, const uint64_t b, const uint64_t borrow_in)
{
    return a - b - (borrow_in >> 63ULL);
}

// {r, carry_out} = a + carry_in + b * c
constexpr std::pair<uint64_t, uint64_t> uint256_t::mac(const uint64_t a,
                                                       const uint64_t b,
                                                       const uint64_t c,
                                                       const uint64_t carry_in)
{
    std::pair<uint64_t, uint64_t> result = mul_wide(b, c);
    result.first += a;
    const auto overflow_c = static_cast<uint64_t>(result.first < a);
    result.first += carry_in;
    const auto overflow_carry = static_cast<uint64_t>(result.first < carry_in);
    result.second += (overflow_c + overflow_carry);
    return result;
}

constexpr uint64_t uint256_t::mac_discard_hi(const uint64_t a,
                                             const uint64_t b,
                                             const uint64_t c,
                                             const uint64_t carry_in)
{
    return (b * c + a + carry_in);
}
#if defined(__wasm__) || !defined(__SIZEOF_INT128__)

/**
 * @brief Multiply one limb by 9 limbs and add to resulting limbs
 *
 */
constexpr void uint256_t::wasm_madd(const uint64_t& left_limb,
                                    const uint64_t* right_limbs,
                                    uint64_t& result_0,
                                    uint64_t& result_1,
                                    uint64_t& result_2,
                                    uint64_t& result_3,
                                    uint64_t& result_4,
                                    uint64_t& result_5,
                                    uint64_t& result_6,
                                    uint64_t& result_7,
                                    uint64_t& result_8)
{
    result_0 += left_limb * right_limbs[0];
    result_1 += left_limb * right_limbs[1];
    result_2 += left_limb * right_limbs[2];
    result_3 += left_limb * right_limbs[3];
    result_4 += left_limb * right_limbs[4];
    result_5 += left_limb * right_limbs[5];
    result_6 += left_limb * right_limbs[6];
    result_7 += left_limb * right_limbs[7];
    result_8 += left_limb * right_limbs[8];
}

/**
 * @brief Convert from 4 64-bit limbs to 9 29-bit limbs
 *
 */
constexpr std::array<uint64_t, WASM_NUM_LIMBS> uint256_t::wasm_convert(const uint64_t* data)
{
    return { data[0] & 0x1fffffff,
             (data[0] >> 29) & 0x1fffffff,
             ((data[0] >> 58) & 0x3f) | ((data[1] & 0x7fffff) << 6),
             (data[1] >> 23) & 0x1fffffff,
             ((data[1] >> 52) & 0xfff) | ((data[2] & 0x1ffff) << 12),
             (data[2] >> 17) & 0x1fffffff,
             ((data[2] >> 46) & 0x3ffff) | ((data[3] & 0x7ff) << 18),
             (data[3] >> 11) & 0x1fffffff,
             (data[3] >> 40) & 0x1fffffff };
}
#endif
constexpr std::pair<uint256_t, uint256_t> uint256_t::divmod(const uint256_t& b) const
{
    if (*this == 0 || b == 0) {
        return { 0, 0 };
    }
    if (b == 1) {
        return { *this, 0 };
    }
    if (*this == b) {
        return { 1, 0 };
    }
    if (b > *this) {
        return { 0, *this };
    }

    uint256_t quotient = 0;
    uint256_t remainder = *this;

    uint64_t bit_difference = get_msb() - b.get_msb();

    uint256_t divisor = b << bit_difference;
    uint256_t accumulator = uint256_t(1) << bit_difference;

    // if the divisor is bigger than the remainder, a and b have the same bit length
    if (divisor > remainder) {
        divisor >>= 1;
        accumulator >>= 1;
    }

    // while the remainder is bigger than our original divisor, we can subtract multiples of b from the remainder,
    // and add to the quotient
    while (remainder >= b) {

        // we've shunted 'divisor' up to have the same bit length as our remainder.
        // If remainder >= divisor, then a is at least '1 << bit_difference' multiples of b
        if (remainder >= divisor) {
            remainder -= divisor;
            // we can use OR here instead of +, as
            // accumulator is always a nice power of two
            quotient |= accumulator;
        }
        divisor >>= 1;
        accumulator >>= 1;
    }

    return { quotient, remainder };
}

/**
 * @brief Compute the result of multiplication modulu 2**512
 *
 */
constexpr std::pair<uint256_t, uint256_t> uint256_t::mul_extended(const uint256_t& other) const
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const auto [r0, t0] = mul_wide(data[0], other.data[0]);
    const auto [q0, t1] = mac(t0, data[0], other.data[1], 0);
    const auto [q1, t2] = mac(t1, data[0], other.data[2], 0);
    const auto [q2, z0] = mac(t2, data[0], other.data[3], 0);

    const auto [r1, t3] = mac(q0, data[1], other.data[0], 0);
    const auto [q3, t4] = mac(q1, data[1], other.data[1], t3);
    const auto [q4, t5] = mac(q2, data[1], other.data[2], t4);
    const auto [q5, z1] = mac(z0, data[1], other.data[3], t5);

    const auto [r2, t6] = mac(q3, data[2], other.data[0], 0);
    const auto [q6, t7] = mac(q4, data[2], other.data[1], t6);
    const auto [q7, t8] = mac(q5, data[2], other.data[2], t7);
    const auto [q8, z2] = mac(z1, data[2], other.data[3], t8);

    const auto [r3, t9] = mac(q6, data[3], other.data[0], 0);
    const auto [r4, t10] = mac(q7, data[3], other.data[1], t9);
    const auto [r5, t11] = mac(q8, data[3], other.data[2], t10);
    const auto [r6, r7] = mac(z2, data[3], other.data[3], t11);

    uint256_t lo(r0, r1, r2, r3);
    uint256_t hi(r4, r5, r6, r7);
    return { lo, hi };
#else
    // Convert 4 64-bit limbs to 9 29-bit limbs
    const auto left = wasm_convert(data);
    const auto right = wasm_convert(other.data);
    constexpr uint64_t mask = 0x1fffffff;
    uint64_t temp_0 = 0;
    uint64_t temp_1 = 0;
    uint64_t temp_2 = 0;
    uint64_t temp_3 = 0;
    uint64_t temp_4 = 0;
    uint64_t temp_5 = 0;
    uint64_t temp_6 = 0;
    uint64_t temp_7 = 0;
    uint64_t temp_8 = 0;
    uint64_t temp_9 = 0;
    uint64_t temp_10 = 0;
    uint64_t temp_11 = 0;
    uint64_t temp_12 = 0;
    uint64_t temp_13 = 0;
    uint64_t temp_14 = 0;
    uint64_t temp_15 = 0;
    uint64_t temp_16 = 0;

    // Multiply and addd all limbs
    wasm_madd(left[0], &right[0], temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    wasm_madd(left[1], &right[0], temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_madd(left[2], &right[0], temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_madd(left[3], &right[0], temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_madd(left[4], &right[0], temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_madd(left[5], &right[0], temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_madd(left[6], &right[0], temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_madd(left[7], &right[0], temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_madd(left[8], &right[0], temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);

    // Convert from relaxed form into strict 29-bit form (except for temp_16)
    temp_1 += temp_0 >> WASM_LIMB_BITS;
    temp_0 &= mask;
    temp_2 += temp_1 >> WASM_LIMB_BITS;
    temp_1 &= mask;
    temp_3 += temp_2 >> WASM_LIMB_BITS;
    temp_2 &= mask;
    temp_4 += temp_3 >> WASM_LIMB_BITS;
    temp_3 &= mask;
    temp_5 += temp_4 >> WASM_LIMB_BITS;
    temp_4 &= mask;
    temp_6 += temp_5 >> WASM_LIMB_BITS;
    temp_5 &= mask;
    temp_7 += temp_6 >> WASM_LIMB_BITS;
    temp_6 &= mask;
    temp_8 += temp_7 >> WASM_LIMB_BITS;
    temp_7 &= mask;
    temp_9 += temp_8 >> WASM_LIMB_BITS;
    temp_8 &= mask;
    temp_10 += temp_9 >> WASM_LIMB_BITS;
    temp_9 &= mask;
    temp_11 += temp_10 >> WASM_LIMB_BITS;
    temp_10 &= mask;
    temp_12 += temp_11 >> WASM_LIMB_BITS;
    temp_11 &= mask;
    temp_13 += temp_12 >> WASM_LIMB_BITS;
    temp_12 &= mask;
    temp_14 += temp_13 >> WASM_LIMB_BITS;
    temp_13 &= mask;
    temp_15 += temp_14 >> WASM_LIMB_BITS;
    temp_14 &= mask;
    temp_16 += temp_15 >> WASM_LIMB_BITS;
    temp_15 &= mask;

    // Convert to 2 4-64-bit limb uint256_t objects
    return { { (temp_0 << 0) | (temp_1 << 29) | (temp_2 << 58),
               (temp_2 >> 6) | (temp_3 << 23) | (temp_4 << 52),
               (temp_4 >> 12) | (temp_5 << 17) | (temp_6 << 46),
               (temp_6 >> 18) | (temp_7 << 11) | (temp_8 << 40) },
             { (temp_8 >> 24) | (temp_9 << 5) | (temp_10 << 34) | (temp_11 << 63),
               (temp_11 >> 1) | (temp_12 << 28) | (temp_13 << 57),
               (temp_13 >> 7) | (temp_14 << 22) | (temp_15 << 51),
               (temp_15 >> 13) | (temp_16 << 16) } };
#endif
}

/**
 * Viewing `this` uint256_t as a bit string, and counting bits from 0, slices a substring.
 * @returns the uint256_t equal to the substring of bits from (and including) the `start`-th bit, to (but excluding) the
 * `end`-th bit of `this`.
 */
constexpr uint256_t uint256_t::slice(const uint64_t start, const uint64_t end) const
{
    const uint64_t range = end - start;
    const uint256_t mask = (range == 256) ? -uint256_t(1) : (uint256_t(1) << range) - 1;
    return ((*this) >> start) & mask;
}

constexpr uint256_t uint256_t::pow(const uint256_t& exponent) const
{
    uint256_t accumulator{ data[0], data[1], data[2], data[3] };
    uint256_t to_mul{ data[0], data[1], data[2], data[3] };
    const uint64_t maximum_set_bit = exponent.get_msb();

    for (int i = static_cast<int>(maximum_set_bit) - 1; i >= 0; --i) {
        accumulator *= accumulator;
        if (exponent.get_bit(static_cast<uint64_t>(i))) {
            accumulator *= to_mul;
        }
    }
    if (exponent == uint256_t(0)) {
        accumulator = uint256_t(1);
    } else if (*this == uint256_t(0)) {
        accumulator = uint256_t(0);
    }
    return accumulator;
}

constexpr bool uint256_t::get_bit(const uint64_t bit_index) const
{
    ASSERT(bit_index < 256);
    if (bit_index > 255) {
        return static_cast<bool>(0);
    }
    const auto idx = static_cast<size_t>(bit_index >> 6);
    const size_t shift = bit_index & 63;
    return static_cast<bool>((data[idx] >> shift) & 1);
}

constexpr uint64_t uint256_t::get_msb() const
{
    uint64_t idx = numeric::get_msb(data[3]);
    idx = (idx == 0 && data[3] == 0) ? numeric::get_msb(data[2]) : idx + 64;
    idx = (idx == 0 && data[2] == 0) ? numeric::get_msb(data[1]) : idx + 64;
    idx = (idx == 0 && data[1] == 0) ? numeric::get_msb(data[0]) : idx + 64;
    return idx;
}

constexpr uint256_t uint256_t::operator+(const uint256_t& other) const
{
    const auto [r0, t0] = addc(data[0], other.data[0], 0);
    const auto [r1, t1] = addc(data[1], other.data[1], t0);
    const auto [r2, t2] = addc(data[2], other.data[2], t1);
    const auto r3 = addc_discard_hi(data[3], other.data[3], t2);
    return { r0, r1, r2, r3 };
};

constexpr uint256_t uint256_t::operator-(const uint256_t& other) const
{

    const auto [r0, t0] = sbb(data[0], other.data[0], 0);
    const auto [r1, t1] = sbb(data[1], other.data[1], t0);
    const auto [r2, t2] = sbb(data[2], other.data[2], t1);
    const auto r3 = sbb_discard_hi(data[3], other.data[3], t2);
    return { r0, r1, r2, r3 };
}

constexpr uint256_t uint256_t::operator-() const
{
    return uint256_t(0) - *this;
}

constexpr uint256_t uint256_t::operator*(const uint256_t& other) const
{

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const auto [r0, t0] = mac(0, data[0], other.data[0], 0ULL);
    const auto [q0, t1] = mac(0, data[0], other.data[1], t0);
    const auto [q1, t2] = mac(0, data[0], other.data[2], t1);
    const auto q2 = mac_discard_hi(0, data[0], other.data[3], t2);

    const auto [r1, t3] = mac(q0, data[1], other.data[0], 0ULL);
    const auto [q3, t4] = mac(q1, data[1], other.data[1], t3);
    const auto q4 = mac_discard_hi(q2, data[1], other.data[2], t4);

    const auto [r2, t5] = mac(q3, data[2], other.data[0], 0ULL);
    const auto q5 = mac_discard_hi(q4, data[2], other.data[1], t5);

    const auto r3 = mac_discard_hi(q5, data[3], other.data[0], 0ULL);

    return { r0, r1, r2, r3 };
#else
    // Convert 4 64-bit limbs to 9 29-bit limbs
    const auto left = wasm_convert(data);
    const auto right = wasm_convert(other.data);
    uint64_t temp_0 = 0;
    uint64_t temp_1 = 0;
    uint64_t temp_2 = 0;
    uint64_t temp_3 = 0;
    uint64_t temp_4 = 0;
    uint64_t temp_5 = 0;
    uint64_t temp_6 = 0;
    uint64_t temp_7 = 0;
    uint64_t temp_8 = 0;

    // Multiply and add the product of left limb 0 by all right limbs
    wasm_madd(left[0], &right[0], temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    // Multiply left limb 1 by limbs 0-7 ((1,8) doesn't need to be computed, because it overflows)
    temp_1 += left[1] * right[0];
    temp_2 += left[1] * right[1];
    temp_3 += left[1] * right[2];
    temp_4 += left[1] * right[3];
    temp_5 += left[1] * right[4];
    temp_6 += left[1] * right[5];
    temp_7 += left[1] * right[6];
    temp_8 += left[1] * right[7];
    // Left limb 2 by right 0-6, etc
    temp_2 += left[2] * right[0];
    temp_3 += left[2] * right[1];
    temp_4 += left[2] * right[2];
    temp_5 += left[2] * right[3];
    temp_6 += left[2] * right[4];
    temp_7 += left[2] * right[5];
    temp_8 += left[2] * right[6];
    temp_3 += left[3] * right[0];
    temp_4 += left[3] * right[1];
    temp_5 += left[3] * right[2];
    temp_6 += left[3] * right[3];
    temp_7 += left[3] * right[4];
    temp_8 += left[3] * right[5];
    temp_4 += left[4] * right[0];
    temp_5 += left[4] * right[1];
    temp_6 += left[4] * right[2];
    temp_7 += left[4] * right[3];
    temp_8 += left[4] * right[4];
    temp_5 += left[5] * right[0];
    temp_6 += left[5] * right[1];
    temp_7 += left[5] * right[2];
    temp_8 += left[5] * right[3];
    temp_6 += left[6] * right[0];
    temp_7 += left[6] * right[1];
    temp_8 += left[6] * right[2];
    temp_7 += left[7] * right[0];
    temp_8 += left[7] * right[1];
    temp_8 += left[8] * right[0];

    // Convert from relaxed form to strict 29-bit form
    constexpr uint64_t mask = 0x1fffffff;
    temp_1 += temp_0 >> WASM_LIMB_BITS;
    temp_0 &= mask;
    temp_2 += temp_1 >> WASM_LIMB_BITS;
    temp_1 &= mask;
    temp_3 += temp_2 >> WASM_LIMB_BITS;
    temp_2 &= mask;
    temp_4 += temp_3 >> WASM_LIMB_BITS;
    temp_3 &= mask;
    temp_5 += temp_4 >> WASM_LIMB_BITS;
    temp_4 &= mask;
    temp_6 += temp_5 >> WASM_LIMB_BITS;
    temp_5 &= mask;
    temp_7 += temp_6 >> WASM_LIMB_BITS;
    temp_6 &= mask;
    temp_8 += temp_7 >> WASM_LIMB_BITS;
    temp_7 &= mask;

    // Convert back to 4 64-bit limbs
    return { (temp_0 << 0) | (temp_1 << 29) | (temp_2 << 58),
             (temp_2 >> 6) | (temp_3 << 23) | (temp_4 << 52),
             (temp_4 >> 12) | (temp_5 << 17) | (temp_6 << 46),
             (temp_6 >> 18) | (temp_7 << 11) | (temp_8 << 40) };
#endif
}

constexpr uint256_t uint256_t::operator/(const uint256_t& other) const
{
    return divmod(other).first;
}

constexpr uint256_t uint256_t::operator%(const uint256_t& other) const
{
    return divmod(other).second;
}

constexpr uint256_t uint256_t::operator&(const uint256_t& other) const
{
    return { data[0] & other.data[0], data[1] & other.data[1], data[2] & other.data[2], data[3] & other.data[3] };
}

constexpr uint256_t uint256_t::operator^(const uint256_t& other) const
{
    return { data[0] ^ other.data[0], data[1] ^ other.data[1], data[2] ^ other.data[2], data[3] ^ other.data[3] };
}

constexpr uint256_t uint256_t::operator|(const uint256_t& other) const
{
    return { data[0] | other.data[0], data[1] | other.data[1], data[2] | other.data[2], data[3] | other.data[3] };
}

constexpr uint256_t uint256_t::operator~() const
{
    return { ~data[0], ~data[1], ~data[2], ~data[3] };
}

constexpr bool uint256_t::operator==(const uint256_t& other) const
{
    return data[0] == other.data[0] && data[1] == other.data[1] && data[2] == other.data[2] && data[3] == other.data[3];
}

constexpr bool uint256_t::operator!=(const uint256_t& other) const
{
    return !(*this == other);
}

constexpr bool uint256_t::operator!() const
{
    return *this == uint256_t(0ULL);
}

constexpr bool uint256_t::operator>(const uint256_t& other) const
{
    bool t0 = data[3] > other.data[3];
    bool t1 = data[3] == other.data[3] && data[2] > other.data[2];
    bool t2 = data[3] == other.data[3] && data[2] == other.data[2] && data[1] > other.data[1];
    bool t3 =
        data[3] == other.data[3] && data[2] == other.data[2] && data[1] == other.data[1] && data[0] > other.data[0];
    return t0 || t1 || t2 || t3;
}

constexpr bool uint256_t::operator>=(const uint256_t& other) const
{
    return (*this > other) || (*this == other);
}

constexpr bool uint256_t::operator<(const uint256_t& other) const
{
    return other > *this;
}

constexpr bool uint256_t::operator<=(const uint256_t& other) const
{
    return (*this < other) || (*this == other);
}

constexpr uint256_t uint256_t::operator>>(const uint256_t& other) const
{
    uint64_t total_shift = other.data[0];

    if (total_shift >= 256 || (other.data[1] != 0U) || (other.data[2] != 0U) || (other.data[3] != 0U)) {
        return 0;
    }

    if (total_shift == 0) {
        return *this;
    }

    uint64_t num_shifted_limbs = total_shift >> 6ULL;
    uint64_t limb_shift = total_shift & 63ULL;

    std::array<uint64_t, 4> shifted_limbs = { 0, 0, 0, 0 };

    if (limb_shift == 0) {
        shifted_limbs[0] = data[0];
        shifted_limbs[1] = data[1];
        shifted_limbs[2] = data[2];
        shifted_limbs[3] = data[3];
    } else {
        uint64_t remainder_shift = 64ULL - limb_shift;

        shifted_limbs[3] = data[3] >> limb_shift;

        uint64_t remainder = (data[3]) << remainder_shift;

        shifted_limbs[2] = (data[2] >> limb_shift) + remainder;

        remainder = (data[2]) << remainder_shift;

        shifted_limbs[1] = (data[1] >> limb_shift) + remainder;

        remainder = (data[1]) << remainder_shift;

        shifted_limbs[0] = (data[0] >> limb_shift) + remainder;
    }
    uint256_t result(0);

    for (size_t i = 0; i < 4 - num_shifted_limbs; ++i) {
        result.data[i] = shifted_limbs[static_cast<size_t>(i + num_shifted_limbs)];
    }

    return result;
}

constexpr uint256_t uint256_t::operator<<(const uint256_t& other) const
{
    uint64_t total_shift = other.data[0];

    if (total_shift >= 256 || (other.data[1] != 0U) || (other.data[2] != 0U) || (other.data[3] != 0U)) {
        return 0;
    }

    if (total_shift == 0) {
        return *this;
    }
    uint64_t num_shifted_limbs = total_shift >> 6ULL;
    uint64_t limb_shift = total_shift & 63ULL;

    std::array<uint64_t, 4> shifted_limbs = { 0, 0, 0, 0 };

    if (limb_shift == 0) {
        shifted_limbs[0] = data[0];
        shifted_limbs[1] = data[1];
        shifted_limbs[2] = data[2];
        shifted_limbs[3] = data[3];
    } else {
        uint64_t remainder_shift = 64ULL - limb_shift;

        shifted_limbs[0] = data[0] << limb_shift;

        uint64_t remainder = data[0] >> remainder_shift;

        shifted_limbs[1] = (data[1] << limb_shift) + remainder;

        remainder = data[1] >> remainder_shift;

        shifted_limbs[2] = (data[2] << limb_shift) + remainder;

        remainder = data[2] >> remainder_shift;

        shifted_limbs[3] = (data[3] << limb_shift) + remainder;
    }
    uint256_t result(0);

    for (size_t i = 0; i < 4 - num_shifted_limbs; ++i) {
        result.data[static_cast<size_t>(i + num_shifted_limbs)] = shifted_limbs[i];
    }

    return result;
}

} // namespace bb::numeric
