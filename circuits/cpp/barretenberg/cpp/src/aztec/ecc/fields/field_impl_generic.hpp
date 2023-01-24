#pragma once

namespace barretenberg {

template <class T>
constexpr std::pair<uint64_t, uint64_t> field<T>::mul_wide(const uint64_t a, const uint64_t b) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = ((uint128_t)a * (uint128_t)b);
    return { (uint64_t)(res), (uint64_t)(res >> 64) };
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
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c) + (uint128_t)carry_in;
    carry_out = (uint64_t)(res >> 64);
    return (uint64_t)res;
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
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c) + (uint128_t)carry_in;
    out = (uint64_t)(res);
    carry_out = (uint64_t)(res >> 64);
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
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c);
    carry_out = (uint64_t)(res >> 64);
    return (uint64_t)(res);
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
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c);
    out = (uint64_t)(res);
    carry_out = (uint64_t)(res >> 64);
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
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c);
    return (uint64_t)(res >> 64);
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
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    uint128_t res = (uint128_t)a + (uint128_t)b + (uint128_t)carry_in;
    carry_out = (uint64_t)(res >> 64);
    return (uint64_t)res;
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
    uint128_t res = (uint128_t)a - ((uint128_t)b + (uint128_t)(borrow_in >> 63));
    borrow_out = (uint64_t)(res >> 64);
    return (uint64_t)res;
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
    const uint128_t product = (uint128_t)b * (uint128_t)c;
    const uint64_t r0 = (uint64_t)product;
    const uint64_t r1 = (uint64_t)(product >> 64);
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
    for (size_t i = 0; i < 4; ++i) {
        c = 0;
        mac(t0, data[i], other.data[0], c, t0, c);
        mac(t1, data[i], other.data[1], c, t1, c);
        mac(t2, data[i], other.data[2], c, t2, c);
        mac(t3, data[i], other.data[3], c, t3, c);
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
    uint64_t c = 0;
    uint64_t t0 = 0;
    uint64_t t1 = 0;
    uint64_t t2 = 0;
    uint64_t t3 = 0;
    uint64_t t4 = 0;
    uint64_t t5 = 0;
    uint64_t t6 = 0;
    uint64_t t7 = 0;
    uint64_t t8 = 0;
    uint64_t t9 = 0;
    uint64_t k = 0;

    constexpr uint64_t wasm_modulus[8]{
        modulus.data[0] & 0xffffffffULL, modulus.data[0] >> 32ULL,        modulus.data[1] & 0xffffffffULL,
        modulus.data[1] >> 32ULL,        modulus.data[2] & 0xffffffffULL, modulus.data[2] >> 32ULL,
        modulus.data[3] & 0xffffffffULL, modulus.data[3] >> 32ULL,
    };
    constexpr uint64_t wasm_rinv = T::r_inv & 0xffffffffULL;

    const uint64_t left[8]{
        data[0] & 0xffffffffULL, data[0] >> 32, data[1] & 0xffffffffULL, data[1] >> 32,
        data[2] & 0xffffffffULL, data[2] >> 32, data[3] & 0xffffffffULL, data[3] >> 32,
    };
    const uint64_t right[8]{
        other.data[0] & 0xffffffffULL, other.data[0] >> 32, other.data[1] & 0xffffffffULL, other.data[1] >> 32,
        other.data[2] & 0xffffffffULL, other.data[2] >> 32, other.data[3] & 0xffffffffULL, other.data[3] >> 32,
    };

    for (size_t i = 0; i < 8; ++i) {
        c = 0;
        mac(t0, left[i], right[0], c, t0, c);
        mac(t1, left[i], right[1], c, t1, c);
        mac(t2, left[i], right[2], c, t2, c);
        mac(t3, left[i], right[3], c, t3, c);
        mac(t4, left[i], right[4], c, t4, c);
        mac(t5, left[i], right[5], c, t5, c);
        mac(t6, left[i], right[6], c, t6, c);
        mac(t7, left[i], right[7], c, t7, c);
        uint64_t end_mul = t8 + c;
        t8 = end_mul & 0xffffffffU;
        t9 = end_mul >> 32;

        c = 0;
        k = (t0 * wasm_rinv) & 0xffffffffU;
        c = mac_discard_lo(t0, k, wasm_modulus[0]);
        mac(t1, k, wasm_modulus[1], c, t0, c);
        mac(t2, k, wasm_modulus[2], c, t1, c);
        mac(t3, k, wasm_modulus[3], c, t2, c);
        mac(t4, k, wasm_modulus[4], c, t3, c);
        mac(t5, k, wasm_modulus[5], c, t4, c);
        mac(t6, k, wasm_modulus[6], c, t5, c);
        mac(t7, k, wasm_modulus[7], c, t6, c);
        uint64_t end_reduce = c + t8;
        t7 = end_reduce & 0xffffffffU;
        c = end_reduce >> 32;
        t8 = t9 + c;
    }
    uint64_t v0 = t0 + (t1 << 32);
    uint64_t v1 = t2 + (t3 << 32);
    uint64_t v2 = t4 + (t5 << 32);
    uint64_t v3 = t6 + (t7 << 32);
    uint64_t v4 = t8;
    uint64_t borrow = 0;
    uint64_t r0 = sbb(v0, modulus.data[0], borrow, borrow);
    uint64_t r1 = sbb(v1, modulus.data[1], borrow, borrow);
    uint64_t r2 = sbb(v2, modulus.data[2], borrow, borrow);
    uint64_t r3 = sbb(v3, modulus.data[3], borrow, borrow);
    borrow = borrow ^ (0ULL - v4);
    r0 += (modulus.data[0] & borrow);
    uint64_t carry = r0 < (modulus.data[0] & borrow);
    r1 = addc(r1, modulus.data[1] & borrow, carry, carry);
    r2 = addc(r2, modulus.data[2] & borrow, carry, carry);
    r3 += (modulus.data[3] & borrow) + carry;
    return { r0, r1, r2, r3 };
#endif
}

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
    constexpr uint64_t wasm_modulus[8]{
        modulus.data[0] & 0xffffffffULL, modulus.data[0] >> 32ULL,        modulus.data[1] & 0xffffffffULL,
        modulus.data[1] >> 32ULL,        modulus.data[2] & 0xffffffffULL, modulus.data[2] >> 32ULL,
        modulus.data[3] & 0xffffffffULL, modulus.data[3] >> 32ULL,
    };
    constexpr uint64_t wasm_rinv = T::r_inv & 0xffffffffULL;

    const uint64_t left[8]{
        data[0] & 0xffffffffULL, data[0] >> 32, data[1] & 0xffffffffULL, data[1] >> 32,
        data[2] & 0xffffffffULL, data[2] >> 32, data[3] & 0xffffffffULL, data[3] >> 32,
    };
    const uint64_t right[8]{
        other.data[0] & 0xffffffffULL, other.data[0] >> 32, other.data[1] & 0xffffffffULL, other.data[1] >> 32,
        other.data[2] & 0xffffffffULL, other.data[2] >> 32, other.data[3] & 0xffffffffULL, other.data[3] >> 32,
    };

    auto [t0, c] = mul_wide(left[0], right[0]);
    uint64_t k = (t0 * wasm_rinv) & 0xffffffffULL;
    uint64_t a = mac_discard_lo(t0, k, wasm_modulus[0]);

    uint64_t t1 = mac_mini(a, left[0], right[1], a);
    mac(t1, k, wasm_modulus[1], c, t0, c);
    uint64_t t2 = mac_mini(a, left[0], right[2], a);
    mac(t2, k, wasm_modulus[2], c, t1, c);
    uint64_t t3 = mac_mini(a, left[0], right[3], a);
    mac(t3, k, wasm_modulus[3], c, t2, c);
    uint64_t t4 = mac_mini(a, left[0], right[4], a);
    mac(t4, k, wasm_modulus[4], c, t3, c);
    uint64_t t5 = mac_mini(a, left[0], right[5], a);
    mac(t5, k, wasm_modulus[5], c, t4, c);
    uint64_t t6 = mac_mini(a, left[0], right[6], a);
    mac(t6, k, wasm_modulus[6], c, t5, c);
    uint64_t t7 = mac_mini(a, left[0], right[7], a);
    mac(t7, k, wasm_modulus[7], c, t6, c);
    t7 = c + a;

    for (size_t i = 1; i < 8; ++i) {
        mac_mini(t0, left[i], right[0], t0, a);
        k = (t0 * wasm_rinv) & 0xffffffffULL;
        c = mac_discard_lo(t0, k, wasm_modulus[0]);
        mac(t1, left[i], right[1], a, t1, a);
        mac(t1, k, wasm_modulus[1], c, t0, c);
        mac(t2, left[i], right[2], a, t2, a);
        mac(t2, k, wasm_modulus[2], c, t1, c);
        mac(t3, left[i], right[3], a, t3, a);
        mac(t3, k, wasm_modulus[3], c, t2, c);
        mac(t4, left[i], right[4], a, t4, a);
        mac(t4, k, wasm_modulus[4], c, t3, c);
        mac(t5, left[i], right[5], a, t5, a);
        mac(t5, k, wasm_modulus[5], c, t4, c);
        mac(t6, left[i], right[6], a, t6, a);
        mac(t6, k, wasm_modulus[6], c, t5, c);
        mac(t7, left[i], right[7], a, t7, a);
        mac(t7, k, wasm_modulus[7], c, t6, c);
        t7 = c + a;
    }

    // mac_mini(t0, left[2], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[2], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[2], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[2], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[2], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[2], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[2], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[2], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    // mac_mini(t0, left[3], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[3], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[3], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[3], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[3], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[3], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[3], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[3], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    // mac_mini(t0, left[4], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[4], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[4], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[4], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[4], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[4], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[4], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[4], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    // mac_mini(t0, left[5], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[5], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[5], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[5], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[5], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[5], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[5], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[5], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    // mac_mini(t0, left[6], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[6], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[6], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[6], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[6], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[6], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[6], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[6], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    // mac_mini(t0, left[7], right[0], t0, a);
    // k = (t0 * wasm_rinv) & 0xffffffffULL;
    // c = mac_discard_lo(t0, k, wasm_modulus[0]);
    // mac(t1, left[7], right[1], a, t1, a);
    // mac(t1, k, wasm_modulus[1], c, t0, c);
    // mac(t2, left[7], right[2], a, t2, a);
    // mac(t2, k, wasm_modulus[2], c, t1, c);
    // mac(t3, left[7], right[3], a, t3, a);
    // mac(t3, k, wasm_modulus[3], c, t2, c);
    // mac(t4, left[7], right[4], a, t4, a);
    // mac(t4, k, wasm_modulus[4], c, t3, c);
    // mac(t5, left[7], right[5], a, t5, a);
    // mac(t5, k, wasm_modulus[5], c, t4, c);
    // mac(t6, left[7], right[6], a, t6, a);
    // mac(t6, k, wasm_modulus[6], c, t5, c);
    // mac(t7, left[7], right[7], a, t7, a);
    // mac(t7, k, wasm_modulus[7], c, t6, c);
    // t7 = c + a;

    return { t0 + (t1 << 32), t2 + (t3 << 32), t4 + (t5 << 32), t6 + (t7 << 32) };
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
    // TODO: apparently the plain old 'mul' operation is faster than the squaring code.
    // `square_accumulate` has too many additions and comparisons, to justify the saved multiplications
    return montgomery_mul(*this);
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
    const uint64_t left[8]{
        data[0] & 0xffffffffULL, data[0] >> 32, data[1] & 0xffffffffULL, data[1] >> 32,
        data[2] & 0xffffffffULL, data[2] >> 32, data[3] & 0xffffffffULL, data[3] >> 32,
    };

    const uint64_t right[8]{
        other.data[0] & 0xffffffffULL, other.data[0] >> 32, other.data[1] & 0xffffffffULL, other.data[1] >> 32,
        other.data[2] & 0xffffffffULL, other.data[2] >> 32, other.data[3] & 0xffffffffULL, other.data[3] >> 32,
    };

    uint64_t carry_2 = 0;
    auto [r0, carry] = mul_wide(left[0], right[0]);
    uint64_t r1 = mac_mini(carry, left[0], right[1], carry);
    uint64_t r2 = mac_mini(carry, left[0], right[2], carry);
    uint64_t r3 = mac_mini(carry, left[0], right[3], carry);
    uint64_t r4 = mac_mini(carry, left[0], right[4], carry);
    uint64_t r5 = mac_mini(carry, left[0], right[5], carry);
    uint64_t r6 = mac_mini(carry, left[0], right[6], carry);
    uint64_t r7 = mac_mini(carry, left[0], right[7], carry_2);

    r1 = mac_mini(r1, left[1], right[0], carry);
    r2 = mac(r2, left[1], right[1], carry, carry);
    r3 = mac(r3, left[1], right[2], carry, carry);
    r4 = mac(r4, left[1], right[3], carry, carry);
    r5 = mac(r5, left[1], right[4], carry, carry);
    r6 = mac(r6, left[1], right[5], carry, carry);
    r7 = mac(r7, left[1], right[6], carry, carry);
    uint64_t r8 = mac(carry_2, left[1], right[7], carry, carry_2);

    r2 = mac_mini(r2, left[2], right[0], carry);
    r3 = mac(r3, left[2], right[1], carry, carry);
    r4 = mac(r4, left[2], right[2], carry, carry);
    r5 = mac(r5, left[2], right[3], carry, carry);
    r6 = mac(r6, left[2], right[4], carry, carry);
    r7 = mac(r7, left[2], right[5], carry, carry);
    r8 = mac(r8, left[2], right[6], carry, carry);
    uint64_t r9 = mac(carry_2, left[2], right[7], carry, carry_2);

    r3 = mac_mini(r3, left[3], right[0], carry);
    r4 = mac(r4, left[3], right[1], carry, carry);
    r5 = mac(r5, left[3], right[2], carry, carry);
    r6 = mac(r6, left[3], right[3], carry, carry);
    r7 = mac(r7, left[3], right[4], carry, carry);
    r8 = mac(r8, left[3], right[5], carry, carry);
    r9 = mac(r9, left[3], right[6], carry, carry);
    uint64_t r10 = mac(carry_2, left[3], right[7], carry, carry_2);

    r4 = mac_mini(r4, left[4], right[0], carry);
    r5 = mac(r5, left[4], right[1], carry, carry);
    r6 = mac(r6, left[4], right[2], carry, carry);
    r7 = mac(r7, left[4], right[3], carry, carry);
    r8 = mac(r8, left[4], right[4], carry, carry);
    r9 = mac(r9, left[4], right[5], carry, carry);
    r10 = mac(r10, left[4], right[6], carry, carry);
    uint64_t r11 = mac(carry_2, left[4], right[7], carry, carry_2);

    r5 = mac_mini(r5, left[5], right[0], carry);
    r6 = mac(r6, left[5], right[1], carry, carry);
    r7 = mac(r7, left[5], right[2], carry, carry);
    r8 = mac(r8, left[5], right[3], carry, carry);
    r9 = mac(r9, left[5], right[4], carry, carry);
    r10 = mac(r10, left[5], right[5], carry, carry);
    r11 = mac(r11, left[5], right[6], carry, carry);
    uint64_t r12 = mac(carry_2, left[5], right[7], carry, carry_2);

    r6 = mac_mini(r6, left[6], right[0], carry);
    r7 = mac(r7, left[6], right[1], carry, carry);
    r8 = mac(r8, left[6], right[2], carry, carry);
    r9 = mac(r9, left[6], right[3], carry, carry);
    r10 = mac(r10, left[6], right[4], carry, carry);
    r11 = mac(r11, left[6], right[5], carry, carry);
    r12 = mac(r12, left[6], right[6], carry, carry);
    uint64_t r13 = mac(carry_2, left[6], right[7], carry, carry_2);

    r7 = mac_mini(r7, left[7], right[0], carry);
    r8 = mac(r8, left[7], right[1], carry, carry);
    r9 = mac(r9, left[7], right[2], carry, carry);
    r10 = mac(r10, left[7], right[3], carry, carry);
    r11 = mac(r11, left[7], right[4], carry, carry);
    r12 = mac(r12, left[7], right[5], carry, carry);
    r13 = mac(r13, left[7], right[6], carry, carry);
    uint64_t r14 = mac(carry_2, left[7], right[7], carry, carry_2);

    return {
        r0 + (r1 << 32), r2 + (r3 << 32),   r4 + (r5 << 32),   r6 + (r7 << 32),
        r8 + (r9 << 32), r10 + (r11 << 32), r12 + (r13 << 32), r14 + (carry_2 << 32),
    };
#endif
}

} // namespace barretenberg