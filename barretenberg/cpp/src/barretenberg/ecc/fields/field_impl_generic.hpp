#pragma once

#include <array>
#include <cstdint>

#include "./field_impl.hpp"
#include "barretenberg/common/op_count.hpp"

namespace bb {

// NOLINTBEGIN(readability-implicit-bool-conversion)
template <class T> constexpr std::pair<uint64_t, uint64_t> field<T>::mul_wide(uint64_t a, uint64_t b) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = (static_cast<uint128_t>(a) * static_cast<uint128_t>(b));
    return { static_cast<uint64_t>(res), static_cast<uint64_t>(res >> 64) };
#else
    const uint64_t product = a * b;
    return { product & 0xffffffffULL, product >> 32 };
#endif
}

template <class T>
constexpr uint64_t field<T>::mac(
    const uint64_t a, const uint64_t b, const uint64_t c, const uint64_t carry_in, uint64_t& carry_out) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = static_cast<uint128_t>(a) + (static_cast<uint128_t>(b) * static_cast<uint128_t>(c)) +
                          static_cast<uint128_t>(carry_in);
    carry_out = static_cast<uint64_t>(res >> 64);
    return static_cast<uint64_t>(res);
#else
    const uint64_t product = b * c + a + carry_in;
    carry_out = product >> 32;
    return product & 0xffffffffULL;
#endif
}

template <class T>
constexpr void field<T>::mac(const uint64_t a,
                             const uint64_t b,
                             const uint64_t c,
                             const uint64_t carry_in,
                             uint64_t& out,
                             uint64_t& carry_out) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = static_cast<uint128_t>(a) + (static_cast<uint128_t>(b) * static_cast<uint128_t>(c)) +
                          static_cast<uint128_t>(carry_in);
    out = static_cast<uint64_t>(res);
    carry_out = static_cast<uint64_t>(res >> 64);
#else
    const uint64_t product = b * c + a + carry_in;
    carry_out = product >> 32;
    out = product & 0xffffffffULL;
#endif
}

template <class T>
constexpr uint64_t field<T>::mac_mini(const uint64_t a,
                                      const uint64_t b,
                                      const uint64_t c,
                                      uint64_t& carry_out) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = static_cast<uint128_t>(a) + (static_cast<uint128_t>(b) * static_cast<uint128_t>(c));
    carry_out = static_cast<uint64_t>(res >> 64);
    return static_cast<uint64_t>(res);
#else
    const uint64_t product = b * c + a;
    carry_out = product >> 32;
    return product & 0xffffffffULL;
#endif
}

template <class T>
constexpr void field<T>::mac_mini(
    const uint64_t a, const uint64_t b, const uint64_t c, uint64_t& out, uint64_t& carry_out) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = static_cast<uint128_t>(a) + (static_cast<uint128_t>(b) * static_cast<uint128_t>(c));
    out = static_cast<uint64_t>(res);
    carry_out = static_cast<uint64_t>(res >> 64);
#else
    const uint64_t result = b * c + a;
    carry_out = result >> 32;
    out = result & 0xffffffffULL;
#endif
}

template <class T>
constexpr uint64_t field<T>::mac_discard_lo(const uint64_t a, const uint64_t b, const uint64_t c) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = static_cast<uint128_t>(a) + (static_cast<uint128_t>(b) * static_cast<uint128_t>(c));
    return static_cast<uint64_t>(res >> 64);
#else
    return (b * c + a) >> 32;
#endif
}

template <class T>
constexpr uint64_t field<T>::addc(const uint64_t a,
                                  const uint64_t b,
                                  const uint64_t carry_in,
                                  uint64_t& carry_out) noexcept
{
    BB_OP_COUNT_TRACK();
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint128_t res = static_cast<uint128_t>(a) + static_cast<uint128_t>(b) + static_cast<uint128_t>(carry_in);
    carry_out = static_cast<uint64_t>(res >> 64);
    return static_cast<uint64_t>(res);
#else
    uint64_t r = a + b;
    const uint64_t carry_temp = r < a;
    r += carry_in;
    carry_out = carry_temp + (r < carry_in);
    return r;
#endif
}

