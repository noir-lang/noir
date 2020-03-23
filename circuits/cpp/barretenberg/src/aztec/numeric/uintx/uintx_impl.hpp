#pragma once

template <class base_uint>
constexpr std::pair<uintx<base_uint>, uintx<base_uint>> uintx<base_uint>::divmod(const uintx& b) const
{
    if (*this == 0 || b == 0) {
        return { uintx(0), uintx(0) };
    } else if (b == 1) {
        return { *this, uintx(0) };
    } else if (*this == b) {
        return { uintx(1), uintx(0) };
    } else if (b > *this) {
        return { uintx(0), *this };
    }

    uintx quotient(0);
    uintx remainder = *this;

    uint64_t bit_difference = get_msb() - b.get_msb();

    uintx divisor = b << bit_difference;
    uintx accumulator = uintx(1) << bit_difference;

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

    return std::make_pair(quotient, remainder);
}

/**
 * computes the inverse of *this, modulo modulus, via the extended Euclidean algorithm
 **/
template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::invmod(const uintx& modulus) const
{
    if (*this == 0 || modulus == 0) {
        return 0;
    }

    uintx t1 = 0;
    uintx t2 = 1;
    uintx r2 = (*this > modulus) ? *this % modulus : *this;
    uintx r1 = modulus;
    uintx q = 0;
    while (r2 != 0) {
        q = r1 / r2;
        uintx temp_t1 = t1;
        uintx temp_r1 = r1;
        t1 = t2;
        t2 = temp_t1 - q * t2;
        r1 = r2;
        r2 = temp_r1 - q * r2;
    }

    if (t1 > modulus) {
        return modulus + t1;
    }
    return t1;
}

template <class base_uint>
constexpr uintx<base_uint> uintx<base_uint>::slice(const uint64_t start, const uint64_t end) const
{
    const uint64_t range = end - start;
    const uintx mask = range == base_uint::length() ? -uintx(1) : (uintx(1) << range) - 1;
    return ((*this) >> start) & mask;
}

template <class base_uint> constexpr bool uintx<base_uint>::get_bit(const uint64_t bit_index) const
{
    if (bit_index >= base_uint::length()) {
        return hi.get_bit(bit_index - base_uint::length());
    }
    return lo.get_bit(bit_index);
}

