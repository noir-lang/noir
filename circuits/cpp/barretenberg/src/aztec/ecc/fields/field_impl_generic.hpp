#pragma once

namespace barretenberg {

// constexpr void mac_wasm(const uint64_t a, const uint64_t b, const uint64_t c, const uint64_t carr)

template <class T>
constexpr std::pair<uint64_t, uint64_t> field<T>::mul_wide(const uint64_t a, const uint64_t b) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = ((uint128_t)a * (uint128_t)b);
    return { (uint64_t)(res), (uint64_t)(res >> 64) };
#else
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
    auto result = mul_wide(b, c);
    result.first += a;
    const uint64_t overflow_c = (result.first < a);
    result.first += carry_in;
    const uint64_t overflow_carry = (result.first < carry_in);
    carry_out = result.second + (overflow_c + overflow_carry);
    return result.first;
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
    auto result = mul_wide(b, c);
    result.first += a;
    const uint64_t overflow_c = (result.first < a);
    result.first += carry_in;
    const uint64_t overflow_carry = (result.first < carry_in);
    carry_out = result.second + (overflow_c + overflow_carry);
    out = result.first;
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
    auto result = mul_wide(b, c);
    result.first += a;
    const uint64_t overflow_c = (result.first < a);
    carry_out = result.second + (overflow_c);
    return result.first;
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
    auto result = mul_wide(b, c);
    result.first += a;
    const uint64_t overflow_c = (result.first < a);
    carry_out = result.second + (overflow_c);
    out = result.first;
#endif
}

template <class T>
constexpr uint64_t field<T>::mac_discard_lo(const uint64_t a, const uint64_t b, const uint64_t c) noexcept
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    const uint128_t res = (uint128_t)a + ((uint128_t)b * (uint128_t)c);
    return (uint64_t)(res >> 64);
#else
    auto result = mul_wide(b, c);
    result.first += a;
    const uint64_t overflow_c = (result.first < a);
    result.second += (overflow_c);
    return result.second;
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
constexpr void field<T>::square_accumulate(const uint64_t a,
                                           const uint64_t b,
                                           const uint64_t c,
                                           const uint64_t carry_in_lo,
                                           const uint64_t carry_in_hi,
                                           uint64_t& out,
                                           uint64_t& carry_lo,
                                           uint64_t& carry_hi) noexcept
{
    const auto [r0, r1] = mul_wide(b, c);
    out = r0 + r0;
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
        // std::cout << "beep" << std::endl;
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
    r3 += (modulus.data[3] & borrow) + carry;

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
    uint64_t c = 0;
    uint64_t t0 = 0;
    uint64_t t1 = 0;
    uint64_t t2 = 0;
    uint64_t t3 = 0;
    uint64_t t4 = 0;
    uint64_t t5 = 0;
    uint64_t k = 0;
    // if (!std::is_constant_evaluated()) {
    //     std::cout << "r_inv = " << std::hex << T::r_inv << std::dec << std::endl;
    // }
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
    // if (!std::is_constant_evaluated()) {
    //     std::cout << "t4, t5 = " << std::hex << t4 << ", " << t5 << std::dec << std::endl;
    // }
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

    // if (t4) {
    //     uint64_t borrow = 0;
    //     t0 = sbb(t0, modulus.data[0], borrow, borrow);
    //     t1 = sbb(t1, modulus.data[1], borrow, borrow);
    //     t2 = sbb(t2, modulus.data[2], borrow, borrow);
    //     t3 = sbb(t3, modulus.data[3], borrow, borrow);
    // }
    return { r0, r1, r2, r3 };
}

template <class T> constexpr field<T> field<T>::montgomery_mul(const field& other) const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        return montgomery_mul_big(other);
    }
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
}

template <class T> constexpr field<T> field<T>::montgomery_square() const noexcept
{
    if constexpr (modulus.data[3] >= 0x4000000000000000ULL) {
        return montgomery_mul_big(*this);
    }
    uint64_t t1 = 0;
    uint64_t t2 = 0;
    uint64_t t3 = 0;
    uint64_t carry_hi = 0;

    auto [t0, carry_lo] = mul_wide(data[0], data[0]);
    square_accumulate(t1, data[1], data[0], carry_lo, carry_hi, t1, carry_lo, carry_hi);
    square_accumulate(t2, data[2], data[0], carry_lo, carry_hi, t2, carry_lo, carry_hi);
    square_accumulate(t3, data[3], data[0], carry_lo, carry_hi, t3, carry_lo, carry_hi);

    uint64_t round_carry = carry_lo;
    uint64_t k = t0 * T::r_inv;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;

    t1 = mac_mini(t1, data[1], data[1], carry_lo);
    carry_hi = 0;
    square_accumulate(t2, data[2], data[1], carry_lo, carry_hi, t2, carry_lo, carry_hi);
    square_accumulate(t3, data[3], data[1], carry_lo, carry_hi, t3, carry_lo, carry_hi);
    round_carry = carry_lo;
    k = t0 * T::r_inv;
    carry_lo = mac_discard_lo(t0, k, modulus.data[0]);
    mac(t1, k, modulus.data[1], carry_lo, t0, carry_lo);
    mac(t2, k, modulus.data[2], carry_lo, t1, carry_lo);
    mac(t3, k, modulus.data[3], carry_lo, t2, carry_lo);
    t3 = carry_lo + round_carry;

    t2 = mac_mini(t2, data[2], data[2], carry_lo);
    carry_hi = 0;
    square_accumulate(t3, data[3], data[2], carry_lo, carry_hi, t3, carry_lo, carry_hi);
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
}

template <class T> constexpr struct field<T>::wide_array field<T>::mul_512(const field& other) const noexcept {
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
}

} // namespace barretenberg