template <class T>
constexpr uint64_t field<T>::sbb(const uint64_t a,
                                 const uint64_t b,
                                 const uint64_t borrow_in,
                                 uint64_t& borrow_out) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint128_t res = static_cast<uint128_t>(a) - (static_cast<uint128_t>(b) + static_cast<uint128_t>(borrow_in >> 63));
    borrow_out = static_cast<uint64_t>(res >> 64);
    return static_cast<uint64_t>(res);
#else
    uint64_t t_1 = a - (borrow_in >> 63ULL);
    uint64_t borrow_temp_1 = t_1 > a;
    uint64_t t_2 = t_1 - b;
    uint64_t borrow_temp_2 = t_2 > t_1;
    borrow_out = 0ULL - (borrow_temp_1 | borrow_temp_2);
    return t_2;
#endif
}

template <class T>
constexpr uint64_t field<T>::square_accumulate(const uint64_t a,
                                               const uint64_t b,
                                               const uint64_t c,
                                               const uint64_t carry_in_lo,
                                               const uint64_t carry_in_hi,
                                               uint64_t& carry_lo,
                                               uint64_t& carry_hi) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t product = static_cast<uint128_t>(b) * static_cast<uint128_t>(c);
    const auto r0 = static_cast<uint64_t>(product);
    const auto r1 = static_cast<uint64_t>(product >> 64);
    uint64_t out = r0 + r0;
    carry_lo = (out < r0);
    out += a;
    carry_lo += (out < a);
    out += carry_in_lo;
    carry_lo += (out < carry_in_lo);
    carry_lo += r1;
    carry_hi = (carry_lo < r1);
    carry_lo += r1;
    carry_hi += (carry_lo < r1);
    carry_lo += carry_in_hi;
    carry_hi += (carry_lo < carry_in_hi);
    return out;
#else
    const auto product = b * c;
    const auto t0 = product + a + carry_in_lo;
    const auto t1 = product + t0;
    carry_hi = t1 < product;
    const auto t2 = t1 + (carry_in_hi << 32);
    carry_hi += t2 < t1;
    carry_lo = t2 >> 32;
    return t2 & 0xffffffffULL;
#endif
}

template <class T> constexpr field<T> field<T>::reduce() const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        uint256_t val{ data[0], data[1], data[2], data[3] };
        if (val >= modulus) {
            val -= modulus;
        }
        return { val.data[0], val.data[1], val.data[2], val.data[3] };
    }
    uint64_t t0 = data[0] + not_modulus.data[0];
    uint64_t c = t0 < data[0];
    auto t1 = addc(data[1], not_modulus.data[1], c, c);
    auto t2 = addc(data[2], not_modulus.data[2], c, c);
    auto t3 = addc(data[3], not_modulus.data[3], c, c);
    const uint64_t selection_mask = 0ULL - c; // 0xffff... if we have overflowed.
    const uint64_t selection_mask_inverse = ~selection_mask;
    // if we overflow, we want to swap
    return {
        (data[0] & selection_mask_inverse) | (t0 & selection_mask),
        (data[1] & selection_mask_inverse) | (t1 & selection_mask),
        (data[2] & selection_mask_inverse) | (t2 & selection_mask),
        (data[3] & selection_mask_inverse) | (t3 & selection_mask),
    };
}

