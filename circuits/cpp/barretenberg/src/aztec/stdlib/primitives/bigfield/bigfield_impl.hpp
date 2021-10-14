#pragma once

#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>

#include "../composers/composers.hpp"

#include "../bit_array/bit_array.hpp"
#include "../field/field.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename C, typename T>
bigfield<C, T>::bigfield(C* parent_context)
    : context(parent_context)
    , binary_basis_limbs{ Limb(barretenberg::fr(0)),
                          Limb(barretenberg::fr(0)),
                          Limb(barretenberg::fr(0)),
                          Limb(barretenberg::fr(0)) }
    , prime_basis_limb(context, 0)
{}

template <typename C, typename T>
bigfield<C, T>::bigfield(C* parent_context, const uint256_t& value)
    : context(parent_context)
    , binary_basis_limbs{ Limb(barretenberg::fr(value.slice(0, NUM_LIMB_BITS))),
                          Limb(barretenberg::fr(value.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2))),
                          Limb(barretenberg::fr(value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3))),
                          Limb(barretenberg::fr(value.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4))) }
    , prime_basis_limb(context, value)
{
    ASSERT(value < modulus);
}

template <typename C, typename T>
bigfield<C, T>::bigfield(const field_t<C>& low_bits,
                         const field_t<C>& high_bits,
                         const bool can_overflow,
                         const size_t maximum_bitlength)
{
    context = low_bits.context == nullptr ? high_bits.context : low_bits.context;
    field_t<C> limb_0(context);
    field_t<C> limb_1(context);
    field_t<C> limb_2(context);
    field_t<C> limb_3(context);
    if (low_bits.witness_index != IS_CONSTANT) {
        std::vector<uint32_t> low_accumulator;
        if constexpr (C::type == waffle::PLOOKUP) {
            // Enforce that low_bits indeed only contains 2*NUM_LIMB_BITS bits
            low_accumulator =
                context->decompose_into_default_range(low_bits.witness_index, static_cast<size_t>(NUM_LIMB_BITS * 2));
            // If this doesn't hold we're using a default plookup range size that doesn't work well with the limb size
            // here
            ASSERT(low_accumulator.size() % 2 == 0);
            size_t mid_index = low_accumulator.size() / 2 - 1;
            limb_0.witness_index = low_accumulator[mid_index]; // Q:safer to just slice this from low_bits?
            limb_1 = (low_bits - limb_0) * shift_right_1;
        } else {
            size_t mid_index;
            low_accumulator =
                context->create_range_constraint(low_bits.witness_index, static_cast<size_t>(NUM_LIMB_BITS * 2));
            mid_index = static_cast<size_t>((NUM_LIMB_BITS / 2) - 1);
            // Turbo plonk range constraint returns an array of partial sums, midpoint will happen to hold the big limb
            // value
            limb_1.witness_index = low_accumulator[mid_index];
            // We can get the first half bits of low_bits from the variables we already created
            limb_0 = (low_bits - (limb_1 * shift_1));
        }
    } else {
        uint256_t slice_0 = uint256_t(low_bits.additive_constant).slice(0, NUM_LIMB_BITS);
        uint256_t slice_1 = uint256_t(low_bits.additive_constant).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        limb_0 = field_t(context, barretenberg::fr(slice_0));
        limb_1 = field_t(context, barretenberg::fr(slice_1));
    }

    // If we wish to continue working with this element with lazy reductions - i.e. not moding out again after each
    // addition we apply a more limited range - 2^s for smallest s such that p<2^s (this is the case can_overflow ==
    // false)
    uint64_t num_last_limb_bits = (can_overflow) ? NUM_LIMB_BITS : NUM_LAST_LIMB_BITS;

    // if maximum_bitlength is set, this supercedes can_overflow
    if (maximum_bitlength > 0) {
        ASSERT(maximum_bitlength > 3 * NUM_LIMB_BITS);
        num_last_limb_bits = maximum_bitlength - (3 * NUM_LIMB_BITS);
    }
    // We create the high limb values similar to the low limb ones above
    const uint64_t num_high_limb_bits = NUM_LIMB_BITS + num_last_limb_bits;
    if (high_bits.witness_index != IS_CONSTANT) {

        std::vector<uint32_t> high_accumulator;
        if constexpr (C::type == waffle::PLOOKUP) {
            ASSERT(NUM_LIMB_BITS % 2 == 0); // required for one of the intermediate sums giving the third limb
            high_accumulator = context->decompose_into_default_range(high_bits.witness_index,
                                                                     static_cast<uint32_t>(num_high_limb_bits));
            size_t mid_index = (NUM_LIMB_BITS / waffle::PlookupComposer::DEFAULT_PLOOKUP_RANGE_BITNUM) / 2 - 1;
            limb_2.witness_index = high_accumulator[mid_index];
            limb_3 = (high_bits - limb_2) * shift_right_1;
        } else {
            high_accumulator =
                context->create_range_constraint(high_bits.witness_index, static_cast<size_t>(num_high_limb_bits));
            limb_3.witness_index = high_accumulator[static_cast<size_t>((num_last_limb_bits / 2) - 1)];
            limb_2 = (high_bits - (limb_3 * shift_1));
        }
    } else {
        uint256_t slice_2 = uint256_t(high_bits.additive_constant).slice(0, NUM_LIMB_BITS);
        uint256_t slice_3 = uint256_t(high_bits.additive_constant).slice(NUM_LIMB_BITS, num_high_limb_bits);
        limb_2 = field_t(context, barretenberg::fr(slice_2));
        limb_3 = field_t(context, barretenberg::fr(slice_3));
    }
    binary_basis_limbs[0] = Limb(limb_0, DEFAULT_MAXIMUM_LIMB);
    binary_basis_limbs[1] = Limb(limb_1, DEFAULT_MAXIMUM_LIMB);
    binary_basis_limbs[2] = Limb(limb_2, DEFAULT_MAXIMUM_LIMB);
    binary_basis_limbs[3] = Limb(limb_3, can_overflow ? DEFAULT_MAXIMUM_LIMB : DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
    prime_basis_limb = low_bits + (high_bits * shift_2);
}

template <typename C, typename T>
bigfield<C, T>::bigfield(const bigfield& other)
    : context(other.context)
    , binary_basis_limbs{ other.binary_basis_limbs[0],
                          other.binary_basis_limbs[1],
                          other.binary_basis_limbs[2],
                          other.binary_basis_limbs[3] }
    , prime_basis_limb(other.prime_basis_limb)
{}

template <typename C, typename T>
bigfield<C, T>::bigfield(bigfield&& other)
    : context(other.context)
    , binary_basis_limbs{ other.binary_basis_limbs[0],
                          other.binary_basis_limbs[1],
                          other.binary_basis_limbs[2],
                          other.binary_basis_limbs[3] }
    , prime_basis_limb(other.prime_basis_limb)
{}

template <typename C, typename T> bigfield<C, T>::bigfield(const byte_array<C>& bytes)
{
    const bit_array<C> bytes_as_bits(bytes);
    const auto rbits = bytes_as_bits.get_bits();
    std::vector<bool_t<C>> bits(rbits.rbegin(), rbits.rend());
    context = bytes.get_context();
    const size_t num_bits = bits.size();
    const size_t offset = num_bits - modulus_u512.get_msb() - 1;
    std::vector<field_t<C>> elements;
    for (size_t i = 0; i < 4; ++i) {
        field_t<C> element = field_t<C>(context, barretenberg::fr(0));
        size_t start;
        if (i == 0) {
            start = 0;
        }
        if (i == 1) {
            start = NUM_LAST_LIMB_BITS;
        }
        if (i == 2) {
            start = NUM_LAST_LIMB_BITS + NUM_LIMB_BITS;
        }
        if (i == 3) {
            start = NUM_LAST_LIMB_BITS + NUM_LIMB_BITS * 2;
        }
        const size_t end = start + ((i == 0) ? NUM_LAST_LIMB_BITS : NUM_LIMB_BITS);
        for (size_t j = start; j < end; ++j) {
            element = element + element;
            element = element + field_t<C>(bits[j + offset]);
        }
        elements.push_back(element);
    }
    binary_basis_limbs[3].element = elements[0];
    binary_basis_limbs[3].maximum_value = DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB;
    binary_basis_limbs[2].element = elements[1];
    binary_basis_limbs[2].maximum_value = DEFAULT_MAXIMUM_LIMB;
    binary_basis_limbs[1].element = elements[2];
    binary_basis_limbs[1].maximum_value = DEFAULT_MAXIMUM_LIMB;
    binary_basis_limbs[0].element = elements[3];
    binary_basis_limbs[0].maximum_value = DEFAULT_MAXIMUM_LIMB;
    prime_basis_limb = (binary_basis_limbs[3].element * shift_3) + (binary_basis_limbs[2].element * shift_2) +
                       (binary_basis_limbs[1].element * shift_1) + (binary_basis_limbs[0].element);
}

template <typename C, typename T> bigfield<C, T>& bigfield<C, T>::operator=(const bigfield& other)
{
    context = other.context;
    binary_basis_limbs[0] = other.binary_basis_limbs[0];
    binary_basis_limbs[1] = other.binary_basis_limbs[1];
    binary_basis_limbs[2] = other.binary_basis_limbs[2];
    binary_basis_limbs[3] = other.binary_basis_limbs[3];
    prime_basis_limb = other.prime_basis_limb;
    return *this;
}

template <typename C, typename T> bigfield<C, T>& bigfield<C, T>::operator=(bigfield&& other)
{
    context = other.context;
    binary_basis_limbs[0] = other.binary_basis_limbs[0];
    binary_basis_limbs[1] = other.binary_basis_limbs[1];
    binary_basis_limbs[2] = other.binary_basis_limbs[2];
    binary_basis_limbs[3] = other.binary_basis_limbs[3];
    prime_basis_limb = other.prime_basis_limb;
    return *this;
}

template <typename C, typename T> uint512_t bigfield<C, T>::get_value() const
{
    uint512_t t0 = uint256_t(binary_basis_limbs[0].element.get_value());
    uint512_t t1 = uint256_t(binary_basis_limbs[1].element.get_value());
    uint512_t t2 = uint256_t(binary_basis_limbs[2].element.get_value());
    uint512_t t3 = uint256_t(binary_basis_limbs[3].element.get_value());
    return t0 + (t1 << (NUM_LIMB_BITS)) + (t2 << (2 * NUM_LIMB_BITS)) + (t3 << (3 * NUM_LIMB_BITS));
}

template <typename C, typename T> uint512_t bigfield<C, T>::get_maximum_value() const
{
    uint512_t t0 = uint512_t(binary_basis_limbs[0].maximum_value);
    uint512_t t1 = uint512_t(binary_basis_limbs[1].maximum_value) << NUM_LIMB_BITS;
    uint512_t t2 = uint512_t(binary_basis_limbs[2].maximum_value) << (NUM_LIMB_BITS * 2);
    uint512_t t3 = uint512_t(binary_basis_limbs[3].maximum_value) << (NUM_LIMB_BITS * 3);
    return t0 + t1 + t2 + t3;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator+(const bigfield& other) const
{
    reduction_check();
    other.reduction_check();
    // needed cause a constant doesn't have a valid context
    C* ctx = context ? context : other.context;

    if (is_constant() && other.is_constant()) {
        return bigfield(ctx, uint256_t((get_value() + other.get_value()) % modulus_u512));
    }
    bigfield result(ctx);
    result.binary_basis_limbs[0].element = binary_basis_limbs[0].element + other.binary_basis_limbs[0].element;
    result.binary_basis_limbs[1].element = binary_basis_limbs[1].element + other.binary_basis_limbs[1].element;
    result.binary_basis_limbs[2].element = binary_basis_limbs[2].element + other.binary_basis_limbs[2].element;
    result.binary_basis_limbs[3].element = binary_basis_limbs[3].element + other.binary_basis_limbs[3].element;
    result.binary_basis_limbs[0].maximum_value =
        binary_basis_limbs[0].maximum_value + other.binary_basis_limbs[0].maximum_value;
    result.binary_basis_limbs[1].maximum_value =
        binary_basis_limbs[1].maximum_value + other.binary_basis_limbs[1].maximum_value;
    result.binary_basis_limbs[2].maximum_value =
        binary_basis_limbs[2].maximum_value + other.binary_basis_limbs[2].maximum_value;
    result.binary_basis_limbs[3].maximum_value =
        binary_basis_limbs[3].maximum_value + other.binary_basis_limbs[3].maximum_value;
    result.prime_basis_limb = prime_basis_limb + other.prime_basis_limb;
    return result;
}

// to make sure we don't go to negative values, add p before subtracting other
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator-(const bigfield& other) const
{
    C* ctx = context ? context : other.context;
    reduction_check();
    other.reduction_check();

    if (is_constant() && other.is_constant()) {
        uint512_t left = get_value() % modulus_u512;
        uint512_t right = other.get_value() % modulus_u512;
        uint512_t out = (left + modulus_u512 - right) % modulus_u512;
        return bigfield(ctx, uint256_t(out.lo));
    }

    if (other.is_constant()) {
        uint512_t right = other.get_value() % modulus_u512;
        uint512_t neg_right = modulus_u512 - right;
        return operator+(bigfield(ctx, uint256_t(neg_right.lo)));
    }

    bigfield result(ctx);

    uint256_t limb_0_maximum_value = other.binary_basis_limbs[0].maximum_value;
    uint64_t limb_0_borrow_shift = std::max(limb_0_maximum_value.get_msb() + 1, NUM_LIMB_BITS);
    uint256_t limb_1_maximum_value =
        other.binary_basis_limbs[1].maximum_value + (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS));
    uint64_t limb_1_borrow_shift = std::max(limb_1_maximum_value.get_msb() + 1, NUM_LIMB_BITS);
    uint256_t limb_2_maximum_value =
        other.binary_basis_limbs[2].maximum_value + (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS));
    uint64_t limb_2_borrow_shift = std::max(limb_2_maximum_value.get_msb() + 1, NUM_LIMB_BITS);

    uint256_t limb_3_maximum_value =
        other.binary_basis_limbs[3].maximum_value + (uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    uint512_t constant_to_add = modulus_u512;
    // add a large enough multiple of p to not get negative result in subtraction
    while (constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo <= limb_3_maximum_value) {
        constant_to_add += modulus_u512;
    }

    uint256_t t0(uint256_t(1) << limb_0_borrow_shift);
    uint256_t t1((uint256_t(1) << limb_1_borrow_shift) - (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t2((uint256_t(1) << limb_2_borrow_shift) - (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t3(uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    uint256_t to_add_0 = uint256_t(constant_to_add.slice(0, NUM_LIMB_BITS));
    uint256_t to_add_1 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2));
    uint256_t to_add_2 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3));
    uint256_t to_add_3 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4));

    result.binary_basis_limbs[0].element =
        binary_basis_limbs[0].element + barretenberg::fr(t0 + to_add_0) - other.binary_basis_limbs[0].element;
    result.binary_basis_limbs[1].element =
        binary_basis_limbs[1].element + barretenberg::fr(t1 + to_add_1) - other.binary_basis_limbs[1].element;
    result.binary_basis_limbs[2].element =
        binary_basis_limbs[2].element + barretenberg::fr(t2 + to_add_2) - other.binary_basis_limbs[2].element;
    result.binary_basis_limbs[3].element =
        binary_basis_limbs[3].element + barretenberg::fr(to_add_3 - t3) - other.binary_basis_limbs[3].element;

    result.binary_basis_limbs[0].maximum_value = binary_basis_limbs[0].maximum_value + t0 + to_add_0;
    result.binary_basis_limbs[1].maximum_value = binary_basis_limbs[1].maximum_value + t1 + to_add_1;
    result.binary_basis_limbs[2].maximum_value = binary_basis_limbs[2].maximum_value + t2 + to_add_2;
    result.binary_basis_limbs[3].maximum_value = binary_basis_limbs[3].maximum_value - t3 + to_add_3;

    uint512_t constant_to_add_mod_p = (constant_to_add) % prime_basis.modulus;
    field_t prime_basis_to_add(ctx, barretenberg::fr(constant_to_add_mod_p.lo));
    result.prime_basis_limb = prime_basis_limb + prime_basis_to_add - other.prime_basis_limb;
    return result;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator*(const bigfield& other) const
{
    reduction_check();
    other.reduction_check();
    if (mul_product_crt_check(get_maximum_value(), other.get_maximum_value(), {})) {
        if (get_maximum_value() > other.get_maximum_value()) {
            self_reduce();
        } else {
            other.self_reduce();
        }
        return (*this).operator*(other);
    }
    C* ctx = context ? context : other.context;

    const auto [quotient_value, remainder_value] = compute_quotient_remainder_values(*this, other, {});
    bigfield remainder;
    bigfield quotient;
    if (is_constant() && other.is_constant()) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        // when writing a*b = q*p + r we wish to enforce r<2^s for smallest s such that p<2^s
        // hence the second constructor call is with can_overflow=false. This will allow using r in more additions mod
        // 2^t without needing to apply the mod, where t=4*NUM_LIMB_BITS
        const size_t num_quotient_bits = get_quotient_max_bits({ DEFAULT_MAXIMUM_REMAINDER });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        remainder = bigfield(
            witness_t(ctx, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    };

    evaluate_multiply_add(*this, other, {}, quotient, { remainder });
    return remainder;
}

// TODO: reduce code duplication
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator/(const bigfield& other) const
{
    reduction_check();
    other.reduction_check();
    if (mul_product_crt_check(DEFAULT_MAXIMUM_REMAINDER, other.get_maximum_value(), {})) {
        self_reduce();
        other.self_reduce();
    }
    // TODO: method for returning correct context and checking there aren't multiple
    C* ctx = context ? context : other.context;
    // a / b = c
    // => c * b = a mod p
    const uint1024_t left = uint1024_t(get_value());
    const uint1024_t right = uint1024_t(other.get_value());
    const uint1024_t modulus(target_basis.modulus);
    uint512_t inverse_value = right.lo.invmod(target_basis.modulus).lo;
    uint1024_t inverse_1024(inverse_value);
    inverse_value = ((left * inverse_1024) % modulus).lo;

    const uint1024_t quotient_1024 = (uint1024_t(inverse_value) * right - left) / modulus;
    const uint512_t quotient_value = quotient_1024.lo;

    bigfield inverse;
    bigfield quotient;
    if (is_constant() && other.is_constant()) {
        inverse = bigfield(ctx, uint256_t(inverse_value));
        return inverse;
    } else {
        const size_t num_quotient_bits = get_quotient_max_bits({ get_maximum_value() });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        inverse = bigfield(
            witness_t(ctx, fr(inverse_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(inverse_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    }

    evaluate_multiply_add(other, inverse, {}, quotient, { *this });
    return inverse;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::sqr() const
{
    reduction_check();
    if (mul_product_crt_check(get_maximum_value(), get_maximum_value(), {})) {
        self_reduce();
    }
    C* ctx = context;

    const auto [quotient_value, remainder_value] = compute_quotient_remainder_values(*this, *this, {});

    bigfield remainder;
    bigfield quotient;
    if (is_constant()) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        const size_t num_quotient_bits = get_quotient_max_bits({ DEFAULT_MAXIMUM_REMAINDER });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        remainder = bigfield(
            witness_t(ctx, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    };

    evaluate_square_add(*this, {}, quotient, remainder);
    return remainder;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::sqradd(const std::vector<bigfield>& to_add) const
{
    reduction_check();
    if (mul_product_crt_check(get_maximum_value(), get_maximum_value(), to_add)) {
        self_reduce();
        ASSERT(mul_product_crt_check(get_maximum_value(), get_maximum_value(), to_add) == 0);
    }
    C* ctx = context;

    uint512_t add_values(0);
    bool add_constant = true;
    for (const auto& add_element : to_add) {
        add_element.reduction_check();
        add_values += add_element.get_value();
        add_constant = add_constant && (add_element.is_constant());
    }

    const uint1024_t left(get_value());
    const uint1024_t right(get_value());
    const uint1024_t add_right(add_values);
    const uint1024_t modulus(target_basis.modulus);

    const auto [quotient_1024, remainder_1024] = (left * right + add_right).divmod(modulus);

    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    if (is_constant() && add_constant) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        const size_t num_quotient_bits = get_quotient_max_bits({ DEFAULT_MAXIMUM_REMAINDER });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        remainder = bigfield(
            witness_t(ctx, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    };
    evaluate_square_add(*this, to_add, quotient, remainder);
    return remainder;
}

template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::madd(const bigfield& to_mul, const std::vector<bigfield>& to_add) const
{
    reduction_check();
    to_mul.reduction_check();

    if (mul_product_crt_check(get_maximum_value(), to_mul.get_maximum_value(), to_add)) {
        if (get_maximum_value() > to_mul.get_maximum_value()) {
            self_reduce();
        } else {
            to_mul.self_reduce();
        }
        return (*this).madd(to_mul, to_add);
    }
    uint512_t add_values(0);
    bool add_constant = true;
    for (const auto& add_element : to_add) {
        add_element.reduction_check();
        add_values += add_element.get_value();
        add_constant = add_constant && (add_element.is_constant());
    }

    C* ctx = context ? context : to_mul.context;

    const uint1024_t left(get_value());
    const uint1024_t mul_right(to_mul.get_value());
    const uint1024_t add_right(add_values);
    const uint1024_t modulus(target_basis.modulus);

    const auto [quotient_1024, remainder_1024] = (left * mul_right + add_right).divmod(modulus);

    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    if (is_constant() && to_mul.is_constant() && add_constant) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        const size_t num_quotient_bits = get_quotient_max_bits({ DEFAULT_MAXIMUM_REMAINDER });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        remainder = bigfield(
            witness_t(ctx, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    };
    evaluate_multiply_add(*this, to_mul, to_add, quotient, { remainder });
    return remainder;
}

template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::div(const std::vector<bigfield>& numerators, const bigfield& denominator)
{
    if (numerators.size() == 0) {
        return bigfield<C, T>(nullptr, uint256_t(0));
    }

    denominator.reduction_check();
    if (mul_product_crt_check(DEFAULT_MAXIMUM_REMAINDER, denominator.get_maximum_value(), {})) {
        denominator.self_reduce();
    }
    C* ctx = denominator.context;
    uint512_t numerator_values(0);
    bool numerator_constant = true;
    for (const auto& numerator_element : numerators) {
        ctx = (ctx == nullptr) ? numerator_element.get_context() : ctx;
        numerator_element.reduction_check();
        numerator_values += numerator_element.get_value();
        numerator_constant = numerator_constant && (numerator_element.is_constant());
    }

    // a / b = c
    // => c * b = a mod p
    const uint1024_t left = uint1024_t(numerator_values);
    const uint1024_t right = uint1024_t(denominator.get_value());
    const uint1024_t modulus(target_basis.modulus);
    uint512_t inverse_value = right.lo.invmod(target_basis.modulus).lo;
    uint1024_t inverse_1024(inverse_value);
    inverse_value = ((left * inverse_1024) % modulus).lo;

    const uint1024_t quotient_1024 = (uint1024_t(inverse_value) * right - left) / modulus;
    const uint512_t quotient_value = quotient_1024.lo;

    bigfield inverse;
    bigfield quotient;
    if (numerator_constant && denominator.is_constant()) {
        inverse = bigfield(ctx, uint256_t(inverse_value));
        return inverse;
    } else {
        std::vector<uint1024_t> numerator_max;
        for (const auto& n : numerators) {
            numerator_max.push_back(n.get_maximum_value());
        }
        const size_t num_quotient_bits = get_quotient_max_bits(numerator_max);
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        inverse = bigfield(
            witness_t(ctx, fr(inverse_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(inverse_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    }

    evaluate_multiply_add(denominator, inverse, {}, quotient, numerators);
    return inverse;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::conditional_negate(const bool_t<C>& predicate) const
{
    C* ctx = context ? context : predicate.context;

    if (is_constant() && predicate.is_constant()) {
        if (predicate.get_value()) {
            uint512_t out_val = (modulus_u512 - get_value()) % modulus_u512;
            return bigfield(ctx, out_val.lo);
        }
        return *this;
    }
    reduction_check();

    uint256_t limb_0_maximum_value = binary_basis_limbs[0].maximum_value;
    uint64_t limb_0_borrow_shift = std::max(limb_0_maximum_value.get_msb() + 1, NUM_LIMB_BITS);
    uint256_t limb_1_maximum_value =
        binary_basis_limbs[1].maximum_value + (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS));
    uint64_t limb_1_borrow_shift = std::max(limb_1_maximum_value.get_msb() + 1, NUM_LIMB_BITS);
    uint256_t limb_2_maximum_value =
        binary_basis_limbs[2].maximum_value + (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS));
    uint64_t limb_2_borrow_shift = std::max(limb_2_maximum_value.get_msb() + 1, NUM_LIMB_BITS);

    uint256_t limb_3_maximum_value =
        binary_basis_limbs[3].maximum_value + (uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    // uint256_t comparison_maximum = uint256_t(modulus_u512.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4));
    // uint256_t additive_term = comparison_maximum;
    uint512_t constant_to_add = modulus_u512;
    while (constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo <= limb_3_maximum_value) {
        constant_to_add += modulus_u512;
    }

    uint256_t t0(uint256_t(1) << limb_0_borrow_shift);
    uint256_t t1((uint256_t(1) << limb_1_borrow_shift) - (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t2((uint256_t(1) << limb_2_borrow_shift) - (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t3(uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    uint256_t to_add_0_u256 = uint256_t(constant_to_add.slice(0, NUM_LIMB_BITS));
    uint256_t to_add_1_u256 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2));
    uint256_t to_add_2_u256 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3));
    uint256_t to_add_3_u256 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4));

    barretenberg::fr to_add_0(t0 + to_add_0_u256);
    barretenberg::fr to_add_1(t1 + to_add_1_u256);
    barretenberg::fr to_add_2(t2 + to_add_2_u256);
    barretenberg::fr to_add_3(to_add_3_u256 - t3);

    // we either return current value if predicate is false, or (limb_i - value) if predicate is true
    // (1 - predicate) * value + predicate * (limb_i - value)
    // = predicate * (limb_i - 2 * value) + value
    barretenberg::fr two(2);

    field_t limb_0 = static_cast<field_t<C>>(predicate).madd(-(binary_basis_limbs[0].element * two) + to_add_0,
                                                             binary_basis_limbs[0].element);
    field_t limb_1 = static_cast<field_t<C>>(predicate).madd(-(binary_basis_limbs[1].element * two) + to_add_1,
                                                             binary_basis_limbs[1].element);
    field_t limb_2 = static_cast<field_t<C>>(predicate).madd(-(binary_basis_limbs[2].element * two) + to_add_2,
                                                             binary_basis_limbs[2].element);
    field_t limb_3 = static_cast<field_t<C>>(predicate).madd(-(binary_basis_limbs[3].element * two) + to_add_3,
                                                             binary_basis_limbs[3].element);

    uint256_t max_limb_0 = binary_basis_limbs[0].maximum_value + to_add_0_u256 + t0;
    uint256_t max_limb_1 = binary_basis_limbs[1].maximum_value + to_add_1_u256 + t1;
    uint256_t max_limb_2 = binary_basis_limbs[2].maximum_value + to_add_2_u256 + t2;
    uint256_t max_limb_3 = binary_basis_limbs[3].maximum_value + to_add_3_u256 - t3;

    bigfield result(ctx);
    result.binary_basis_limbs[0] = Limb(limb_0, max_limb_0);
    result.binary_basis_limbs[1] = Limb(limb_1, max_limb_1);
    result.binary_basis_limbs[2] = Limb(limb_2, max_limb_2);
    result.binary_basis_limbs[3] = Limb(limb_3, max_limb_3);

    uint512_t constant_to_add_mod_p = constant_to_add % prime_basis.modulus;
    field_t prime_basis_to_add(ctx, barretenberg::fr(constant_to_add_mod_p.lo));
    result.prime_basis_limb =
        static_cast<field_t<C>>(predicate).madd(-(prime_basis_limb * two) + prime_basis_to_add, prime_basis_limb);

    return result;
}

template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::conditional_select(const bigfield& other, const bool_t<C>& predicate) const
{
    if (is_constant() && other.is_constant() && predicate.is_constant()) {
        if (predicate.get_value()) {
            return *this;
        }
        return other;
    }
    reduction_check();
    C* ctx = context ? context : (other.context ? other.context : predicate.context);

    field_t binary_limb_0 = static_cast<field_t<C>>(predicate).madd(
        other.binary_basis_limbs[0].element - binary_basis_limbs[0].element, binary_basis_limbs[0].element);
    field_t binary_limb_1 = static_cast<field_t<C>>(predicate).madd(
        other.binary_basis_limbs[1].element - binary_basis_limbs[1].element, binary_basis_limbs[1].element);
    field_t binary_limb_2 = static_cast<field_t<C>>(predicate).madd(
        other.binary_basis_limbs[2].element - binary_basis_limbs[2].element, binary_basis_limbs[2].element);
    field_t binary_limb_3 = static_cast<field_t<C>>(predicate).madd(
        other.binary_basis_limbs[3].element - binary_basis_limbs[3].element, binary_basis_limbs[3].element);
    field_t prime_limb =
        static_cast<field_t<C>>(predicate).madd(other.prime_basis_limb - prime_basis_limb, prime_basis_limb);

    bigfield result(ctx);
    // the maximum of the maximal values of elements is large enough
    result.binary_basis_limbs[0] =
        Limb(binary_limb_0, std::max(binary_basis_limbs[0].maximum_value, other.binary_basis_limbs[0].maximum_value));
    result.binary_basis_limbs[1] =
        Limb(binary_limb_1, std::max(binary_basis_limbs[1].maximum_value, other.binary_basis_limbs[1].maximum_value));
    result.binary_basis_limbs[2] =
        Limb(binary_limb_2, std::max(binary_basis_limbs[2].maximum_value, other.binary_basis_limbs[2].maximum_value));
    result.binary_basis_limbs[3] =
        Limb(binary_limb_3, std::max(binary_basis_limbs[3].maximum_value, other.binary_basis_limbs[3].maximum_value));
    result.prime_basis_limb = prime_limb;
    return result;
}

// reduce values to s bits if limbs have gotten to big (e.g. after addition chain)
template <typename C, typename T> void bigfield<C, T>::reduction_check() const
{
    if (is_constant()) { // this seems not a reduction check, but actually computing the reduction
                         // TODO THIS IS UGLY WHY CAN'T WE JUST DO (*THIS) = REDUCED?
        uint256_t reduced_value = (get_value() % modulus_u512).lo;
        bigfield reduced(context, uint256_t(reduced_value));
        binary_basis_limbs[0] = reduced.binary_basis_limbs[0];
        binary_basis_limbs[1] = reduced.binary_basis_limbs[1];
        binary_basis_limbs[2] = reduced.binary_basis_limbs[2];
        binary_basis_limbs[3] = reduced.binary_basis_limbs[3];
        prime_basis_limb = reduced.prime_basis_limb;
        return;
    }
    uint256_t maximum_limb_value = get_maximum_unreduced_limb_value();
    bool limb_overflow_test_0 = binary_basis_limbs[0].maximum_value > maximum_limb_value;
    bool limb_overflow_test_1 = binary_basis_limbs[1].maximum_value > maximum_limb_value;
    bool limb_overflow_test_2 = binary_basis_limbs[2].maximum_value > maximum_limb_value;
    bool limb_overflow_test_3 = binary_basis_limbs[3].maximum_value > maximum_limb_value;
    if (get_maximum_value() > get_maximum_unreduced_value() || limb_overflow_test_0 || limb_overflow_test_1 ||
        limb_overflow_test_2 || limb_overflow_test_3) {
        self_reduce();
    }
}

// create a version with mod 2^t element part in [0,p-1]
// should we call this assert if it actually creates new vars satisfying assert?
// After reducing to size 2^s, we check p-a is non-negative as integer.
// We perform subtraction using carries on blocks of size 2^b. The operations insde the blocks are done mod r
// Including the effect of carries the operation inside each limb is in the range [-2^b-1,2^{b+1}]
// Assuming this values are all distinct mod r, which happens e.g. if r/2>2^{b+1}, then if all limb values are
// non-negative at the end of subtraction, we know the subtraction result is positive as integers and a<p
template <typename C, typename T> void bigfield<C, T>::assert_is_in_field() const
{
    if (is_constant()) {
        return;
    }

    self_reduce(); // this method in particular enforces limb vals are <2^b - needed for logic described above
    uint256_t value = get_value().lo;
    // TODO:make formal assert that modulus<=256 bits
    constexpr uint256_t modulus_minus_one = modulus_u512.lo - 1;

    constexpr uint256_t modulus_0_value = modulus_minus_one.slice(0, NUM_LIMB_BITS);
    constexpr uint256_t modulus_1_value = modulus_minus_one.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2);
    constexpr uint256_t modulus_2_value = modulus_minus_one.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3);
    constexpr uint256_t modulus_3_value = modulus_minus_one.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4);

    bool borrow_0_value = value.slice(0, NUM_LIMB_BITS) > modulus_0_value;
    bool borrow_1_value = (value.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2) - uint256_t(borrow_0_value)) > modulus_1_value;
    bool borrow_2_value =
        (value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3) - uint256_t(borrow_1_value)) > modulus_2_value;

    field_t<C> modulus_0(context, modulus_0_value);
    field_t<C> modulus_1(context, modulus_1_value);
    field_t<C> modulus_2(context, modulus_2_value);
    field_t<C> modulus_3(context, modulus_3_value);
    // where are you constraining the borrows to be correct? (maybe not needed)
    bool_t<C> borrow_0(witness_t<C>(context, borrow_0_value));
    bool_t<C> borrow_1(witness_t<C>(context, borrow_1_value));
    bool_t<C> borrow_2(witness_t<C>(context, borrow_2_value));

    field_t<C> r0 = modulus_0 - binary_basis_limbs[0].element + static_cast<field_t<C>>(borrow_0) * shift_1;
    field_t<C> r1 = modulus_1 - binary_basis_limbs[1].element + static_cast<field_t<C>>(borrow_1) * shift_1 -
                    static_cast<field_t<C>>(borrow_0);
    field_t<C> r2 = modulus_2 - binary_basis_limbs[2].element + static_cast<field_t<C>>(borrow_2) * shift_1 -
                    static_cast<field_t<C>>(borrow_1);
    field_t<C> r3 = modulus_3 - binary_basis_limbs[3].element - static_cast<field_t<C>>(borrow_2);
    r0 = r0.normalize();
    r1 = r1.normalize();
    r2 = r2.normalize();
    r3 = r3.normalize();
    if constexpr (C::type == waffle::PLOOKUP) {
        context->decompose_into_default_range(r0.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r1.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r2.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r3.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
    } else {
        context->create_range_constraint(r0.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->create_range_constraint(r1.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->create_range_constraint(r2.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->create_range_constraint(r3.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
    }
}

// check elements are equal mod p by proving their integer difference is a multiple of p.
// This relies on the minus operator for a-b increasing a by a multiple of p large enough so diff is non-negative
template <typename C, typename T> void bigfield<C, T>::assert_equal(const bigfield& other) const
{
    C* ctx = this->context ? this->context : other.context;

    bigfield diff = *this - other;
    const uint512_t diff_val = diff.get_value();
    const uint512_t modulus(target_basis.modulus);

    const auto [quotient_512, remainder_512] = (diff_val).divmod(modulus);
    if (remainder_512 != 0)
        std::cout << "remainder not zero!" << std::endl;
    ASSERT(remainder_512 == 0);
    bigfield quotient;

    const size_t num_quotient_bits = get_quotient_max_bits({ 0 });
    quotient = bigfield(witness_t(ctx, fr(quotient_512.slice(0, NUM_LIMB_BITS * 2).lo)),
                        witness_t(ctx, fr(quotient_512.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                        false,
                        num_quotient_bits);
    evaluate_multiply_add(diff, { one() }, {}, quotient, { zero() });
}

// construct a proof that points are different mod p, when they are different mod r
// WARNING: This method doesn't have perfect completeness - for points equal mod r (or with certain difference kp mod r)
// but different mod p, you can't construct a proof.
// Note also that the number of constraints depends on how much the values have overflown beyond p
// e.g. due to an addition chain
// The function is based on the following. Suppose a-b = 0 mod p. Then a-b = k*p for k in a range [-R,L]
// such that L*p>= a, R*p>=b. And also a-b = k*p mod r for such k. Thus we can verify a-b is non-zero mod p
// by taking the product of such values (a-b-kp) and showing it's non-zero mod r
template <typename C, typename T> void bigfield<C, T>::assert_is_not_equal(const bigfield& other) const
{
    const auto get_overload_count = [target_modulus = modulus_u512](const uint512_t& maximum_value) {
        uint512_t target = target_modulus;
        size_t overload_count = 0;
        while (target < maximum_value) {
            ++overload_count;
            target += target_modulus;
        }
        return overload_count;
    };
    const size_t lhs_overload_count = get_overload_count(get_maximum_value());
    const size_t rhs_overload_count = get_overload_count(other.get_maximum_value());

    // if (a == b) then (a == b mod n)
    // to save gates, we only check that (a == b mod n)

    // if numeric val of a = a' + p.q
    // we want to check (a' + p.q == b mod n)
    const field_t<C> base_diff = prime_basis_limb - other.prime_basis_limb;
    auto diff = base_diff;
    field_t<C> prime_basis(get_context(), modulus);
    field_t<C> prime_basis_accumulator = prime_basis;
    for (size_t i = 0; i < lhs_overload_count; ++i) {
        diff = diff * (base_diff - prime_basis_accumulator);
        prime_basis_accumulator += prime_basis;
    }
    prime_basis_accumulator = prime_basis;
    for (size_t i = 0; i < rhs_overload_count; ++i) {
        diff = diff * (base_diff + prime_basis_accumulator);
        prime_basis_accumulator += prime_basis;
    }
    diff.assert_is_not_zero();
}

// We reduce an element's mod 2^t representation (t=4*NUM_LIMB_BITS) to size 2^s for smallest s with 2^s>p
// This is much cheaper than actually reducing mod p and suffices for addition chains (where we just need not to
// overflow 2^t) We also reduce any "spillage" inside the first 3 limbs, so that their range is NUM_LIMB_BITS and not
// larger
template <typename C, typename T> void bigfield<C, T>::self_reduce() const
{
    if (is_constant()) {
        return;
    }
    const auto [quotient_value, remainder_value] = get_value().divmod(target_basis.modulus);

    bigfield quotient(context);

    uint512_t maximum_quotient_size = get_maximum_value() / target_basis.modulus;
    uint64_t maximum_quotient_bits = maximum_quotient_size.get_msb() + 1;
    if ((maximum_quotient_bits & 1ULL) == 1ULL) {
        ++maximum_quotient_bits;
    }
    // TODO: implicit assumption here - NUM_LIMB_BITS large enough for all the quotient
    uint32_t quotient_limb_index = context->add_variable(barretenberg::fr(quotient_value.lo));
    field_t<C> quotient_limb = field_t<C>::from_witness_index(context, quotient_limb_index);
    if constexpr (C::type == waffle::PLOOKUP) {
        context->decompose_into_default_range(quotient_limb.witness_index, static_cast<size_t>(maximum_quotient_bits));
    } else {
        context->create_range_constraint(quotient_limb.witness_index, static_cast<size_t>(maximum_quotient_bits));
    }

    ASSERT((uint1024_t(1) << maximum_quotient_bits) * uint1024_t(modulus_u512) + DEFAULT_MAXIMUM_REMAINDER <
           get_maximum_crt_product());
    quotient.binary_basis_limbs[0] = Limb(quotient_limb, uint256_t(1) << maximum_quotient_bits);
    quotient.binary_basis_limbs[1] = Limb(field_t<C>(context, barretenberg::fr(0)), 0);
    quotient.binary_basis_limbs[2] = Limb(field_t<C>(context, barretenberg::fr(0)), 0);
    quotient.binary_basis_limbs[3] = Limb(field_t<C>(context, barretenberg::fr(0)), 0);
    quotient.prime_basis_limb = quotient_limb;
    // this constructor with can_overflow=false will create remainder of size<2^s
    bigfield remainder = bigfield(
        witness_t(context, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
        witness_t(context, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));

    evaluate_multiply_add(*this, one(), {}, quotient, { remainder });
    uint512_t test = get_value() % prime_basis.modulus;
    barretenberg::fr binary_basis_mod_prime_basis(test.lo);  // variable not used
    binary_basis_limbs[0] = remainder.binary_basis_limbs[0]; // how is this method const?
    binary_basis_limbs[1] = remainder.binary_basis_limbs[1];
    binary_basis_limbs[2] = remainder.binary_basis_limbs[2];
    binary_basis_limbs[3] = remainder.binary_basis_limbs[3];
    prime_basis_limb = remainder.prime_basis_limb;
}

// See explanation at https://hackmd.io/LoEG5nRHQe-PvstVaD51Yw?view
template <typename C, typename T>
void bigfield<C, T>::evaluate_multiply_add(const bigfield& left,
                                           const bigfield& to_mul,
                                           const std::vector<bigfield>& to_add,
                                           const bigfield& quotient,
                                           const std::vector<bigfield>& remainders)
{
    C* ctx = left.context ? left.context : to_mul.context;

    uint256_t max_b0 = (left.binary_basis_limbs[1].maximum_value * to_mul.binary_basis_limbs[0].maximum_value);
    max_b0 += (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_b1 = (left.binary_basis_limbs[0].maximum_value * to_mul.binary_basis_limbs[1].maximum_value);
    max_b1 += (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_c0 = (left.binary_basis_limbs[1].maximum_value * to_mul.binary_basis_limbs[1].maximum_value);
    max_c0 += (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_c1 = (left.binary_basis_limbs[2].maximum_value * to_mul.binary_basis_limbs[0].maximum_value);
    max_c1 += (neg_modulus_limbs_u256[2] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_c2 = (left.binary_basis_limbs[0].maximum_value * to_mul.binary_basis_limbs[2].maximum_value);
    max_c2 += (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[2].maximum_value);
    uint256_t max_d0 = (left.binary_basis_limbs[3].maximum_value * to_mul.binary_basis_limbs[0].maximum_value);
    max_d0 += (neg_modulus_limbs_u256[3] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_d1 = (left.binary_basis_limbs[2].maximum_value * to_mul.binary_basis_limbs[1].maximum_value);
    max_d1 += (neg_modulus_limbs_u256[2] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_d2 = (left.binary_basis_limbs[1].maximum_value * to_mul.binary_basis_limbs[2].maximum_value);
    max_d2 += (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[2].maximum_value);
    uint256_t max_d3 = (left.binary_basis_limbs[0].maximum_value * to_mul.binary_basis_limbs[3].maximum_value);
    max_d3 += (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[3].maximum_value);

    uint256_t max_r0 = left.binary_basis_limbs[0].maximum_value * to_mul.binary_basis_limbs[0].maximum_value;
    max_r0 += (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[0].maximum_value);

    const uint256_t max_r1 = max_b0 + max_b1;
    const uint256_t max_r2 = max_c0 + max_c1 + max_c2;
    const uint256_t max_r3 = max_d0 + max_d1 + max_d2 + max_d3;

    uint256_t max_a0(0);
    uint256_t max_a1(0);
    for (size_t i = 0; i < to_add.size(); ++i) {
        max_a0 += to_add[i].binary_basis_limbs[0].maximum_value +
                  (to_add[i].binary_basis_limbs[1].maximum_value << NUM_LIMB_BITS);
        max_a1 += to_add[i].binary_basis_limbs[2].maximum_value +
                  (to_add[i].binary_basis_limbs[3].maximum_value << NUM_LIMB_BITS);
    }
    const uint256_t max_lo = max_r0 + (max_r1 << NUM_LIMB_BITS) + max_a0;
    const uint256_t max_hi = max_r2 + (max_r3 << NUM_LIMB_BITS) + max_a1;

    uint64_t max_lo_bits = max_lo.get_msb() + 1;
    uint64_t max_hi_bits = max_hi.get_msb() + 1;
    if ((max_lo_bits & 1ULL) == 1ULL) {
        ++max_lo_bits;
    }
    if ((max_hi_bits & 1ULL) == 1ULL) {
        ++max_hi_bits;
    }

    const field_t b0 = left.binary_basis_limbs[1].element.madd(
        to_mul.binary_basis_limbs[0].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[0]);
    const field_t b1 = left.binary_basis_limbs[0].element.madd(
        to_mul.binary_basis_limbs[1].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[1]);
    const field_t c0 = left.binary_basis_limbs[1].element.madd(
        to_mul.binary_basis_limbs[1].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[1]);
    const field_t c1 = left.binary_basis_limbs[2].element.madd(
        to_mul.binary_basis_limbs[0].element, quotient.binary_basis_limbs[2].element * neg_modulus_limbs[0]);
    const field_t c2 = left.binary_basis_limbs[0].element.madd(
        to_mul.binary_basis_limbs[2].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[2]);
    const field_t d0 = left.binary_basis_limbs[3].element.madd(
        to_mul.binary_basis_limbs[0].element, quotient.binary_basis_limbs[3].element * neg_modulus_limbs[0]);
    const field_t d1 = left.binary_basis_limbs[2].element.madd(
        to_mul.binary_basis_limbs[1].element, quotient.binary_basis_limbs[2].element * neg_modulus_limbs[1]);
    const field_t d2 = left.binary_basis_limbs[1].element.madd(
        to_mul.binary_basis_limbs[2].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[2]);
    const field_t d3 = left.binary_basis_limbs[0].element.madd(
        to_mul.binary_basis_limbs[3].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[3]);

    // We wish to show that left*right - quotient*remainder = 0 mod 2^t, we do this by collecting the limb products
    // into two separate variables - carry_lo and carry_hi, which are still small enough not to wrap mod r
    // Their first t/2 bits will equal, respectively, the first and second t/2 bits of the expresssion
    // Thus it will suffice to check that each of them begins with t/2 zeroes. We do this by in fact assigning
    // to these variables those expressions divided by 2^{t/2}. Since we have bounds on their ranage that are smaller
    // than r, We can range check the divisions by the original range bounds divided by 2^{t/2}

    const field_t r0 = left.binary_basis_limbs[0].element.madd(
        to_mul.binary_basis_limbs[0].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[0]);

    field_t r1 = b0.add_two(b1, -remainders[0].binary_basis_limbs[1].element);
    const field_t r2 = c0.add_two(c1, c2);
    const field_t r3 = d0 + d1.add_two(d2, d3);

    field_t carry_lo_0 = r0 * shift_right_2;
    field_t carry_lo_1 = r1 * (shift_1 * shift_right_2);
    field_t carry_lo_2 = -(remainders[0].binary_basis_limbs[0].element * shift_right_2);
    field_t carry_lo = carry_lo_0.add_two(carry_lo_1, carry_lo_2);
    for (const auto& add_element : to_add) {
        carry_lo = carry_lo.add_two(add_element.binary_basis_limbs[0].element * shift_right_2,
                                    add_element.binary_basis_limbs[1].element * (shift_1 * shift_right_2));
    }
    for (size_t i = 1; i < remainders.size(); ++i) {
        carry_lo = carry_lo.add_two(-remainders[i].binary_basis_limbs[0].element * shift_right_2,
                                    -remainders[i].binary_basis_limbs[1].element * (shift_1 * shift_right_2));
    }
    field_t t1 = carry_lo.add_two(-remainders[0].binary_basis_limbs[2].element,
                                  -(remainders[0].binary_basis_limbs[3].element * shift_1));
    field_t carry_hi_0 = r2 * shift_right_2;
    field_t carry_hi_1 = r3 * (shift_1 * shift_right_2);
    field_t carry_hi_2 = t1 * shift_right_2;
    field_t carry_hi = carry_hi_0.add_two(carry_hi_1, carry_hi_2);

    for (const auto& add_element : to_add) {
        carry_hi = carry_hi.add_two(add_element.binary_basis_limbs[2].element * shift_right_2,
                                    add_element.binary_basis_limbs[3].element * (shift_1 * shift_right_2));
    }
    for (size_t i = 1; i < remainders.size(); ++i) {
        carry_hi = carry_hi.add_two(-remainders[i].binary_basis_limbs[2].element * shift_right_2,
                                    -remainders[i].binary_basis_limbs[3].element * (shift_1 * shift_right_2));
    }
    barretenberg::fr neg_prime = -barretenberg::fr(uint256_t(target_basis.modulus));

    field_t<C> linear_terms(ctx, barretenberg::fr(0));
    if (to_add.size() >= 2) {
        for (size_t i = 0; i < to_add.size(); i += 2) {
            linear_terms = linear_terms.add_two(to_add[i].prime_basis_limb, to_add[i + 1].prime_basis_limb);
        }
    }
    if ((to_add.size() & 1UL) == 1UL) {
        linear_terms += to_add[to_add.size() - 1].prime_basis_limb;
    }
    if (remainders.size() >= 2) {
        for (size_t i = 0; i < remainders.size(); i += 2) {
            linear_terms = linear_terms.add_two(-remainders[i].prime_basis_limb, -remainders[i + 1].prime_basis_limb);
        }
    }
    if ((remainders.size() & 1UL) == 1UL) {
        linear_terms += -remainders[remainders.size() - 1].prime_basis_limb;
    }

    // This is where we show our identity is zero mod r (to use CRT we show it's zero mod r and mod 2^t)
    field_t<C>::evaluate_polynomial_identity(
        left.prime_basis_limb, to_mul.prime_basis_limb, quotient.prime_basis_limb * neg_prime, linear_terms);

    const uint64_t carry_lo_msb = max_lo_bits - (2 * NUM_LIMB_BITS);
    const uint64_t carry_hi_msb = max_hi_bits - (2 * NUM_LIMB_BITS);

    const barretenberg::fr carry_lo_shift(uint256_t(uint256_t(1) << carry_lo_msb));
    // std::cout <<"lowmsb:" << carry_lo_msb << " highmsb:" <<carry_hi_msb <<std::endl;
    if constexpr (C::type == waffle::PLOOKUP) {
        carry_lo = carry_lo.normalize();
        // std::cout <<"carrylow:" << carry_lo << std::endl;
        // std::cout <<"carrylowmsb:" << ((uint256_t)ctx->get_variable(carry_lo.witness_index)).get_msb() << " bound:"
        // << carry_lo_msb <<std::endl;
        carry_hi = carry_hi.normalize();
        ctx->decompose_into_default_range(carry_lo.witness_index, static_cast<size_t>(carry_lo_msb));
        ctx->decompose_into_default_range(carry_hi.witness_index, static_cast<size_t>(carry_hi_msb));

    } else {
        field_t carry_combined = carry_lo + (carry_hi * carry_lo_shift);
        carry_combined = carry_combined.normalize();
        const auto accumulators = ctx->create_range_constraint(carry_combined.witness_index,
                                                               static_cast<size_t>(carry_lo_msb + carry_hi_msb));
        carry_hi = carry_hi.normalize();
        ctx->assert_equal(carry_hi.witness_index, accumulators[static_cast<size_t>((carry_hi_msb / 2) - 1)]);
    }
}

template <typename C, typename T>
void bigfield<C, T>::evaluate_square_add(const bigfield& left,
                                         const std::vector<bigfield>& to_add,
                                         const bigfield& quotient,
                                         const bigfield& remainder)
{
    C* ctx = left.context == nullptr ? quotient.context : left.context;

    uint256_t max_b0 = (left.binary_basis_limbs[1].maximum_value * left.binary_basis_limbs[0].maximum_value);
    max_b0 += (neg_modulus_limbs_u256[1] << NUM_LIMB_BITS);
    max_b0 += max_b0;
    uint256_t max_c0 = (left.binary_basis_limbs[1].maximum_value * left.binary_basis_limbs[1].maximum_value);
    max_c0 += (neg_modulus_limbs_u256[1] << NUM_LIMB_BITS);
    uint256_t max_c1 = (left.binary_basis_limbs[2].maximum_value * left.binary_basis_limbs[0].maximum_value);
    max_c1 += (neg_modulus_limbs_u256[2] << NUM_LIMB_BITS);
    max_c1 += max_c1;
    uint256_t max_d0 = (left.binary_basis_limbs[3].maximum_value * left.binary_basis_limbs[0].maximum_value);
    max_d0 += (neg_modulus_limbs_u256[3] << NUM_LIMB_BITS);
    max_d0 += max_d0;
    uint256_t max_d1 = (left.binary_basis_limbs[2].maximum_value * left.binary_basis_limbs[1].maximum_value);
    max_d1 += (neg_modulus_limbs_u256[2] << NUM_LIMB_BITS);
    max_d1 += max_d1;

    uint256_t max_r0 = left.binary_basis_limbs[0].maximum_value * left.binary_basis_limbs[0].maximum_value;
    max_r0 += (neg_modulus_limbs_u256[0] << NUM_LIMB_BITS);

    const uint256_t max_r1 = max_b0;
    const uint256_t max_r2 = max_c0 + max_c1;
    const uint256_t max_r3 = max_d0 + max_d1;

    uint256_t max_a0(0);
    uint256_t max_a1(1);
    for (size_t i = 0; i < to_add.size(); ++i) {
        max_a0 += to_add[i].binary_basis_limbs[0].maximum_value +
                  (to_add[i].binary_basis_limbs[1].maximum_value << NUM_LIMB_BITS);
        max_a1 += to_add[i].binary_basis_limbs[2].maximum_value +
                  (to_add[i].binary_basis_limbs[3].maximum_value << NUM_LIMB_BITS);
    }
    const uint256_t max_lo = max_r0 + (max_r1 << NUM_LIMB_BITS) + max_a0;
    const uint256_t max_hi = max_r2 + (max_r3 << NUM_LIMB_BITS) + max_a1;

    uint64_t max_lo_bits = max_lo.get_msb() + 1;
    uint64_t max_hi_bits = max_hi.get_msb() + 1;
    if ((max_lo_bits & 1ULL) == 1ULL) {
        ++max_lo_bits;
    }
    if ((max_hi_bits & 1ULL) == 1ULL) {
        ++max_hi_bits;
    }

    field_t half(ctx, barretenberg::fr(2).invert());
    field_t two(ctx, barretenberg::fr(2));
    field_t b_quotient_0 = (quotient.binary_basis_limbs[1].element * neg_modulus_limbs[0]);
    field_t b_quotient_1 = (quotient.binary_basis_limbs[0].element * neg_modulus_limbs[1]);

    field_t c_quotient_0 = (quotient.binary_basis_limbs[2].element * neg_modulus_limbs[0]);
    field_t c_quotient_1 = (quotient.binary_basis_limbs[0].element * neg_modulus_limbs[2]);

    field_t d_quotient_0 = (quotient.binary_basis_limbs[3].element * neg_modulus_limbs[0]);
    field_t d_quotient_1 = (quotient.binary_basis_limbs[1].element * neg_modulus_limbs[2]);
    field_t d_quotient_2 = (quotient.binary_basis_limbs[0].element * neg_modulus_limbs[3]);
    field_t d_quotient_3 = (quotient.binary_basis_limbs[2].element * neg_modulus_limbs[1]);

    const field_t b0 =
        two * left.binary_basis_limbs[1].element.madd(left.binary_basis_limbs[0].element, b_quotient_0 * half);

    const field_t c0 = left.binary_basis_limbs[1].element.madd(
        left.binary_basis_limbs[1].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[1]);
    const field_t c1 =
        two * left.binary_basis_limbs[2].element.madd(left.binary_basis_limbs[0].element, c_quotient_0 * half);

    const field_t d0 =
        two * left.binary_basis_limbs[3].element.madd(left.binary_basis_limbs[0].element, d_quotient_0 * half);

    const field_t d1 =
        two * left.binary_basis_limbs[2].element.madd(left.binary_basis_limbs[1].element, d_quotient_1 * half);

    const field_t r0 = left.binary_basis_limbs[0].element.madd(
        left.binary_basis_limbs[0].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[0]);

    const field_t r1 = b0.add_two(b_quotient_1, -remainder.binary_basis_limbs[1].element);
    const field_t r2 = c0.add_two(c_quotient_1, c1);
    const field_t r3 = d0.add_two(d_quotient_2, d1) + d_quotient_3;

    field_t carry_lo_0 = r0 * shift_right_2;
    field_t carry_lo_1 = r1 * (shift_1 * shift_right_2);
    field_t carry_lo_2 = -(remainder.binary_basis_limbs[0].element * shift_right_2);
    field_t carry_lo = carry_lo_0.add_two(carry_lo_1, carry_lo_2);

    for (const auto& add_element : to_add) {
        carry_lo = carry_lo.add_two(add_element.binary_basis_limbs[0].element * shift_right_2,
                                    add_element.binary_basis_limbs[1].element * (shift_1 * shift_right_2));
    }

    field_t t1 = carry_lo.add_two(-remainder.binary_basis_limbs[2].element,
                                  -(remainder.binary_basis_limbs[3].element * shift_1));
    field_t carry_hi_0 = r2 * shift_right_2;
    field_t carry_hi_1 = r3 * (shift_1 * shift_right_2);
    field_t carry_hi_2 = t1 * shift_right_2;
    field_t carry_hi = carry_hi_0.add_two(carry_hi_1, carry_hi_2);

    for (const auto& add_element : to_add) {
        carry_hi = carry_hi.add_two(add_element.binary_basis_limbs[2].element * shift_right_2,
                                    add_element.binary_basis_limbs[3].element * (shift_1 * shift_right_2));
    }

    barretenberg::fr neg_prime = -barretenberg::fr(uint256_t(target_basis.modulus));
    field_t<C> linear_terms = -remainder.prime_basis_limb;
    if (to_add.size() >= 2) {
        for (size_t i = 0; i < to_add.size(); i += 2) {
            linear_terms = linear_terms.add_two(to_add[i].prime_basis_limb, to_add[i + 1].prime_basis_limb);
        }
    }
    if ((to_add.size() & 1UL) == 1UL) {
        linear_terms += to_add[to_add.size() - 1].prime_basis_limb;
    }

    field_t<C>::evaluate_polynomial_identity(
        left.prime_basis_limb, left.prime_basis_limb, quotient.prime_basis_limb * neg_prime, linear_terms);

    const uint64_t carry_lo_msb = max_lo_bits - (2 * NUM_LIMB_BITS);
    const uint64_t carry_hi_msb = max_hi_bits - (2 * NUM_LIMB_BITS);

    const barretenberg::fr carry_lo_shift(uint256_t(uint256_t(1) << carry_lo_msb));
    if constexpr (C::type == waffle::PLOOKUP) {
        carry_lo = carry_lo.normalize();
        carry_hi = carry_hi.normalize();
        ctx->decompose_into_default_range(carry_lo.witness_index, static_cast<size_t>(carry_lo_msb));
        ctx->decompose_into_default_range(carry_hi.witness_index, static_cast<size_t>(carry_hi_msb));

    } else {
        field_t carry_combined = carry_lo + (carry_hi * carry_lo_shift);
        carry_combined = carry_combined.normalize();
        const auto accumulators = ctx->create_range_constraint(carry_combined.witness_index,
                                                               static_cast<size_t>(carry_lo_msb + carry_hi_msb));
        carry_hi = carry_hi.normalize();
        ctx->assert_equal(carry_hi.witness_index, accumulators[static_cast<size_t>((carry_hi_msb / 2) - 1)]);
    }
}

template <typename C, typename T>
std::pair<uint512_t, uint512_t> bigfield<C, T>::compute_quotient_remainder_values(const bigfield& a,
                                                                                  const bigfield& b,
                                                                                  const std::vector<bigfield>& to_add)
{
    uint512_t add_values(0);
    bool add_constant = true;
    for (const auto& add_element : to_add) {
        add_element.reduction_check();
        add_values += add_element.get_value();
        add_constant = add_constant && (add_element.is_constant());
    }

    const uint1024_t left(a.get_value());
    const uint1024_t right(b.get_value());
    const uint1024_t add_right(add_values);
    const uint1024_t modulus(target_basis.modulus);

    const auto [quotient_1024, remainder_1024] = (left * right + add_right).divmod(modulus);

    return { quotient_1024.lo, remainder_1024.lo };
}

/**
 * USED FOR TESTS ONLY! DO NOT USE IN PRODUCTION CODE!
 */
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::bad_mul(const bigfield& other) const
{
    reduction_check();
    other.reduction_check();

    C* ctx = context ? context : other.context;

    // const uint1024_t left(get_value());
    // const uint1024_t right(other.get_value());
    /**
     * Tn/p = q
     *
     * q = |Tn/p|
     * qp + r > Tn
     **/
    const uint1024_t modulus(target_basis.modulus);
    const uint1024_t one(1);
    const uint1024_t t = one << (68 * 4);
    const uint1024_t n = uint1024_t(uint512_t(barretenberg::fr::modulus));
    const uint1024_t t_n = t * n;

    const auto [quotient_1024, remainder_1024] = (t_n).divmod(modulus);

    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    if (is_constant() && other.is_constant()) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        // when writing a*b = q*p + r we wish to enforce r<2^s for smallest s such that p<2^s
        // hence the second constructor call is with can_overflow=false. This will allow using r in more additions mod
        // 2^t without needing to apply the mod, where t=4*NUM_LIMB_BITS
        const size_t num_quotient_bits = get_quotient_max_bits({ DEFAULT_MAXIMUM_REMAINDER });
        quotient = bigfield(witness_t(ctx, fr(quotient_value.slice(0, NUM_LIMB_BITS * 2).lo)),
                            witness_t(ctx, fr(quotient_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                            false,
                            num_quotient_bits);
        remainder = bigfield(
            witness_t(ctx, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
            witness_t(ctx, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));
    };

    evaluate_multiply_add(*this, other, {}, quotient, { remainder });
    return remainder;
}

} // namespace stdlib
} // namespace plonk