#pragma once

constexpr std::pair<uint512_t, uint512_t> uint512_t::divmod(const uint512_t& b) const
{
    if (*this == 0 || b == 0) {
        return { uint512_t(0), uint512_t(0) };
    } else if (b == 1) {
        return { *this, uint512_t(0) };
    } else if (*this == b) {
        return { uint512_t(1), uint512_t(0) };
    } else if (b > *this) {
        return { uint512_t(0), *this };
    }

    uint512_t quotient = 0;
    uint512_t remainder = *this;

    uint64_t bit_difference = get_msb() - b.get_msb();

    uint512_t divisor = b << bit_difference;
    uint512_t accumulator = uint512_t(1) << bit_difference;

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
 * computes the inverse of *this, modulo modulus, via the extended Euclidean algorithm
 **/
constexpr uint512_t uint512_t::invmod(const uint512_t& modulus) const
{
    if (*this == 0 || modulus == 0) {
        return 0;
    }

    uint512_t t1 = 0;
    uint512_t t2 = 1;
    uint512_t r2 = (*this > modulus) ? *this % modulus : *this;
    uint512_t r1 = modulus;
    uint512_t q = 0;
    while (r2 != 0) {
        q = r1 / r2;
        uint512_t temp_t1 = t1;
        uint512_t temp_r1 = r1;
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

constexpr bool uint512_t::get_bit(const uint64_t bit_index) const
{
    if (bit_index >= 256) {
        return hi.get_bit(bit_index - 256);
    }
    return lo.get_bit(bit_index);
}

constexpr uint64_t uint512_t::get_msb() const
{
    uint64_t hi_idx = hi.get_msb();
    uint64_t lo_idx = lo.get_msb();
    return (hi_idx || (hi > uint256_t(0))) ? (hi_idx + 256) : lo_idx;
}

constexpr uint512_t uint512_t::operator+(const uint512_t& other) const
{
    uint256_t res_lo = lo + other.lo;
    bool carry = res_lo < lo;
    uint256_t res_hi = hi + other.hi + ((carry == true) ? uint256_t(1) : uint256_t(0));
    return { res_lo, res_hi };
};

constexpr uint512_t uint512_t::operator-(const uint512_t& other) const
{
    uint256_t res_lo = lo - other.lo;
    bool borrow = res_lo > lo;
    uint256_t res_hi = hi - other.hi - ((borrow == true) ? uint256_t(1) : uint256_t(0));
    return { res_lo, res_hi };
}

constexpr uint512_t uint512_t::operator-() const
{
    return uint512_t(0) - *this;
}

constexpr uint512_t uint512_t::operator*(const uint512_t& other) const
{
    const auto lolo = lo.mul_512(other.lo);
    const auto lohi = lo.mul_512(other.hi);
    const auto hilo = hi.mul_512(other.lo);

    uint256_t top = lolo.second + hilo.first + lohi.first;
    uint256_t bottom = lolo.first;
    return { bottom, top };
}

constexpr uint512_t uint512_t::operator/(const uint512_t& other) const
{
    return divmod(other).first;
}

constexpr uint512_t uint512_t::operator%(const uint512_t& other) const
{
    return divmod(other).second;
}

constexpr uint512_t uint512_t::operator&(const uint512_t& other) const
{
    return { lo & other.lo, hi & other.hi };
}

constexpr uint512_t uint512_t::operator^(const uint512_t& other) const
{
    return { lo ^ other.lo, hi ^ other.hi };
}

constexpr uint512_t uint512_t::operator|(const uint512_t& other) const
{
    return { lo | other.lo, hi | other.hi };
}

constexpr uint512_t uint512_t::operator~() const
{
    return { ~lo, ~hi };
}

constexpr bool uint512_t::operator==(const uint512_t& other) const
{
    return ((lo == other.lo) && (hi == other.hi));
}

constexpr bool uint512_t::operator!=(const uint512_t& other) const
{
    return !(*this == other);
}

constexpr bool uint512_t::operator!() const
{
    return *this == uint512_t(0ULL);
}

constexpr bool uint512_t::operator>(const uint512_t& other) const
{
    bool hi_gt = hi > other.hi;
    bool lo_gt = lo > other.lo;

    bool gt = hi_gt || (lo_gt && (hi == other.hi));
    return gt;
}

constexpr bool uint512_t::operator>=(const uint512_t& other) const
{
    return (*this > other) || (*this == other);
}

constexpr bool uint512_t::operator<(const uint512_t& other) const
{
    return other > *this;
}

constexpr bool uint512_t::operator<=(const uint512_t& other) const
{
    return (*this < other) || (*this == other);
}

constexpr uint512_t uint512_t::operator>>(const uint256_t& other) const
{
    uint64_t total_shift = other.data[0];
    if (total_shift >= 512) {
        return uint512_t(0);
    }
    if (total_shift == 0) {
        return *this;
    }
    uint64_t num_shifted_limbs = total_shift >> 8ULL;
    uint64_t limb_shift = total_shift & 255ULL;

    uint256_t shifted_limbs[2] = { 0 };
    if (limb_shift == 0) {
        shifted_limbs[0] = lo;
        shifted_limbs[1] = hi;
    } else {
        uint256_t remainder_shift = 256ULL - limb_shift;

        shifted_limbs[1] = hi >> limb_shift;

        uint256_t remainder = (hi) << remainder_shift;

        shifted_limbs[0] = (lo >> limb_shift) + remainder;
    }
    uint512_t result(0);
    if (num_shifted_limbs == 0) {
        result.hi = shifted_limbs[1];
        result.lo = shifted_limbs[0];
    } else {
        result.lo = shifted_limbs[1];
    }
    return result;
}

constexpr uint512_t uint512_t::operator<<(const uint256_t& other) const
{
    uint64_t total_shift = other.data[0];
    if (total_shift >= 512) {
        return uint512_t(0);
    }
    if (total_shift == 0) {
        return *this;
    }
    uint64_t num_shifted_limbs = total_shift >> 8ULL;
    uint64_t limb_shift = total_shift & 255ULL;

    uint256_t shifted_limbs[2] = { 0 };
    if (limb_shift == 0) {
        shifted_limbs[0] = lo;
        shifted_limbs[1] = hi;
    } else {
        uint256_t remainder_shift = 256ULL - limb_shift;

        shifted_limbs[0] = lo << limb_shift;

        uint256_t remainder = lo >> remainder_shift;

        shifted_limbs[1] = (hi << limb_shift) + remainder;
    }
    uint512_t result(0);
    if (num_shifted_limbs == 0) {
        result.hi = shifted_limbs[1];
        result.lo = shifted_limbs[0];
    } else {
        result.hi = shifted_limbs[0];
    }
    return result;
}