template <class T> constexpr field<T> field<T>::add(const field& other) const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        uint64_t r0 = data[0] + other.data[0];
        uint64_t c = r0 < data[0];
        auto r1 = addc(data[1], other.data[1], c, c);
        auto r2 = addc(data[2], other.data[2], c, c);
        auto r3 = addc(data[3], other.data[3], c, c);
        if (c) {
            uint64_t b = 0;
            r0 = sbb(r0, modulus.data[0], b, b);
            r1 = sbb(r1, modulus.data[1], b, b);
            r2 = sbb(r2, modulus.data[2], b, b);
            r3 = sbb(r3, modulus.data[3], b, b);
            // Since both values are in [0, 2**256), the result is in [0, 2**257-2]. Subtracting one p might not be
            // enough. We need to ensure that we've underflown the 0 and that might require subtracting an additional p
            if (!b) {
                b = 0;
                r0 = sbb(r0, modulus.data[0], b, b);
                r1 = sbb(r1, modulus.data[1], b, b);
                r2 = sbb(r2, modulus.data[2], b, b);
                r3 = sbb(r3, modulus.data[3], b, b);
            }
        }
        return { r0, r1, r2, r3 };
    } else {
        uint64_t r0 = data[0] + other.data[0];
        uint64_t c = r0 < data[0];
        auto r1 = addc(data[1], other.data[1], c, c);
        auto r2 = addc(data[2], other.data[2], c, c);
        uint64_t r3 = data[3] + other.data[3] + c;

        uint64_t t0 = r0 + twice_not_modulus.data[0];
        c = t0 < twice_not_modulus.data[0];
        uint64_t t1 = addc(r1, twice_not_modulus.data[1], c, c);
        uint64_t t2 = addc(r2, twice_not_modulus.data[2], c, c);
        uint64_t t3 = addc(r3, twice_not_modulus.data[3], c, c);
        const uint64_t selection_mask = 0ULL - c;
        const uint64_t selection_mask_inverse = ~selection_mask;

        return {
            (r0 & selection_mask_inverse) | (t0 & selection_mask),
            (r1 & selection_mask_inverse) | (t1 & selection_mask),
            (r2 & selection_mask_inverse) | (t2 & selection_mask),
            (r3 & selection_mask_inverse) | (t3 & selection_mask),
        };
    }
}

template <class T> constexpr field<T> field<T>::subtract(const field& other) const noexcept
{
    uint64_t borrow = 0;
    uint64_t r0 = sbb(data[0], other.data[0], borrow, borrow);
    uint64_t r1 = sbb(data[1], other.data[1], borrow, borrow);
    uint64_t r2 = sbb(data[2], other.data[2], borrow, borrow);
    uint64_t r3 = sbb(data[3], other.data[3], borrow, borrow);

    r0 += (modulus.data[0] & borrow);
    uint64_t carry = r0 < (modulus.data[0] & borrow);
    r1 = addc(r1, modulus.data[1] & borrow, carry, carry);
    r2 = addc(r2, modulus.data[2] & borrow, carry, carry);
    r3 = addc(r3, (modulus.data[3] & borrow), carry, carry);
    // The value being subtracted is in [0, 2**256), if we subtract 0 - 2*255 and then add p, the value will stay
    // negative. If we are adding p, we need to check that we've overflown 2**256. If not, we should add p again
    if (!carry) {
        r0 += (modulus.data[0] & borrow);
        uint64_t carry = r0 < (modulus.data[0] & borrow);
        r1 = addc(r1, modulus.data[1] & borrow, carry, carry);
        r2 = addc(r2, modulus.data[2] & borrow, carry, carry);
        r3 = addc(r3, (modulus.data[3] & borrow), carry, carry);
    }
    return { r0, r1, r2, r3 };
}

/**
 * @brief
 *
 * @tparam T
 * @param other
 * @return constexpr field<T>
 */
template <class T> constexpr field<T> field<T>::subtract_coarse(const field& other) const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        return subtract(other);
    }
    uint64_t borrow = 0;
    uint64_t r0 = sbb(data[0], other.data[0], borrow, borrow);
    uint64_t r1 = sbb(data[1], other.data[1], borrow, borrow);
    uint64_t r2 = sbb(data[2], other.data[2], borrow, borrow);
    uint64_t r3 = sbb(data[3], other.data[3], borrow, borrow);

    r0 += (twice_modulus.data[0] & borrow);
    uint64_t carry = r0 < (twice_modulus.data[0] & borrow);
    r1 = addc(r1, twice_modulus.data[1] & borrow, carry, carry);
    r2 = addc(r2, twice_modulus.data[2] & borrow, carry, carry);
    r3 += (twice_modulus.data[3] & borrow) + carry;

    return { r0, r1, r2, r3 };
}