template <class base_uint> constexpr uint64_t uintx<base_uint>::get_msb() const
{
    uint64_t hi_idx = hi.get_msb();
    uint64_t lo_idx = lo.get_msb();
    return (hi_idx || (hi > base_uint(0))) ? (hi_idx + base_uint::length()) : lo_idx;
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator+(const uintx& other) const
{
    base_uint res_lo = lo + other.lo;
    bool carry = res_lo < lo;
    base_uint res_hi = hi + other.hi + ((carry == true) ? base_uint(1) : base_uint(0));
    return { res_lo, res_hi };
};

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator-(const uintx& other) const
{
    base_uint res_lo = lo - other.lo;
    bool borrow = res_lo > lo;
    base_uint res_hi = hi - other.hi - ((borrow == true) ? base_uint(1) : base_uint(0));
    return { res_lo, res_hi };
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator-() const
{
    return uintx(0) - *this;
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator*(const uintx& other) const
{
    const auto lolo = lo.mul_extended(other.lo);
    const auto lohi = lo.mul_extended(other.hi);
    const auto hilo = hi.mul_extended(other.lo);

    base_uint top = lolo.second + hilo.first + lohi.first;
    base_uint bottom = lolo.first;
    return { bottom, top };
}

template <class base_uint>
constexpr std::pair<uintx<base_uint>, uintx<base_uint>> uintx<base_uint>::mul_extended(const uintx& other) const
{
    const auto lolo = lo.mul_extended(other.lo);
    const auto lohi = lo.mul_extended(other.hi);
    const auto hilo = hi.mul_extended(other.lo);
    const auto hihi = hi.mul_extended(other.hi);

    base_uint t0 = lolo.first;
    base_uint t1 = lolo.second;
    base_uint t2 = hilo.second;
    base_uint t3 = hihi.second;
    base_uint t2_carry(0);
    base_uint t3_carry(0);
    t1 += hilo.first;
    t2_carry += (t1 < hilo.first ? base_uint(1) : base_uint(0));
    t1 += lohi.first;
    t2_carry += (t1 < lohi.first ? base_uint(1) : base_uint(0));
    t2 += lohi.second;
    t3_carry += (t2 < lohi.second ? base_uint(1) : base_uint(0));
    t2 += hihi.first;
    t3_carry += (t2 < hihi.first ? base_uint(1) : base_uint(0));
    t2 += t2_carry;
    t3_carry += (t2 < t2_carry ? base_uint(1) : base_uint(0));
    t3 += t3_carry;
    return { uintx(t0, t1), uintx(t2, t3) };
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator/(const uintx& other) const
{
    return divmod(other).first;
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator%(const uintx& other) const
{
    return divmod(other).second;
}
// 0x2af0296feca4188a80fd373ebe3c64da87a232934abb3a99f9c4cd59e6758a65
// 0x1182c6cdb54193b51ca27c1932b95c82bebac691e3996e5ec5e1d4395f3023e3
template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator&(const uintx& other) const
{
    return { lo & other.lo, hi & other.hi };
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator^(const uintx& other) const
{
    return { lo ^ other.lo, hi ^ other.hi };
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator|(const uintx& other) const
{
    return { lo | other.lo, hi | other.hi };
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator~() const
{
    return { ~lo, ~hi };
}

template <class base_uint> constexpr bool uintx<base_uint>::operator==(const uintx& other) const
{
    return ((lo == other.lo) && (hi == other.hi));
}

template <class base_uint> constexpr bool uintx<base_uint>::operator!=(const uintx& other) const
{
    return !(*this == other);
}

template <class base_uint> constexpr bool uintx<base_uint>::operator!() const
{
    return *this == uintx(0ULL);
}

template <class base_uint> constexpr bool uintx<base_uint>::operator>(const uintx& other) const
{
    bool hi_gt = hi > other.hi;
    bool lo_gt = lo > other.lo;

    bool gt = (hi_gt) || (lo_gt && (hi == other.hi));
    return gt;
}

template <class base_uint> constexpr bool uintx<base_uint>::operator>=(const uintx& other) const
{
    return (*this > other) || (*this == other);
}

template <class base_uint> constexpr bool uintx<base_uint>::operator<(const uintx& other) const
{
    return other > *this;
}

template <class base_uint> constexpr bool uintx<base_uint>::operator<=(const uintx& other) const
{
    return (*this < other) || (*this == other);
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator>>(const uint64_t other) const
{
    const uint64_t total_shift = other;
    if (total_shift >= length()) {
        return uintx(0);
    }
    if (total_shift == 0) {
        return *this;
    }
    const uint64_t num_shifted_limbs = total_shift >> (base_uint(base_uint::length()).get_msb());

    const uint64_t limb_shift = total_shift & static_cast<uint64_t>(base_uint::length() - 1);

    base_uint shifted_limbs[2] = { 0 };
    if (limb_shift == 0) {
        shifted_limbs[0] = lo;
        shifted_limbs[1] = hi;
    } else {
        const uint64_t remainder_shift = static_cast<uint64_t>(base_uint::length()) - limb_shift;

        shifted_limbs[1] = hi >> limb_shift;

        base_uint remainder = (hi) << remainder_shift;

        shifted_limbs[0] = (lo >> limb_shift) + remainder;
    }
    uintx result(0);
    if (num_shifted_limbs == 0) {
        result.hi = shifted_limbs[1];
        result.lo = shifted_limbs[0];
    } else {
        result.lo = shifted_limbs[1];
    }
    return result;
}

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator<<(const uint64_t other) const
{
    const uint64_t total_shift = other;
    if (total_shift >= length()) {
        return uintx(0);
    }
    if (total_shift == 0) {
        return *this;
    }
    const uint64_t num_shifted_limbs = total_shift >> (base_uint(base_uint::length()).get_msb());
    const uint64_t limb_shift = total_shift & static_cast<uint64_t>(base_uint::length() - 1);

    base_uint shifted_limbs[2] = { 0 };
    if (limb_shift == 0) {
        shifted_limbs[0] = lo;
        shifted_limbs[1] = hi;
    } else {
        const uint64_t remainder_shift = static_cast<uint64_t>(base_uint::length()) - limb_shift;

        shifted_limbs[0] = lo << limb_shift;

        base_uint remainder = lo >> remainder_shift;

        shifted_limbs[1] = (hi << limb_shift) + remainder;
    }
    uintx result(0);
    if (num_shifted_limbs == 0) {
        result.hi = shifted_limbs[1];
        result.lo = shifted_limbs[0];
    } else {
        result.hi = shifted_limbs[0];
    }
    return result;
}