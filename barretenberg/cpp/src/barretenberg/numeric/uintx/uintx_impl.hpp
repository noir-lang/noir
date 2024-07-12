#pragma once
#include "./uintx.hpp"
#include "barretenberg/common/assert.hpp"

namespace bb::numeric {
template <class base_uint>
constexpr std::pair<uintx<base_uint>, uintx<base_uint>> uintx<base_uint>::divmod_base(const uintx& b) const
{
    ASSERT(b != 0);
    if (*this == 0) {
        return { uintx(0), uintx(0) };
    }
    if (b == 1) {
        return { *this, uintx(0) };
    }
    if (*this == b) {
        return { uintx(1), uintx(0) };
    }
    if (b > *this) {
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
 * Computes invmod. Only for internal usage within the class.
 * This is an insecure version of the algorithm that doesn't take into account the 0 case and cases when modulus is
 *close to the top margin.
 *
 * @param modulus The modulus of the ring
 *
 * @return The inverse of *this modulo modulus
 **/
template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::unsafe_invmod(const uintx& modulus) const
{

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

/**
 * Computes the inverse of *this, modulo modulus, via the extended Euclidean algorithm.
 *
 * Delegates to appropriate unsafe_invmod (if the modulus is close to uintx top margin there is a need to expand)
 *
 * @param modulus The modulus
 * @return The inverse of *this modulo modulus
 **/
template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::invmod(const uintx& modulus) const
{
    ASSERT((*this) != 0);
    if (modulus == 0) {
        return 0;
    }
    if (modulus.get_msb() >= (2 * base_uint::length() - 1)) {
        uintx<uintx<base_uint>> a_expanded(*this);
        uintx<uintx<base_uint>> modulus_expanded(modulus);
        return a_expanded.unsafe_invmod(modulus_expanded).lo;
    }
    return this->unsafe_invmod(modulus);
}

/**
 * Viewing `this` as a bit string, and counting bits from 0, slices a substring.
 * @returns the uintx equal to the substring of bits from (and including) the `start`-th bit, to (but excluding) the
 * `end`-th bit of `this`.
 */
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
    base_uint res_hi = hi + other.hi + ((carry) ? base_uint(1) : base_uint(0));
    return { res_lo, res_hi };
};

template <class base_uint> constexpr uintx<base_uint> uintx<base_uint>::operator-(const uintx& other) const
{
    base_uint res_lo = lo - other.lo;
    bool borrow = res_lo > lo;
    base_uint res_hi = hi - other.hi - ((borrow) ? base_uint(1) : base_uint(0));
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

    std::array<base_uint, 2> shifted_limbs = { 0, 0 };
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

    std::array<base_uint, 2> shifted_limbs = { 0, 0 };
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

template <class base_uint>
constexpr std::pair<uintx<base_uint>, uintx<base_uint>> uintx<base_uint>::divmod(const uintx& b) const
{
    constexpr uint256_t BN254FQMODULUS256 =
        uint256_t(0x3C208C16D87CFD47UL, 0x97816a916871ca8dUL, 0xb85045b68181585dUL, 0x30644e72e131a029UL);
    constexpr uint256_t SECP256K1FQMODULUS256 =
        uint256_t(0xFFFFFFFEFFFFFC2FULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL);
    constexpr uint256_t SECP256R1FQMODULUS256 =
        uint256_t(0xFFFFFFFFFFFFFFFFULL, 0x00000000FFFFFFFFULL, 0x0000000000000000ULL, 0xFFFFFFFF00000001ULL);

    if (b == uintx(BN254FQMODULUS256)) {
        return (*this).template barrett_reduction<BN254FQMODULUS256>();
    }
    if (b == uintx(SECP256K1FQMODULUS256)) {
        return (*this).template barrett_reduction<SECP256K1FQMODULUS256>();
    }
    if (b == uintx(SECP256R1FQMODULUS256)) {
        return (*this).template barrett_reduction<SECP256R1FQMODULUS256>();
    }

    return divmod_base(b);
}

/**
 * @brief Compute fast division via a barrett reduction
 *        Evaluates x = qm + r where m = modulus. returns q, r
 * @details This implementation is less efficient due to making no assumptions about the value of *self.
 *          When using this method to perform modular reductions e.g. (*self) mod m, if (*self) < m^2 a lot of the
 *          `uintx` operations in this method could be replaced with `base_uint` operations
 *
 * @tparam base_uint
 * @tparam modulus
 * @return constexpr std::pair<uintx<base_uint>, uintx<base_uint>>
 */
template <class base_uint>
template <base_uint modulus>
constexpr std::pair<uintx<base_uint>, uintx<base_uint>> uintx<base_uint>::barrett_reduction() const
{
    // N.B. k could be modulus.get_msb() + 1 if we have strong bounds on the max value of (*self)
    //      (a smaller k would allow us to fit `redc_parameter` into `base_uint` and not `uintx`)
    constexpr size_t k = base_uint::length() - 1;
    // N.B. computation of redc_parameter requires division operation - if this cannot be precomputed (or amortized over
    // multiple reductions over the same modulus), barrett_reduction is much slower than divmod
    constexpr uintx redc_parameter = ((uintx(1) << (k * 2)).divmod_base(uintx(modulus))).first;

    const auto x = *this;

    // compute x * redc_parameter
    const auto mul_result = x.mul_extended(redc_parameter);
    constexpr size_t shift = 2 * k;

    // compute (x * redc_parameter) >> 2k
    // This is equivalent to (x * (2^{2k} / modulus) / 2^{2k})
    // which approximates to x / modulus
    const uintx downshifted_hi_bits = mul_result.second & ((uintx(1) << shift) - 1);
    const uintx mul_hi_underflow = uintx(downshifted_hi_bits) << (length() - shift);
    uintx quotient = (mul_result.first >> shift) | mul_hi_underflow;

    // compute remainder by determining value of x - quotient * modulus
    uintx qm_lo(0);
    {
        const auto lolo = quotient.lo.mul_extended(modulus);
        const auto lohi = quotient.hi.mul_extended(modulus);
        base_uint t0 = lolo.first;
        base_uint t1 = lolo.second;
        t1 = t1 + lohi.first;
        qm_lo = uintx(t0, t1);
    }
    uintx remainder = x - qm_lo;

    // because redc_parameter is an imperfect representation of 2^{2k} / n (might be too small),
    // the computed quotient may be off by up to 3 (classic algorithm should be up to 1,
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1051): investigate, why)
    size_t i = 0;
    while (remainder >= uintx(modulus)) {
        ASSERT(i < 3);
        remainder = remainder - modulus;
        quotient = quotient + 1;
        i++;
    }
    return std::make_pair(quotient, remainder);
}
} // namespace bb::numeric