/**
 * @brief Mongtomery multiplication for moduli > 2²⁵⁴
 *
 * @details Explanation of Montgomery form can be found in \ref field_docs_montgomery_explainer and the difference
 * between WASM and generic versions is explained in \ref field_docs_architecture_details
 */
template <class T> constexpr field<T> field<T>::montgomery_mul_big(const field& other) const noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint64_t c = 0;
    uint64_t t0 = 0;
    uint64_t t1 = 0;
    uint64_t t2 = 0;
    uint64_t t3 = 0;
    uint64_t t4 = 0;
    uint64_t t5 = 0;
    uint64_t k = 0;
    for (const auto& element : data) {
        c = 0;
        mac(t0, element, other.data[0], c, t0, c);
        mac(t1, element, other.data[1], c, t1, c);
        mac(t2, element, other.data[2], c, t2, c);
        mac(t3, element, other.data[3], c, t3, c);
        t4 = addc(t4, c, 0, t5);

        c = 0;
        k = t0 * T::r_inv;
        c = mac_discard_lo(t0, k, modulus.data[0]);
        mac(t1, k, modulus.data[1], c, t0, c);
        mac(t2, k, modulus.data[2], c, t1, c);
        mac(t3, k, modulus.data[3], c, t2, c);
        t3 = addc(c, t4, 0, c);
        t4 = t5 + c;
    }
    uint64_t borrow = 0;
    uint64_t r0 = sbb(t0, modulus.data[0], borrow, borrow);
    uint64_t r1 = sbb(t1, modulus.data[1], borrow, borrow);
    uint64_t r2 = sbb(t2, modulus.data[2], borrow, borrow);
    uint64_t r3 = sbb(t3, modulus.data[3], borrow, borrow);
    borrow = borrow ^ (0ULL - t4);
    r0 += (modulus.data[0] & borrow);
    uint64_t carry = r0 < (modulus.data[0] & borrow);
    r1 = addc(r1, modulus.data[1] & borrow, carry, carry);
    r2 = addc(r2, modulus.data[2] & borrow, carry, carry);
    r3 += (modulus.data[3] & borrow) + carry;
    return { r0, r1, r2, r3 };
