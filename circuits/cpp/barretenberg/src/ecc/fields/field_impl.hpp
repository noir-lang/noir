#pragma once
#include "../random/engine.hpp"

#ifndef DISABLE_SHENANIGANS
#ifdef __BMI2__
#define BBERG_USE_ASM
#endif
#endif

#ifdef BBERG_USE_ASM
#include "field_impl_x64.hpp"
#else
#include "field_impl_generic.hpp"
#endif

#include <type_traits>

namespace barretenberg {

// template <class T> constexpr void field<T>::butterfly(field& left, field& right) noexcept
// {
// #ifndef BBERG_USE_ASM

// #else
//     if (std::is_constant_evaluated()) {
//     }
//     return asm_butterfly(left, right);
// #endif
// }
/**
 *
 * Mutiplication
 *
 **/
template <class T> constexpr field<T> field<T>::operator*(const field& other) const noexcept
{
#ifndef BBERG_USE_ASM
    return montgomery_mul(other);
#else
    if (std::is_constant_evaluated()) {
        return montgomery_mul(other);
    }
    return asm_mul_with_coarse_reduction(*this, other);
#endif
}

template <class T> constexpr field<T> field<T>::operator*=(const field& other) noexcept
{
#ifndef BBERG_USE_ASM
    *this = operator*(other);
#else
    if (std::is_constant_evaluated()) {
        *this = operator*(other);
    } else {
        asm_self_mul_with_coarse_reduction(*this, other); // asm_self_mul(*this, other);
    }
#endif
    return *this;
}

/**
 *
 * Squaring
 *
 **/
template <class T> constexpr field<T> field<T>::sqr() const noexcept
{
#ifndef BBERG_USE_ASM
    return montgomery_square();
#else
    if (std::is_constant_evaluated()) {
        return montgomery_square();
    } else {
        return asm_sqr_with_coarse_reduction(*this); // asm_sqr(*this);
    }
#endif
}

template <class T> constexpr void field<T>::self_sqr() noexcept
{
#ifndef BBERG_USE_ASM
    *this = montgomery_square();
#else
    if (std::is_constant_evaluated()) {
        *this = montgomery_square();
    } else {
        asm_self_sqr_with_coarse_reduction(*this);
    }
#endif
}

/**
 *
 * Addition
 *
 **/
template <class T> constexpr field<T> field<T>::operator+(const field& other) const noexcept
{
#ifndef BBERG_USE_ASM
    return add(other);
#else
    if (std::is_constant_evaluated()) {
        return add(other);
    } else {
        return asm_add_with_coarse_reduction(*this, other); // asm_add_without_reduction(*this, other);
    }
#endif
}

template <class T> constexpr field<T> field<T>::operator+=(const field& other) noexcept
{
#ifndef BBERG_USE_ASM
    (*this) = operator+(other);
#else
    if (std::is_constant_evaluated()) {
        (*this) = operator+(other);
    } else {
        asm_self_add_with_coarse_reduction(*this, other); // asm_self_add(*this, other);
    }
#endif
    return *this;
}

/**
 *
 * Subtraction
 *
 **/
template <class T> constexpr field<T> field<T>::operator-(const field& other) const noexcept
{
#ifndef BBERG_USE_ASM
    return subtract_coarse(other); // modulus - *this;
#else
    if (std::is_constant_evaluated()) {
        return subtract_coarse(other); // subtract(other);
    } else {
        return asm_sub_with_coarse_reduction(*this, other); // asm_sub(*this, other);
    }
#endif
}

template <class T> constexpr field<T> field<T>::operator-() const noexcept
{
    constexpr field p{ twice_modulus.data[0], twice_modulus.data[1], twice_modulus.data[2], twice_modulus.data[3] };
    return p - *this; // modulus - *this;
}

template <class T> constexpr field<T> field<T>::operator-=(const field& other) noexcept
{
#ifndef BBERG_USE_ASM
    *this = subtract_coarse(other); // subtract(other);
#else
    if (std::is_constant_evaluated()) {
        *this = subtract_coarse(other); // subtract(other);
    } else {
        asm_self_sub_with_coarse_reduction(*this, other); // asm_self_sub(*this, other);
    }
#endif
    return *this;
}

template <class T> constexpr void field<T>::self_neg() noexcept
{
    *this = twice_modulus - *this;
}

template <class T> constexpr void field<T>::self_conditional_negate(const uint64_t predicate) noexcept
{
#ifndef BBERG_USE_ASM
    *this = predicate ? -(*this) : *this;
#else
    if (std::is_constant_evaluated()) {
        *this = predicate ? -(*this) : *this;
    } else {
        asm_conditional_negate(*this, predicate);
    }
#endif
}

/**
 * Comparison operators
 **/
template <class T> constexpr bool field<T>::operator>(const field& other) const noexcept
{
    const field left = reduce_once();
    const field right = other.reduce_once();
    const bool t0 = left.data[3] > right.data[3];
    const bool t1 = (left.data[3] == right.data[3]) && (left.data[2] > right.data[2]);
    const bool t2 =
        (left.data[3] == right.data[3]) && (left.data[2] == right.data[2]) && (left.data[1] > right.data[1]);
    const bool t3 = (left.data[3] == right.data[3]) && (left.data[2] == right.data[2]) &&
                    (left.data[1] == right.data[1]) && (left.data[0] > right.data[0]);
    return (t0 || t1 || t2 || t3);
}

template <class T> constexpr bool field<T>::operator<(const field& other) const noexcept
{
    return (other > *this);
}

template <class T> constexpr bool field<T>::operator==(const field& other) const noexcept
{
    const field left = reduce_once();
    const field right = other.reduce_once();
    return (left.data[0] == right.data[0]) && (left.data[1] == right.data[1]) && (left.data[2] == right.data[2]) &&
           (left.data[3] == right.data[3]);
}

template <class T> constexpr bool field<T>::operator!=(const field& other) const noexcept
{
    return (!operator==(other));
}

template <class T> constexpr field<T> field<T>::to_montgomery_form() const noexcept
{
    constexpr field r_squared{ T::r_squared_0, T::r_squared_1, T::r_squared_2, T::r_squared_3 };

    field result = *this;
    result.reduce_once();
    result.reduce_once();
    result.reduce_once();
    return (result * r_squared).reduce_once();
}

template <class T> constexpr field<T> field<T>::from_montgomery_form() const noexcept
{
    constexpr field one_raw{ 1, 0, 0, 0 };
    return operator*(one_raw).reduce_once();
}

template <class T> constexpr void field<T>::self_to_montgomery_form() noexcept
{
    constexpr field r_squared{ T::r_squared_0, T::r_squared_1, T::r_squared_2, T::r_squared_3 };
    self_reduce_once();
    self_reduce_once();
    self_reduce_once();
    *this *= r_squared;
    self_reduce_once();
}

template <class T> constexpr void field<T>::self_from_montgomery_form() noexcept
{
    constexpr field one_raw{ 1, 0, 0, 0 };
    *this *= one_raw;
    self_reduce_once();
}

template <class T> constexpr field<T> field<T>::reduce_once() const noexcept
{
#ifndef BBERG_USE_ASM
    return reduce();
#else
    if (std::is_constant_evaluated()) {
        return reduce();
    } else {
        return asm_reduce_once(*this);
    }
#endif
}

template <class T> constexpr void field<T>::self_reduce_once() noexcept
{
#ifndef BBERG_USE_ASM
    *this = reduce();
#else
    if (std::is_constant_evaluated()) {
        *this = reduce();
    } else {
        asm_self_reduce_once(*this);
    }
#endif
}

template <class T> constexpr field<T> field<T>::pow(const uint256_t& exponent) const noexcept
{
    if (*this == zero()) {
        return zero();
    }
    if (exponent == uint256_t(0)) {
        return one();
    }
    if (exponent == uint256_t(1)) {
        return *this;
    }

    field accumulator{ data[0], data[1], data[2], data[3] };
    const uint64_t maximum_set_bit = exponent.get_msb();

    for (uint64_t i = maximum_set_bit - 1; i < maximum_set_bit; --i) {
        accumulator.self_sqr();
        if (exponent.get_bit(i)) {
            accumulator *= *this;
        }
    }
    return accumulator;
}

template <class T> constexpr field<T> field<T>::pow(const uint64_t exponent) const noexcept
{
    return pow({ exponent, 0, 0, 0 });
}

template <class T> constexpr field<T> field<T>::invert() const noexcept
{
    if (*this == zero()) {
        return zero();
    }

    const field pow_two = sqr();
    const field pow_three = operator*(pow_two);
    const field pow_four = pow_two.sqr();
    const field pow_five = operator*(pow_four);
    const field pow_six = pow_three.sqr();
    const field pow_seven = operator*(pow_six);
    const field pow_eight = pow_four.sqr();

    const field lookup_table[15]{ *this,
                                  pow_two,
                                  pow_three,
                                  pow_four,
                                  pow_five,
                                  pow_six,
                                  pow_seven,
                                  pow_eight,
                                  operator*(pow_eight),
                                  pow_five.sqr(),
                                  pow_five * pow_six,
                                  pow_six.sqr(),
                                  pow_six * pow_seven,
                                  pow_seven.sqr(),
                                  pow_seven * pow_eight };

    constexpr wnaf_table window = wnaf_table(modulus - uint256_t(2));

    field accumulator = (window.windows[63] > 0) ? lookup_table[window.windows[63] - 1] : one();

    for (size_t i = 62; i < 63; --i) {
        accumulator.self_sqr();
        accumulator.self_sqr();
        accumulator.self_sqr();
        accumulator.self_sqr();
        if (window.windows[i] > 0) {
            accumulator *= lookup_table[window.windows[i] - 1];
        }
    }
    return accumulator;
}

template <class T> void field<T>::batch_invert(field* coeffs, const size_t n) noexcept
{
    field* temporaries = new field[n];
    field accumulator = one();
    for (size_t i = 0; i < n; ++i) {
        temporaries[i] = accumulator;
        accumulator = accumulator * coeffs[i];
    }

    accumulator = accumulator.invert();

    field T0;
    for (size_t i = n - 1; i < n; --i) {
        T0 = accumulator * temporaries[i];
        accumulator = accumulator * coeffs[i];
        coeffs[i] = T0;
    }
    delete[] temporaries;
}

template <class T> constexpr field<T> field<T>::tonelli_shanks_sqrt() const noexcept
{
    // Tonelli-shanks algorithm begins by finding a field element Q and integer S,
    // such that (p - 1) = Q.2^{s}

    // We can compute the square root of a, by considering a^{(Q + 1) / 2} = R
    // Once we have found such an R, we have
    // R^{2} = a^{Q + 1} = a^{Q}a
    // If a^{Q} = 1, we have found our square root.
    // Otherwise, we have a^{Q} = t, where t is a 2^{s-1}'th root of unity.
    // This is because t^{2^{s-1}} = a^{Q.2^{s-1}}.
    // We know that (p - 1) = Q.w^{s}, therefore t^{2^{s-1}} = a^{(p - 1) / 2}
    // From Euler's criterion, if a is a quadratic residue, a^{(p - 1) / 2} = 1
    // i.e. t^{2^{s-1}} = 1

    // To proceed with computing our square root, we want to transform t into a smaller subgroup,
    // specifically, the (s-2)'th roots of unity.
    // We do this by finding some value b,such that
    // (t.b^2)^{2^{s-2}} = 1 and R' = R.b
    // Finding such a b is trivial, because from Euler's criterion, we know that,
    // for any quadratic non-residue z, z^{(p - 1) / 2} = -1
    // i.e. z^{Q.2^{s-1}} = -1
    // => z^Q is a 2^{s-1}'th root of -1
    // => z^{Q^2} is a 2^{s-2}'th root of -1
    // Since t^{2^{s-1}} = 1, we know that t^{2^{s - 2}} = -1
    // => t.z^{Q^2} is a 2^{s - 2}'th root of unity.

    // We can iteratively transform t into ever smaller subgroups, until t = 1.
    // At each iteration, we need to find a new value for b, which we can obtain
    // by repeatedly squaring z^{Q}
    constexpr uint256_t Q = (modulus - 1) >> static_cast<uint64_t>(primitive_root_log_size() - 1);
    constexpr uint256_t Q_minus_one_over_two = (Q - 1) >> 2;

    // __to_montgomery_form(Q_minus_one_over_two, Q_minus_one_over_two);
    field z = coset_generator(0); // the generator is a non-residue
    field b = pow(Q_minus_one_over_two);
    field r = operator*(b); // r = a^{(Q + 1) / 2}
    field t = r * b;        // t = a^{(Q - 1) / 2 + (Q + 1) / 2} = a^{Q}

    // check if t is a square with euler's criterion
    // if not, we don't have a quadratic residue and a has no square root!
    field check = t;
    for (size_t i = 0; i < primitive_root_log_size() - 1; ++i) {
        check.self_sqr();
    }
    if (check != one()) {
        return zero();
    }
    field t1 = z.pow(Q_minus_one_over_two);
    field t2 = t1 * z;
    field c = t2 * t1; // z^Q

    size_t m = primitive_root_log_size();

    while (t != one()) {
        size_t i = 0;
        field t2m = t;

        // find the smallest value of m, such that t^{2^m} = 1
        while (t2m != one()) {
            t2m.self_sqr();
            i += 1;
        }

        size_t j = m - i - 1;
        b = c;
        while (j > 0) {
            b.self_sqr();
            --j;
        } // b = z^2^(m-i-1)

        c = b.sqr();
        t = t * c;
        r = r * b;
        m = i;
    }
    return r;
}

template <class T> constexpr field<T> field<T>::sqrt() const noexcept
{
    if constexpr ((T::modulus_0 & 0x3UL) == 0x3UL) {
        constexpr uint256_t sqrt_exponent = (modulus + uint256_t(1)) >> 2;
        return pow(sqrt_exponent);
    } else {
        return tonelli_shanks_sqrt();
    }
} // namespace barretenberg

template <class T> constexpr field<T> field<T>::operator/(const field& other) const noexcept
{
    return operator*(other.invert());
}

template <class T> constexpr field<T> field<T>::operator/=(const field& other) noexcept
{
    *this = operator/(other);
    return *this;
}

template <class T> constexpr uint64_t field<T>::get_msb() const noexcept
{
    constexpr auto get_uint64_msb = [](const uint64_t in) {
        constexpr uint8_t de_bruijn_sequence[64]{ 0,  47, 1,  56, 48, 27, 2,  60, 57, 49, 41, 37, 28, 16, 3,  61,
                                                  54, 58, 35, 52, 50, 42, 21, 44, 38, 32, 29, 23, 17, 11, 4,  62,
                                                  46, 55, 26, 59, 40, 36, 15, 53, 34, 51, 20, 43, 31, 22, 10, 45,
                                                  25, 39, 14, 33, 19, 30, 9,  24, 13, 18, 8,  12, 7,  6,  5,  63 };

        uint64_t t = in | (in >> 1);
        t |= t >> 2;
        t |= t >> 4;
        t |= t >> 8;
        t |= t >> 16;
        t |= t >> 32;
        return static_cast<uint64_t>(de_bruijn_sequence[(t * 0x03F79D71B4CB0A89ULL) >> 58ULL]);
    };

    uint64_t idx = get_uint64_msb(data[3]);
    idx = idx == 0 ? get_uint64_msb(data[2]) : idx + 64;
    idx = idx == 0 ? get_uint64_msb(data[1]) : idx + 64;
    idx = idx == 0 ? get_uint64_msb(data[0]) : idx + 64;
    return idx;
}

template <class T> constexpr void field<T>::self_set_msb() noexcept
{
    data[3] = 0ULL | (1ULL << 63ULL);
}

template <class T> constexpr bool field<T>::is_msb_set() const noexcept
{
    return (data[3] >> 63ULL) == 1ULL;
}

template <class T> constexpr uint64_t field<T>::is_msb_set_word() const noexcept
{
    return (data[3] >> 63ULL);
}

template <class T> constexpr bool field<T>::get_bit(const uint64_t bit_index) const noexcept
{
    return bool((data[bit_index >> 6] >> (bit_index & 63)) & 1);
}

template <class T> constexpr bool field<T>::is_zero() const noexcept
{
    return ((data[0] | data[1] | data[2] | data[3]) == 0) ||
           (data[0] == T::modulus_0 && data[1] == T::modulus_1 && data[2] == T::modulus_2 && data[3] == T::modulus_3);
}

template <class T> constexpr field<T> field<T>::get_root_of_unity(const size_t subgroup_size) noexcept
{
    field r{ T::primitive_root_0, T::primitive_root_1, T::primitive_root_2, T::primitive_root_3 };
    for (size_t i = primitive_root_log_size(); i > subgroup_size; --i) {
        r.self_sqr();
    }
    return r;
}

template <class T>
field<T> field<T>::random_element(std::mt19937_64* engine, std::uniform_int_distribution<uint64_t>* dist) noexcept
{
    std::mt19937_64* engine_ptr;
    std::uniform_int_distribution<uint64_t>* dist_ptr;
    if (engine == nullptr) {
        engine_ptr = barretenberg::random::get_debug_engine();
        dist_ptr = barretenberg::random::get_distribution();
    } else {
        engine_ptr = engine;
        dist_ptr = dist;
    }
    wide_array random_data{ dist_ptr->operator()(*engine_ptr), dist_ptr->operator()(*engine_ptr),
                            dist_ptr->operator()(*engine_ptr), dist_ptr->operator()(*engine_ptr),
                            dist_ptr->operator()(*engine_ptr), dist_ptr->operator()(*engine_ptr),
                            dist_ptr->operator()(*engine_ptr), dist_ptr->operator()(*engine_ptr) };
    random_data.data[7] = random_data.data[7] & 0b0000111111111111111111111111111111111111111111111111111111111111ULL;
    random_data.data[3] = random_data.data[3] & 0b0000111111111111111111111111111111111111111111111111111111111111ULL;
    field left{ random_data.data[0], random_data.data[1], random_data.data[2], random_data.data[3] };
    field right{ random_data.data[4], random_data.data[5], random_data.data[6], random_data.data[7] };
    left = left.reduce_once().reduce_once();
    right = right.reduce_once().reduce_once();
    field result = (left * right).reduce_once();
    return result;
}

template <class T> constexpr size_t field<T>::primitive_root_log_size() noexcept
{
    uint256_t target = modulus - 1;
    size_t result = 0;
    while (target.get_bit(result) == 0) {
        ++result;
    }
    return result;
}

template <class T>
constexpr std::array<field<T>, field<T>::COSET_GENERATOR_SIZE> field<T>::compute_coset_generators() noexcept
{
    constexpr size_t n = COSET_GENERATOR_SIZE;
    constexpr uint64_t subgroup_size = 1 << 30;

    std::array<field, COSET_GENERATOR_SIZE> result{ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
    if (n > 0) {
        result[0] = (multiplicative_generator());
    }
    field work_variable = multiplicative_generator() + field(1);

    size_t count = 1;
    while (count < n) {
        // work_variable contains a new field element, and we need to test that, for all previous vector elements,
        // result[i] / work_variable is not a member of our subgroup
        field work_inverse = work_variable.invert();
        bool valid = true;
        for (size_t j = 0; j < count; ++j) {
            field subgroup_check = (work_inverse * result[j]).pow(subgroup_size);
            if (subgroup_check == field(1)) {
                valid = false;
                break;
            }
        }
        if (valid) {
            result[count] = (work_variable);
            ++count;
        }
        work_variable += field(1);
    }
    return result;
}

template <class T> constexpr field<T> field<T>::multiplicative_generator() noexcept
{
    field target(1);
    uint256_t p_minus_one_over_two = (modulus - 1) >> 1;
    bool found = false;
    while (!found) {
        target += field(1);
        found = (target.pow(p_minus_one_over_two) == -field(1));
    }
    return target;
}

} // namespace barretenberg