#else

    // Convert 4 64-bit limbs to 9 29-bit limbs
    auto left = wasm_convert(data);
    auto right = wasm_convert(other.data);
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
    uint64_t temp_17 = 0;

    // Multiply-add 0th limb of the left argument by all 9 limbs of the right arguemnt
    wasm_madd(left[0], right, temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    // Instantly reduce
    wasm_reduce(temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    // Continue for other limbs
    wasm_madd(left[1], right, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_reduce(temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_madd(left[2], right, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_reduce(temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_madd(left[3], right, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_reduce(temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_madd(left[4], right, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_reduce(temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_madd(left[5], right, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_reduce(temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_madd(left[6], right, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_reduce(temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_madd(left[7], right, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_reduce(temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_madd(left[8], right, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);
    wasm_reduce(temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);

    // After all multiplications and additions, convert relaxed form to strict (all limbs are 29 bits)
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
    temp_17 += temp_16 >> WASM_LIMB_BITS;
    temp_16 &= mask;

    uint64_t r_temp_0;
    uint64_t r_temp_1;
    uint64_t r_temp_2;
    uint64_t r_temp_3;
    uint64_t r_temp_4;
    uint64_t r_temp_5;
    uint64_t r_temp_6;
    uint64_t r_temp_7;
    uint64_t r_temp_8;
    // Subtract modulus from result
    r_temp_0 = temp_9 - wasm_modulus[0];
    r_temp_1 = temp_10 - wasm_modulus[1] - ((r_temp_0) >> 63);
    r_temp_2 = temp_11 - wasm_modulus[2] - ((r_temp_1) >> 63);
    r_temp_3 = temp_12 - wasm_modulus[3] - ((r_temp_2) >> 63);
    r_temp_4 = temp_13 - wasm_modulus[4] - ((r_temp_3) >> 63);
    r_temp_5 = temp_14 - wasm_modulus[5] - ((r_temp_4) >> 63);
    r_temp_6 = temp_15 - wasm_modulus[6] - ((r_temp_5) >> 63);
    r_temp_7 = temp_16 - wasm_modulus[7] - ((r_temp_6) >> 63);
    r_temp_8 = temp_17 - wasm_modulus[8] - ((r_temp_7) >> 63);

    // Depending on whether the subtraction underflowed, choose original value or the result of subtraction
    uint64_t new_mask = 0 - (r_temp_8 >> 63);
    uint64_t inverse_mask = (~new_mask) & mask;
    temp_9 = (temp_9 & new_mask) | (r_temp_0 & inverse_mask);
    temp_10 = (temp_10 & new_mask) | (r_temp_1 & inverse_mask);
    temp_11 = (temp_11 & new_mask) | (r_temp_2 & inverse_mask);
    temp_12 = (temp_12 & new_mask) | (r_temp_3 & inverse_mask);
    temp_13 = (temp_13 & new_mask) | (r_temp_4 & inverse_mask);
    temp_14 = (temp_14 & new_mask) | (r_temp_5 & inverse_mask);
    temp_15 = (temp_15 & new_mask) | (r_temp_6 & inverse_mask);
    temp_16 = (temp_16 & new_mask) | (r_temp_7 & inverse_mask);
    temp_17 = (temp_17 & new_mask) | (r_temp_8 & inverse_mask);

    // Convert back to 4 64-bit limbs
    return { (temp_9 << 0) | (temp_10 << 29) | (temp_11 << 58),
             (temp_11 >> 6) | (temp_12 << 23) | (temp_13 << 52),
             (temp_13 >> 12) | (temp_14 << 17) | (temp_15 << 46),
             (temp_15 >> 18) | (temp_16 << 11) | (temp_17 << 40) };

#endif
}

#if defined(__wasm__) || !defined(__SIZEOF_INT128__)

/**
 * @brief Multiply left limb by a sequence of 9 limbs and put into result variables
 *
 */
template <class T>
constexpr void field<T>::wasm_madd(uint64_t& left_limb,
                                   const std::array<uint64_t, WASM_NUM_LIMBS>& right_limbs,
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
 * @brief Perform 29-bit montgomery reduction on 1 limb (result_0 should be zero modulo 2**29 after this)
 *
 */
template <class T>
constexpr void field<T>::wasm_reduce(uint64_t& result_0,
                                     uint64_t& result_1,
                                     uint64_t& result_2,
                                     uint64_t& result_3,
                                     uint64_t& result_4,
                                     uint64_t& result_5,
                                     uint64_t& result_6,
                                     uint64_t& result_7,
                                     uint64_t& result_8)
{
    constexpr uint64_t mask = 0x1fffffff;
    constexpr uint64_t r_inv = T::r_inv & mask;
    uint64_t k = (result_0 * r_inv) & mask;
    result_0 += k * wasm_modulus[0];
    result_1 += k * wasm_modulus[1] + (result_0 >> WASM_LIMB_BITS);
    result_2 += k * wasm_modulus[2];
    result_3 += k * wasm_modulus[3];
    result_4 += k * wasm_modulus[4];
    result_5 += k * wasm_modulus[5];
    result_6 += k * wasm_modulus[6];
    result_7 += k * wasm_modulus[7];
    result_8 += k * wasm_modulus[8];
}
/**
 * @brief Convert 4 64-bit limbs into 9 29-bit limbs
 *
 */
template <class T> constexpr std::array<uint64_t, WASM_NUM_LIMBS> field<T>::wasm_convert(const uint64_t* data)
{
    return { data[0] & 0x1fffffff,
             (data[0] >> WASM_LIMB_BITS) & 0x1fffffff,
             ((data[0] >> 58) & 0x3f) | ((data[1] & 0x7fffff) << 6),
             (data[1] >> 23) & 0x1fffffff,
             ((data[1] >> 52) & 0xfff) | ((data[2] & 0x1ffff) << 12),
             (data[2] >> 17) & 0x1fffffff,
             ((data[2] >> 46) & 0x3ffff) | ((data[3] & 0x7ff) << 18),
             (data[3] >> 11) & 0x1fffffff,
             (data[3] >> 40) & 0x1fffffff };
}
#endif
template <class T> constexpr field<T> field<T>::montgomery_mul(const field& other) const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        return montgomery_mul_big(other);
    }
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    auto [t0, c] = mul_wide(data[0], other.data[0]);
    uint64_t k = t0 * T::r_inv;
    uint64_t a = mac_discard_lo(t0, k, modulus.data[0]);

    uint64_t t1 = mac_mini(a, data[0], other.data[1], a);
    mac(t1, k, modulus.data[1], c, t0, c);
    uint64_t t2 = mac_mini(a, data[0], other.data[2], a);
    mac(t2, k, modulus.data[2], c, t1, c);
    uint64_t t3 = mac_mini(a, data[0], other.data[3], a);
    mac(t3, k, modulus.data[3], c, t2, c);
    t3 = c + a;

    mac_mini(t0, data[1], other.data[0], t0, a);
    k = t0 * T::r_inv;
    c = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, data[1], other.data[1], a, t1, a);
    mac(t1, k, modulus.data[1], c, t0, c);
    mac(t2, data[1], other.data[2], a, t2, a);
    mac(t2, k, modulus.data[2], c, t1, c);
    mac(t3, data[1], other.data[3], a, t3, a);
    mac(t3, k, modulus.data[3], c, t2, c);
    t3 = c + a;

    mac_mini(t0, data[2], other.data[0], t0, a);
    k = t0 * T::r_inv;
    c = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, data[2], other.data[1], a, t1, a);
    mac(t1, k, modulus.data[1], c, t0, c);
    mac(t2, data[2], other.data[2], a, t2, a);
    mac(t2, k, modulus.data[2], c, t1, c);
    mac(t3, data[2], other.data[3], a, t3, a);
    mac(t3, k, modulus.data[3], c, t2, c);
    t3 = c + a;

    mac_mini(t0, data[3], other.data[0], t0, a);
    k = t0 * T::r_inv;
    c = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, data[3], other.data[1], a, t1, a);
    mac(t1, k, modulus.data[1], c, t0, c);
    mac(t2, data[3], other.data[2], a, t2, a);
    mac(t2, k, modulus.data[2], c, t1, c);
    mac(t3, data[3], other.data[3], a, t3, a);
    mac(t3, k, modulus.data[3], c, t2, c);
    t3 = c + a;
    return { t0, t1, t2, t3 };
#else

    // Convert 4 64-bit limbs to 9 29-bit ones
    auto left = wasm_convert(data);
    auto right = wasm_convert(other.data);
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

    // Perform a series of multiplications and reductions (we multiply 1 limb of left argument by the whole right
    // argument and then reduce)
    wasm_madd(left[0], right, temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    wasm_madd(left[1], right, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_madd(left[2], right, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_madd(left[3], right, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_madd(left[4], right, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_madd(left[5], right, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_madd(left[6], right, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_madd(left[7], right, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_madd(left[8], right, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);
    wasm_reduce(temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    wasm_reduce(temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_reduce(temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_reduce(temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_reduce(temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_reduce(temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_reduce(temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_reduce(temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_reduce(temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);

    // Convert result to unrelaxed form (all limbs are 29 bits)
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

    // Convert back to 4 64-bit limbs form
    return { (temp_9 << 0) | (temp_10 << 29) | (temp_11 << 58),
             (temp_11 >> 6) | (temp_12 << 23) | (temp_13 << 52),
             (temp_13 >> 12) | (temp_14 << 17) | (temp_15 << 46),
             (temp_15 >> 18) | (temp_16 << 11) };
#endif
}

template <class T> constexpr field<T> field<T>::montgomery_square() const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        return montgomery_mul_big(*this);
    }
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint64_t carry_hi = 0;

    auto [t0, carry_lo] = mul_wide(data[0], data[0]);
    uint64_t t1 = square_accumulate(0, data[1], data[0], carry_lo, carry_hi, carry_lo, carry_hi);
    uint64_t t2 = square_accumulate(0, data[2], data[0], carry_lo, carry_hi, carry_lo, carry_hi);
    uint64_t t3 = square_accumulate(0, data[3], data[0], carry_lo, carry_hi, carry_lo, carry_hi);

    uint64_t round_carry = carry_lo;
    uint64_t k = t0 * T::r_inv;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;

    t1 = mac_mini(t1, data[1], data[1], carry_lo);
    carry_hi = 0;
    t2 = square_accumulate(t2, data[2], data[1], carry_lo, carry_hi, carry_lo, carry_hi);
    t3 = square_accumulate(t3, data[3], data[1], carry_lo, carry_hi, carry_lo, carry_hi);
    round_carry = carry_lo;
    k = t0 * T::r_inv;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;

    t2 = mac_mini(t2, data[2], data[2], carry_lo);
    carry_hi = 0;
    t3 = square_accumulate(t3, data[3], data[2], carry_lo, carry_hi, carry_lo, carry_hi);
    round_carry = carry_lo;
    k = t0 * T::r_inv;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;

    t3 = mac_mini(t3, data[3], data[3], carry_lo);
    k = t0 * T::r_inv;
    round_carry = carry_lo;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;
    return { t0, t1, t2, t3 };
#else
    // Convert from 4 64-bit limbs to 9 29-bit ones
    auto left = wasm_convert(data);
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
    uint64_t acc;
    // Perform multiplications, but accumulated results for limb k=i+j so that we can double them at the same time
    temp_0 += left[0] * left[0];
    acc = 0;
    acc += left[0] * left[1];
    temp_1 += (acc << 1);
    acc = 0;
    acc += left[0] * left[2];
    temp_2 += left[1] * left[1];
    temp_2 += (acc << 1);
    acc = 0;
    acc += left[0] * left[3];
    acc += left[1] * left[2];
    temp_3 += (acc << 1);
    acc = 0;
    acc += left[0] * left[4];
    acc += left[1] * left[3];
    temp_4 += left[2] * left[2];
    temp_4 += (acc << 1);
    acc = 0;
    acc += left[0] * left[5];
    acc += left[1] * left[4];
    acc += left[2] * left[3];
    temp_5 += (acc << 1);
    acc = 0;
    acc += left[0] * left[6];
    acc += left[1] * left[5];
    acc += left[2] * left[4];
    temp_6 += left[3] * left[3];
    temp_6 += (acc << 1);
    acc = 0;
    acc += left[0] * left[7];
    acc += left[1] * left[6];
    acc += left[2] * left[5];
    acc += left[3] * left[4];
    temp_7 += (acc << 1);
    acc = 0;
    acc += left[0] * left[8];
    acc += left[1] * left[7];
    acc += left[2] * left[6];
    acc += left[3] * left[5];
    temp_8 += left[4] * left[4];
    temp_8 += (acc << 1);
    acc = 0;
    acc += left[1] * left[8];
    acc += left[2] * left[7];
    acc += left[3] * left[6];
    acc += left[4] * left[5];
    temp_9 += (acc << 1);
    acc = 0;
    acc += left[2] * left[8];
    acc += left[3] * left[7];
    acc += left[4] * left[6];
    temp_10 += left[5] * left[5];
    temp_10 += (acc << 1);
    acc = 0;
    acc += left[3] * left[8];
    acc += left[4] * left[7];
    acc += left[5] * left[6];
    temp_11 += (acc << 1);
    acc = 0;
    acc += left[4] * left[8];
    acc += left[5] * left[7];
    temp_12 += left[6] * left[6];
    temp_12 += (acc << 1);
    acc = 0;
    acc += left[5] * left[8];
    acc += left[6] * left[7];
    temp_13 += (acc << 1);
    acc = 0;
    acc += left[6] * left[8];
    temp_14 += left[7] * left[7];
    temp_14 += (acc << 1);
    acc = 0;
    acc += left[7] * left[8];
    temp_15 += (acc << 1);
    temp_16 += left[8] * left[8];

    // Perform reductions
    wasm_reduce(temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    wasm_reduce(temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_reduce(temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_reduce(temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_reduce(temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_reduce(temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_reduce(temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_reduce(temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_reduce(temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);

    // Convert to unrelaxed 29-bit form
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
    // Convert to 4 64-bit form
    return { (temp_9 << 0) | (temp_10 << 29) | (temp_11 << 58),
             (temp_11 >> 6) | (temp_12 << 23) | (temp_13 << 52),
             (temp_13 >> 12) | (temp_14 << 17) | (temp_15 << 46),
             (temp_15 >> 18) | (temp_16 << 11) };
#endif
}

template <class T> constexpr struct field<T>::wide_array field<T>::mul_512(const field& other) const noexcept {
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint64_t carry_2 = 0;
    auto [r0, carry] = mul_wide(data[0], other.data[0]);
    uint64_t r1 = mac_mini(carry, data[0], other.data[1], carry);
    uint64_t r2 = mac_mini(carry, data[0], other.data[2], carry);
    uint64_t r3 = mac_mini(carry, data[0], other.data[3], carry_2);

    r1 = mac_mini(r1, data[1], other.data[0], carry);
    r2 = mac(r2, data[1], other.data[1], carry, carry);
    r3 = mac(r3, data[1], other.data[2], carry, carry);
    uint64_t r4 = mac(carry_2, data[1], other.data[3], carry, carry_2);

    r2 = mac_mini(r2, data[2], other.data[0], carry);
    r3 = mac(r3, data[2], other.data[1], carry, carry);
    r4 = mac(r4, data[2], other.data[2], carry, carry);
    uint64_t r5 = mac(carry_2, data[2], other.data[3], carry, carry_2);

    r3 = mac_mini(r3, data[3], other.data[0], carry);
    r4 = mac(r4, data[3], other.data[1], carry, carry);
    r5 = mac(r5, data[3], other.data[2], carry, carry);
    uint64_t r6 = mac(carry_2, data[3], other.data[3], carry, carry_2);

    return { r0, r1, r2, r3, r4, r5, r6, carry_2 };
#else
    // Convert from 4 64-bit limbs to 9 29-bit limbs
    auto left = wasm_convert(data);
    auto right = wasm_convert(other.data);
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

    // Multiply-add all limbs
    wasm_madd(left[0], right, temp_0, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8);
    wasm_madd(left[1], right, temp_1, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9);
    wasm_madd(left[2], right, temp_2, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10);
    wasm_madd(left[3], right, temp_3, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11);
    wasm_madd(left[4], right, temp_4, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12);
    wasm_madd(left[5], right, temp_5, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13);
    wasm_madd(left[6], right, temp_6, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14);
    wasm_madd(left[7], right, temp_7, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15);
    wasm_madd(left[8], right, temp_8, temp_9, temp_10, temp_11, temp_12, temp_13, temp_14, temp_15, temp_16);

    // Convert to unrelaxed 29-bit form
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

    // Convert to 8 64-bit limbs
    return { (temp_0 << 0) | (temp_1 << 29) | (temp_2 << 58),
             (temp_2 >> 6) | (temp_3 << 23) | (temp_4 << 52),
             (temp_4 >> 12) | (temp_5 << 17) | (temp_6 << 46),
             (temp_6 >> 18) | (temp_7 << 11) | (temp_8 << 40),
             (temp_8 >> 24) | (temp_9 << 5) | (temp_10 << 34) | (temp_11 << 63),
             (temp_11 >> 1) | (temp_12 << 28) | (temp_13 << 57),
             (temp_13 >> 7) | (temp_14 << 22) | (temp_15 << 51),
             (temp_15 >> 13) | (temp_16 << 16) };
#endif
}

// NOLINTEND(readability-implicit-bool-conversion)
} // namespace bb