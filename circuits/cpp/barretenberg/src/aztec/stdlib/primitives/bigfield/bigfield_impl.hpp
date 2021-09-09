#pragma once

#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>
#include <tuple>

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
bigfield<C, T>::bigfield(const field_t<C>& low_bits_in, const field_t<C>& high_bits_in, const bool can_overflow)
{
    const auto low_bits = low_bits_in.normalize();
    const auto high_bits = high_bits_in.normalize();
    context = low_bits.context == nullptr ? high_bits.context : low_bits.context;
    field_t<C> limb_0(context);
    field_t<C> limb_1(context);
    field_t<C> limb_2(context);
    field_t<C> limb_3(context);
    if (low_bits.witness_index != IS_CONSTANT) {
        std::vector<uint32_t> low_accumulator;
        if constexpr (C::type == waffle::PLOOKUP) {
            // If this doesn't hold we're using a default plookup range size that doesn't work well with the limb size
            // here
            ASSERT(low_accumulator.size() % 2 == 0);
            // Enforce that low_bits indeed only contains 2*NUM_LIMB_BITS bits
            low_accumulator =
                context->decompose_into_default_range(low_bits.witness_index, static_cast<size_t>(NUM_LIMB_BITS * 2));
            size_t mid_index = low_accumulator.size() / 2 - 1;
            limb_0.witness_index = low_accumulator[mid_index]; // Q:safer to just slice this from low_bits?
            limb_1 = (low_bits - limb_0) * shift_right_1;
        } else {
            size_t mid_index;
            low_accumulator = context->decompose_into_base4_accumulators(low_bits.witness_index,
                                                                         static_cast<size_t>(NUM_LIMB_BITS * 2));
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
    if ((num_last_limb_bits & 1ULL) == 1ULL) {
        ++num_last_limb_bits;
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
            high_accumulator = context->decompose_into_base4_accumulators(high_bits.witness_index,
                                                                          static_cast<size_t>(num_high_limb_bits));
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

template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::create_from_u512_as_witness(C* ctx, const uint512_t& value, const bool can_overflow)
{
    std::array<uint256_t, 4> limbs;
    limbs[0] = value.slice(0, NUM_LIMB_BITS).lo;
    limbs[1] = value.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo;
    limbs[2] = value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo;
    limbs[3] = value.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo;

    return bigfield(witness_t(ctx, fr(limbs[0] + limbs[1] * shift_1)),
                    witness_t(ctx, fr(limbs[2] + limbs[3] * shift_1)),
                    can_overflow);
}

template <typename C, typename T> bigfield<C, T>::bigfield(const byte_array<C>& bytes)
{

    const auto split_byte_into_nibbles = [](C* ctx, const field_t<C>& split_byte) {
        const uint64_t byte_val = uint256_t(split_byte.get_value()).data[0];
        const uint64_t lo_nibble_val = byte_val & 15ULL;
        const uint64_t hi_nibble_val = byte_val >> 4;

        const field_t<C> lo_nibble(witness_t<C>(ctx, lo_nibble_val));
        const field_t<C> hi_nibble(witness_t<C>(ctx, hi_nibble_val));
        lo_nibble.create_range_constraint(4);
        hi_nibble.create_range_constraint(4);
        const field_t<C> sum = lo_nibble + (hi_nibble * 16);
        sum.assert_equal(split_byte);
        return std::make_pair<field_t<C>, field_t<C>>((field_t<C>)lo_nibble, (field_t<C>)hi_nibble);
    };

    const auto reconstruct_two_limbs = [&split_byte_into_nibbles](C* ctx,
                                                                  const field_t<C>& hi_bytes,
                                                                  const field_t<C>& lo_bytes,
                                                                  const field_t<C>& split_byte) {
        const auto [lo_nibble, hi_nibble] = split_byte_into_nibbles(ctx, split_byte);

        field_t<C> hi_limb = hi_nibble + hi_bytes * 16;
        field_t<C> lo_limb = lo_bytes + lo_nibble * field_t<C>(ctx, uint256_t(1) << 64);
        return std::make_pair<field_t<C>, field_t<C>>((field_t<C>)lo_limb, (field_t<C>)hi_limb);
    };
    C* ctx = bytes.get_context();

    const field_t<C> hi_8_bytes(bytes.slice(0, 6));
    const field_t<C> mid_split_byte(bytes.slice(6, 1));
    const field_t<C> mid_8_bytes(bytes.slice(7, 8));

    const field_t<C> lo_8_bytes(bytes.slice(15, 8));
    const field_t<C> lo_split_byte(bytes.slice(23, 1));
    const field_t<C> lolo_8_bytes(bytes.slice(24, 8));

    const auto [limb0, limb1] = reconstruct_two_limbs(ctx, lo_8_bytes, lolo_8_bytes, lo_split_byte);
    const auto [limb2, limb3] = reconstruct_two_limbs(ctx, hi_8_bytes, mid_8_bytes, mid_split_byte);

    const auto res = bigfield(limb0, limb1, limb2, limb3, true);

    *this = res;
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

    bigfield result(ctx);
    result.binary_basis_limbs[0].maximum_value =
        binary_basis_limbs[0].maximum_value + other.binary_basis_limbs[0].maximum_value;
    result.binary_basis_limbs[1].maximum_value =
        binary_basis_limbs[1].maximum_value + other.binary_basis_limbs[1].maximum_value;
    result.binary_basis_limbs[2].maximum_value =
        binary_basis_limbs[2].maximum_value + other.binary_basis_limbs[2].maximum_value;
    result.binary_basis_limbs[3].maximum_value =
        binary_basis_limbs[3].maximum_value + other.binary_basis_limbs[3].maximum_value;

    result.binary_basis_limbs[0].element = binary_basis_limbs[0].element + other.binary_basis_limbs[0].element;
    result.binary_basis_limbs[1].element = binary_basis_limbs[1].element + other.binary_basis_limbs[1].element;
    result.binary_basis_limbs[2].element = binary_basis_limbs[2].element + other.binary_basis_limbs[2].element;
    result.binary_basis_limbs[3].element = binary_basis_limbs[3].element + other.binary_basis_limbs[3].element;
    result.prime_basis_limb = prime_basis_limb + other.prime_basis_limb;
    return result;
}

// to make sure we don't go to negative values, add p before subtracting other
/**
 * Subtraction operator.
 *
 * Like operator+, we use lazy reduction techniques to save on field reductions.
 *
 * Instead of computing `*this - other`, we compute offset X and compute:
 * `*this + X - other`
 * This ensures we do not underflow!
 *
 * Offset `X` will be a multiple of our bigfield modulus `p`
 *
 * i.e `X = m * p`
 *
 * It is NOT enough to ensure that the integer value of `*this + X - other` does not underflow.
 * We must ALSO ensure that each LIMB of the result does not underflow
 *
 * We must compute the MINIMUM value of `m` that ensures that none of the bigfield limbs will underflow!
 *
 * i.e. We must compute the MINIMUM value of `m` such that, for each limb `i`, the following result is positive:
 *
 * *this.limb[i] + X.limb[i] - other.limb[i]
 **/
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

    /**
     * Step 1: For each limb compute the MAXIMUM value we will have to borrow from the next significant limb
     *
     * i.e. if we assume that `*this = 0` and `other = other.maximum_value`, how many bits do we need to borrow from
     * the next significant limb to ensure each limb value is positive?
     *
     * N.B. for this segment `maximum_value` really refers to maximum NEGATIVE value of the result
     **/
    uint256_t limb_0_maximum_value = other.binary_basis_limbs[0].maximum_value;

    // Compute maximum shift factor for limb_0
    uint64_t limb_0_borrow_shift = std::max(limb_0_maximum_value.get_msb() + 1, NUM_LIMB_BITS);

    // Compute the maximum negative value of limb_1, including the bits limb_0 may need to borrow
    uint256_t limb_1_maximum_value =
        other.binary_basis_limbs[1].maximum_value + (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS));

    // repeat the above for the remaining limbs
    uint64_t limb_1_borrow_shift = std::max(limb_1_maximum_value.get_msb() + 1, NUM_LIMB_BITS);
    uint256_t limb_2_maximum_value =
        other.binary_basis_limbs[2].maximum_value + (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS));
    uint64_t limb_2_borrow_shift = std::max(limb_2_maximum_value.get_msb() + 1, NUM_LIMB_BITS);

    uint256_t limb_3_maximum_value =
        other.binary_basis_limbs[3].maximum_value + (uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    /**
     * Step 2: Compute the constant value `X = m * p` we must add to the result to ensure EVERY limb is >= 0
     *
     * We need to find a value `X` where `X.limb[3] > limb_3_maximum_value`.
     * As long as the above holds, we can borrow bits from X.limb[3] to ensure less significant limbs are positive
     *
     * Start by setting constant_to_add = p
     **/
    uint512_t constant_to_add = modulus_u512;
    // add a large enough multiple of p to not get negative result in subtraction
    while (constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo <= limb_3_maximum_value) {
        constant_to_add += modulus_u512;
    }

    /**
     * Step 3: Compute offset terms t0, t1, t2, t3 that we add to our result to ensure each limb is positive
     *
     * t3 represents the value we are BORROWING from constant_to_add.limb[3]
     * t2, t1, t0 are the terms we will ADD to constant_to_add.limb[2], constant_to_add.limb[1], constant_to_add.limb[0]
     *
     * i.e. The net value we add to `constant_to_add` is 0. We must ensure that:
     * t3 = t0 + (t1 << NUM_LIMB_BITS) + (t2 << NUM_LIMB_BITS * 2)
     *
     * e.g. the value we borrow to produce t0 is subtracted from t1,
     *      the value we borrow from t1 is subtracted from t2
     *      the value we borrow from t2 is equal to t3
     **/
    uint256_t t0(uint256_t(1) << limb_0_borrow_shift);
    uint256_t t1((uint256_t(1) << limb_1_borrow_shift) - (uint256_t(1) << (limb_0_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t2((uint256_t(1) << limb_2_borrow_shift) - (uint256_t(1) << (limb_1_borrow_shift - NUM_LIMB_BITS)));
    uint256_t t3(uint256_t(1) << (limb_2_borrow_shift - NUM_LIMB_BITS));

    /**
     * Compute the limbs of `constant_to_add`, including our offset terms t0, t1, t2, t3 that ensure each result limb is
     *positive
     **/
    uint256_t to_add_0 = uint256_t(constant_to_add.slice(0, NUM_LIMB_BITS)) + t0;
    uint256_t to_add_1 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2)) + t1;
    uint256_t to_add_2 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3)) + t2;
    uint256_t to_add_3 = uint256_t(constant_to_add.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4)) - t3;

    /**
     * Update the maximum possible value of the result. We assume here that (*this.value) = 0
     **/
    result.binary_basis_limbs[0].maximum_value = binary_basis_limbs[0].maximum_value + to_add_0;
    result.binary_basis_limbs[1].maximum_value = binary_basis_limbs[1].maximum_value + to_add_1;
    result.binary_basis_limbs[2].maximum_value = binary_basis_limbs[2].maximum_value + to_add_2;
    result.binary_basis_limbs[3].maximum_value = binary_basis_limbs[3].maximum_value + to_add_3;

    /**
     * Compute the binary basis limbs of our result
     **/
    result.binary_basis_limbs[0].element = binary_basis_limbs[0].element + barretenberg::fr(to_add_0);
    result.binary_basis_limbs[1].element = binary_basis_limbs[1].element + barretenberg::fr(to_add_1);
    result.binary_basis_limbs[2].element = binary_basis_limbs[2].element + barretenberg::fr(to_add_2);
    result.binary_basis_limbs[3].element = binary_basis_limbs[3].element + barretenberg::fr(to_add_3);
    result.binary_basis_limbs[0].element -= other.binary_basis_limbs[0].element;
    result.binary_basis_limbs[1].element -= other.binary_basis_limbs[1].element;
    result.binary_basis_limbs[2].element -= other.binary_basis_limbs[2].element;
    result.binary_basis_limbs[3].element -= other.binary_basis_limbs[3].element;

    /**
     * Compute the prime basis limb of the result
     **/
    uint512_t constant_to_add_mod_p = (constant_to_add) % prime_basis.modulus;
    field_t prime_basis_to_add(ctx, barretenberg::fr(constant_to_add_mod_p.lo));
    result.prime_basis_limb = prime_basis_limb + prime_basis_to_add;
    result.prime_basis_limb -= other.prime_basis_limb;
    return result;
}

/**
 * Evaluate a non-native field multiplication: (a * b = c mod p) where p == target_basis.modulus
 *
 * We compute quotient term `q` and remainder `c` and evaluate that:
 *
 * a * b - q * p - c = 0 mod modulus_u512 (binary basis modulus, currently 2**272)
 * a * b - q * p - c = 0 mod circuit modulus
 **/
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator*(const bigfield& other) const
{
    C* ctx = context ? context : other.context;
    reduction_check();
    other.reduction_check();

    /**
     * Step 1: compute the numeric values of quotient and remainder
     *
     * We first convert our numeric values to 1024-bit integers.
     * Our operands can take values up to modulus_u512 - 1 (2**272 - 1).
     * Multiplying our operands can produce values > 512 bits therfore we need 1024-bit arithmetic
     **/
    const uint1024_t left(get_value());
    const uint1024_t right(other.get_value());
    const uint1024_t modulus(target_basis.modulus);
    const auto [quotient_1024, remainder_1024] = (left * right).divmod(modulus);
    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    // If operands are constant, define result as a constant value and return
    if (is_constant() && other.is_constant()) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        // when writing a*b = q*p + r we wish to enforce r<2^s for smallest s such that p<2^s
        // hence the second constructor call is with can_overflow=false. This will allow using r in more additions mod
        // 2^t without needing to apply the mod, where t=4*NUM_LIMB_BITS
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        remainder = create_from_u512_as_witness(ctx, remainder_value);
    };

    // Call `evaluate_multiply_add` to validate the correctness of our computed quotient and remainder
    evaluate_multiply_add(*this, other, {}, quotient, { remainder });
    return remainder;
}

// TODO: reduce code duplication between this and ::div
/**
 * Division operator
 *
 * To evaluate (a / b = c mod p), we instead evaluate (c * b = a mod p)
 **/
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::operator/(const bigfield& other) const
{
    C* ctx = context ? context : other.context;
    reduction_check();
    other.reduction_check();
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
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        inverse = create_from_u512_as_witness(ctx, inverse_value);
    }

    evaluate_multiply_add(other, inverse, {}, quotient, { *this });
    return inverse;
}

/**
 * Compute a * a = c mod p
 *
 * Slightly cheaper than operator* for StandardPlonk and TurboPlonk
 **/
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::sqr() const
{
    reduction_check();
    C* ctx = context;

    const uint1024_t left(get_value());
    const uint1024_t right(get_value());
    const uint1024_t modulus(target_basis.modulus);

    const auto [quotient_1024, remainder_1024] = (left * right).divmod(modulus);

    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    if (is_constant()) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        remainder = create_from_u512_as_witness(ctx, remainder_value);
    };

    evaluate_square_add(*this, {}, quotient, remainder);
    return remainder;
}

/**
 * Compute a * a + ...to_add = b mod p
 *
 * We can chain multiple additions to a square/multiply with a single quotient/remainder.
 *
 * Chaining the additions here is cheaper than calling operator+ because we can combine some gates in
 *`evaluate_multiply_add`
 **/
template <typename C, typename T> bigfield<C, T> bigfield<C, T>::sqradd(const std::vector<bigfield>& to_add) const
{
    C* ctx = context;
    reduction_check();

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
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        remainder = create_from_u512_as_witness(ctx, remainder_value);
    };
    evaluate_square_add(*this, to_add, quotient, remainder);
    return remainder;
}

/**
 * Compute a * b + ...to_add = c mod p
 **/
template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::madd(const bigfield& to_mul, const std::vector<bigfield>& to_add) const
{
    C* ctx = context ? context : to_mul.context;
    reduction_check();
    to_mul.reduction_check();

    uint512_t add_values(0);
    bool add_constant = true;
    for (const auto& add_element : to_add) {
        add_element.reduction_check();
        add_values += add_element.get_value();
        add_constant = add_constant && (add_element.is_constant());
    }

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
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        remainder = create_from_u512_as_witness(ctx, remainder_value);
    };
    evaluate_multiply_add(*this, to_mul, to_add, quotient, { remainder });
    return remainder;
}

/**
 * Compute (left_a * right_a) + (left_b * right_b) + ...to_add = c mod p
 *
 * This is cheaper than two multiplication operations, as the above only requires one quotient/remainder
 **/
template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::dual_madd(const bigfield& left_a,
                                         const bigfield& right_a,
                                         const bigfield& left_b,
                                         const bigfield& right_b,
                                         const std::vector<bigfield>& to_add,
                                         cached_product& cache)
{
    left_a.reduction_check();
    right_a.reduction_check();
    left_b.reduction_check();
    right_b.reduction_check();

    uint512_t add_values(0);
    bool add_constant = true;
    for (const auto& add_element : to_add) {
        add_element.reduction_check();
        add_values += add_element.get_value();
        add_constant = add_constant && (add_element.is_constant());
    }

    C* ctx = left_a.context ? left_a.context : right_a.context;

    const uint1024_t left_a_val(left_a.get_value());
    const uint1024_t right_a_val(right_a.get_value());
    const uint1024_t left_b_val(left_b.get_value());
    const uint1024_t right_b_val(right_b.get_value());

    const uint1024_t add_right(add_values);
    const uint1024_t modulus(target_basis.modulus);

    const auto [quotient_1024, remainder_1024] =
        ((left_a_val * right_a_val) + (left_b_val * right_b_val) + add_right).divmod(modulus);

    const uint512_t quotient_value = quotient_1024.lo;
    const uint512_t remainder_value = remainder_1024.lo;

    bigfield remainder;
    bigfield quotient;
    if (left_a.is_constant() && right_a.is_constant() && left_b.is_constant() && right_b.is_constant() &&
        add_constant) {
        remainder = bigfield(ctx, uint256_t(remainder_value.lo));
        return remainder;
    } else {
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        remainder = create_from_u512_as_witness(ctx, remainder_value);
    };

    std::vector<cached_product> cachevector{ cache };
    evaluate_multiple_multiply_add(
        { left_a, left_b }, { right_a, right_b }, to_add, quotient, { remainder }, cachevector);
    cache = cachevector[0];
    return remainder;
}

/**
 * multiply, subtract, divide.
 * This method computes:
 *
 * result = -(\sum{mul_left[i] * mul_right[i]} + ...to_add) / divisor
 *
 * Algorithm is constructed in this way to ensure that all computed terms are positive
 *
 * i.e. we evaluate:
 * result * divisor + (\sum{mul_left[i] * mul_right[i]) + ...to_add) = 0
 *
 * It is critical that ALL the terms on the LHS are positive to eliminate the possiblity of underflows
 * when calling `evaluate_multiple_multiply_add`
 *
 * only requires one quotient and remainder + overflow limbs
 * `cache` stores the unreduced value of `mul_left * mul_right`, as this can
 * be shared across multiple msub_div calls that contain the same numerator
 **/
template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::msub_div(const std::vector<bigfield>& mul_left,
                                        const std::vector<bigfield>& mul_right,
                                        const bigfield& divisor,
                                        const std::vector<bigfield>& to_sub,
                                        cached_product& cache)
{
    C* ctx = divisor.context;

    const size_t num_multiplications = mul_left.size();

    native product_native = 0;
    uint1024_t product_1024 = 0;
    bool products_constant = true;
    for (size_t i = 0; i < num_multiplications; ++i) {
        mul_left[i].reduction_check();
        mul_right[i].reduction_check();
        const native mul_left_native(uint512_t(mul_left[i].get_value() % modulus_u512).lo);
        const native mul_right_native(uint512_t(mul_right[i].get_value() % modulus_u512).lo);
        product_native += (mul_left_native * -mul_right_native);
        products_constant = products_constant && mul_left[i].is_constant() && mul_right[i].is_constant();

        const uint1024_t mul_left_v(mul_left[i].get_value());
        const uint1024_t mul_right_v(mul_right[i].get_value());
        product_1024 += (mul_left_v * mul_right_v);
    }

    divisor.reduction_check();
    native sub_native(0);
    bool sub_constant = true;
    for (const auto& sub : to_sub) {
        sub.reduction_check();
        sub_native += (uint512_t(sub.get_value() % modulus_u512).lo);
        sub_constant = sub_constant && sub.is_constant();
    }

    native divisor_native(uint512_t(divisor.get_value() % modulus_u512).lo);

    const native result_native = (product_native - sub_native) / divisor_native;

    const uint1024_t result_value = uint1024_t(uint512_t(static_cast<uint256_t>(result_native)));

    if (sub_constant && products_constant && divisor.is_constant()) {
        return bigfield(ctx, uint256_t(result_value.lo.lo));
    }

    const uint1024_t divisor_v(divisor.get_value());
    uint1024_t sub_v(0);
    for (const auto& sub : to_sub) {
        sub_v += uint1024_t(sub.get_value());
    }

    const uint1024_t identity = result_value * divisor_v + product_1024 + sub_v;

    const auto quotient_1024 = identity / uint1024_t(modulus_u512);

    bigfield result = create_from_u512_as_witness(ctx, result_value.lo);
    bigfield quotient = create_from_u512_as_witness(ctx, quotient_1024.lo, true);

    std::vector<cached_product> cachevector{ cache };

    std::vector<bigfield> eval_left{ result };
    std::vector<bigfield> eval_right{ divisor };
    for (const auto& in : mul_left) {
        eval_left.emplace_back(in);
    }
    for (const auto& in : mul_right) {
        eval_right.emplace_back(in);
    }
    for (size_t i = 1; i < num_multiplications; ++i) {
        cachevector.push_back(cached_product());
    }
    evaluate_multiple_multiply_add(eval_left, eval_right, to_sub, quotient, {}, cachevector /*{ cache }*/);
    cache = cachevector[0];
    return result;
}

/**
 * Div method.
 *
 * Similar to operator/ but numerator can be linear sum of multiple elements
 *
 * TODO: reduce code duplication. Should operator/ call div ? Would this add any gates to operator/?
 **/
template <typename C, typename T>
bigfield<C, T> bigfield<C, T>::div(const std::vector<bigfield>& numerators, const bigfield& denominator)
{
    C* ctx = denominator.context;
    if (numerators.size() == 0) {
        return bigfield<C, T>(nullptr, uint256_t(0));
    }

    denominator.reduction_check();
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
        quotient = create_from_u512_as_witness(ctx, quotient_value, true);
        inverse = create_from_u512_as_witness(ctx, inverse_value);
    }

    evaluate_multiply_add(denominator, inverse, {}, quotient, numerators);
    return inverse;
}

template <typename C, typename T> bigfield<C, T> bigfield<C, T>::conditional_negate(const bool_t<C>& predicate) const
{
    C* ctx = context ? context : predicate.context;

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
    reduction_check();
    C* ctx = context ? context : (other.context ? other.context : predicate.context);

    // TODO: use field_t::conditional_assign method
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

/**
 * REDUCTION CHECK
 *
 * When performing bigfield operations, we need to ensure the maximum value is less than:
 *      sqrt(2^{272} * native_modulus)
 *
 * We also need to ensure each binary basis limb is less than the maximum limb value
 *
 * This prevents our field arithmetic from overflowing the native modulus boundary, whilst ensuring we can
 * still use the chinese remainder theorem to validate field multiplications with a reduced number of range checks
 *
 **/
template <typename C, typename T> void bigfield<C, T>::reduction_check() const
{

    if (is_constant()) { // this seems not a reduction check, but actually computing the reduction
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
    constexpr uint256_t modulus_value = modulus_u512.lo;

    constexpr uint256_t modulus_0_value = modulus_value.slice(0, NUM_LIMB_BITS);
    constexpr uint256_t modulus_1_value = modulus_value.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2);
    constexpr uint256_t modulus_2_value = modulus_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3);
    constexpr uint256_t modulus_3_value = modulus_value.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4);

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
    if constexpr (C::type == waffle::PLOOKUP) {
        r0 = r0.normalize();
        r1 = r1.normalize();
        r2 = r2.normalize();
        r3 = r3.normalize();
        context->decompose_into_default_range(r0.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r1.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r2.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
        context->decompose_into_default_range(r3.witness_index, static_cast<size_t>(NUM_LIMB_BITS));
    } else {
        r0.create_range_constraint(static_cast<size_t>(NUM_LIMB_BITS));
        r1.create_range_constraint(static_cast<size_t>(NUM_LIMB_BITS));
        r2.create_range_constraint(static_cast<size_t>(NUM_LIMB_BITS));
        r3.create_range_constraint(static_cast<size_t>(NUM_LIMB_BITS));
    }
}

// check elements are equal mod p by proving their integer difference is a multiple of p.
// This relies on the minus operator for a-b increasing a by a multiple of p large enough so diff is non-negative
template <typename C, typename T> void bigfield<C, T>::assert_equal(const bigfield& other) const
{
    C* ctx = this->context ? this->context : other.context;

    if (is_constant() && other.is_constant()) {
        std::cout << "calling assert equal on 2 CONSTANT bigfield elements...is this intended?" << std::endl;
        return;
    } else if (other.is_constant()) {
        // evaluate a strict equality - make sure *this is reduced first, or an honest prover
        // might not be able to satisfy these constraints.
        field_t<C> t0 = (binary_basis_limbs[0].element - other.binary_basis_limbs[0].element);
        field_t<C> t1 = (binary_basis_limbs[1].element - other.binary_basis_limbs[1].element);
        field_t<C> t2 = (binary_basis_limbs[2].element - other.binary_basis_limbs[2].element);
        field_t<C> t3 = (binary_basis_limbs[3].element - other.binary_basis_limbs[3].element);
        field_t<C> t4 = (prime_basis_limb - other.prime_basis_limb);
        field_t<C> diff = t0 * t1 * t2 * t3 * t4;
        diff.assert_is_zero();
        return;
    } else if (is_constant()) {
        other.assert_equal(*this);
        return;
    }

    bigfield diff = *this - other;
    const uint512_t diff_val = diff.get_value();
    const uint512_t modulus(target_basis.modulus);

    const auto [quotient_512, remainder_512] = (diff_val).divmod(modulus);
    if (remainder_512 != 0)
        std::cout << "remainder not zero!" << std::endl;
    ASSERT(remainder_512 == 0);
    bigfield quotient;

    quotient = bigfield(witness_t(ctx, fr(quotient_512.slice(0, NUM_LIMB_BITS * 2).lo)),
                        witness_t(ctx, fr(quotient_512.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo)),
                        true);
    evaluate_multiply_add(diff, { one() }, {}, quotient, { zero() });
}

// construct a proof that points are different mod p, when they are different mod r
// WARNING: This method doesn't have perfect completeness - for points equal mod r (or with certain difference kp mod r)
// but different mod p, you can't construct a proof.
// The chances of an honest prover running afoul of this condition are extremely small (TODO: compute probability)
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

    field_t<C> diff = prime_basis_limb - other.prime_basis_limb;
    field_t<C> prime_basis(get_context(), modulus);
    field_t<C> prime_basis_accumulator = prime_basis;
    // Each loop iteration adds 1 gate
    // (prime_basis and prime_basis accumulator are constant so only the * operator adds a gate)
    for (size_t i = 0; i < lhs_overload_count; ++i) {
        diff = diff * (diff - prime_basis_accumulator);
        prime_basis_accumulator += prime_basis;
    }
    prime_basis_accumulator = prime_basis;
    for (size_t i = 0; i < rhs_overload_count; ++i) {
        diff = diff * (diff + prime_basis_accumulator);
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
    // TODO: handle situation where some limbs are constant and others are not constant
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
        quotient_limb.create_range_constraint(static_cast<size_t>(maximum_quotient_bits));
    }
    quotient.binary_basis_limbs[0] = Limb(quotient_limb, uint256_t(1) << maximum_quotient_bits);
    quotient.binary_basis_limbs[1] = Limb(field_t<C>::from_witness_index(context, context->zero_idx), 0);
    quotient.binary_basis_limbs[2] = Limb(field_t<C>::from_witness_index(context, context->zero_idx), 0);
    quotient.binary_basis_limbs[3] = Limb(field_t<C>::from_witness_index(context, context->zero_idx), 0);
    quotient.prime_basis_limb = quotient_limb;
    // this constructor with can_overflow=false will create remainder of size<2^s
    bigfield remainder = bigfield(
        witness_t(context, fr(remainder_value.slice(0, NUM_LIMB_BITS * 2).lo)),
        witness_t(context, fr(remainder_value.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS).lo)));

    evaluate_multiply_add(*this, one(), {}, quotient, { remainder });
    binary_basis_limbs[0] = remainder.binary_basis_limbs[0]; // how is this method const?
    binary_basis_limbs[1] = remainder.binary_basis_limbs[1];
    binary_basis_limbs[2] = remainder.binary_basis_limbs[2];
    binary_basis_limbs[3] = remainder.binary_basis_limbs[3];
    prime_basis_limb = remainder.prime_basis_limb;
}

// See explanation at https://hackmd.io/LoEG5nRHQe-PvstVaD51Yw?view
template <typename C, typename T>
void bigfield<C, T>::evaluate_multiply_add(const bigfield& input_left,
                                           const bigfield& input_to_mul,
                                           const std::vector<bigfield>& to_add,
                                           const bigfield& input_quotient,
                                           const std::vector<bigfield>& input_remainders)
{

    std::vector<bigfield> remainders(input_remainders);
    bigfield left = input_left;
    bigfield to_mul = input_to_mul;
    bigfield quotient = input_quotient;

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

    uint64_t max_lo_bits = (max_lo.get_msb() + 1);
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
    // to these variables those expressions divided by 2^{t/2}. Since we have bounds on their ranage that are
    // smaller than r, We can range check the divisions by the original range bounds divided by 2^{t/2}

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
        carry_hi = carry_hi.normalize();
        ctx->decompose_into_default_range(carry_lo.witness_index, static_cast<size_t>(carry_lo_msb));
        ctx->decompose_into_default_range(carry_hi.witness_index, static_cast<size_t>(carry_hi_msb));

    } else {
        field_t carry_combined = carry_lo + (carry_hi * carry_lo_shift);
        carry_combined = carry_combined.normalize();
        const auto accumulators = ctx->decompose_into_base4_accumulators(
            carry_combined.witness_index, static_cast<size_t>(carry_lo_msb + carry_hi_msb));
        field_t<C> accumulator_midpoint =
            field_t<C>::from_witness_index(ctx, accumulators[static_cast<size_t>((carry_hi_msb / 2) - 1)]);
        carry_hi.assert_equal(accumulator_midpoint, "bigfield multiply range check failed");
    }
}

/**
 * Evaluate a quadratic relation involving multiple multiplications
 *
 * i.e. evalaute:
 *
 * (left_0 * right_0) + ... + (left_n-1 * right_n-1) + ...to_add - (input_quotient * q + ...input_remainders) = 0
 *
 * This method supports multiple "remainders" because, when evaluating divisions, some of these remainders are terms
 *we're subtracting from our product (see msub_div for more details)
 *
 * The above quadratic relation can be evaluated using only a single quotient/remainder term.
 *
 * Params:
 *
 * `input_left`: left multiplication operands
 * `input_right` : right multiplication operands
 * `to_add` : vector of elements to add to the product
 * `input_quotient` : quotient
 * `input_remainders` : vector of remainders
 * `caches` : vector of cached prior multiplication results
 *
 * ### explanation of std::vector<cached_product>& caches ###
 *
 * The `caches` vector exists to avoid creating redundant gates across multiple calls to this method.
 *
 * E.g. consider the following relations:
 *  a * b + c * d = x
 *  e * f + c * d = y
 *
 * It is NOT efficient to compute intermediate bigfield element (c * d) because that adds an additional bigfield
 *reduction (i.e. creation of a quotient and remainder) HOWEVER inside `evaluate_multiple_multiply_add` we will compute
 *the limb products of `c * d` i.e (c[0] * d[0], c[1] * d[0] etc etc) We don't want to do this twice! This is where
 *`cached_product` comes in. `cached_product` stores UNREDUCED limb multiplications.
 *
 * We ASSUME the first product in `evaluate_multiple_multiply_add` is unique.
 * i.e. (input_left[0] * input_right[0]) is NOT going to appear again in the circuit.
 *
 * For subsequent product terms, we will examine `caches` to check if the limb products have previously been computed.
 * If they have been, this method will skip over computing them again.
 * if they have not, this method will compute the limb products and populate `caches` for potential future
 *multiplication operations.
 **/
template <typename C, typename T>
void bigfield<C, T>::evaluate_multiple_multiply_add(const std::vector<bigfield>& input_left,
                                                    const std::vector<bigfield>& input_right,
                                                    const std::vector<bigfield>& to_add,
                                                    const bigfield& input_quotient,
                                                    const std::vector<bigfield>& input_remainders,
                                                    std::vector<cached_product>& caches)
{

    std::vector<bigfield> remainders(input_remainders);
    std::vector<bigfield> left(input_left);
    std::vector<bigfield> right(input_right);
    bigfield quotient = input_quotient;
    const size_t num_multiplications = input_left.size();

    C* ctx = input_left[0].context ? input_left[0].context : input_right[0].context;

    const auto get_product_maximum = [](const bigfield& left, const bigfield& right) {
        uint256_t max_b0_inner = (left.binary_basis_limbs[1].maximum_value * right.binary_basis_limbs[0].maximum_value);
        uint256_t max_b1_inner = (left.binary_basis_limbs[0].maximum_value * right.binary_basis_limbs[1].maximum_value);
        uint256_t max_c0_inner = (left.binary_basis_limbs[1].maximum_value * right.binary_basis_limbs[1].maximum_value);
        uint256_t max_c1_inner = (left.binary_basis_limbs[2].maximum_value * right.binary_basis_limbs[0].maximum_value);
        uint256_t max_c2_inner = (left.binary_basis_limbs[0].maximum_value * right.binary_basis_limbs[2].maximum_value);
        uint256_t max_d0_inner = (left.binary_basis_limbs[3].maximum_value * right.binary_basis_limbs[0].maximum_value);
        uint256_t max_d1_inner = (left.binary_basis_limbs[2].maximum_value * right.binary_basis_limbs[1].maximum_value);
        uint256_t max_d2_inner = (left.binary_basis_limbs[1].maximum_value * right.binary_basis_limbs[2].maximum_value);
        uint256_t max_d3_inner = (left.binary_basis_limbs[0].maximum_value * right.binary_basis_limbs[3].maximum_value);
        uint256_t max_r0_inner = left.binary_basis_limbs[0].maximum_value * right.binary_basis_limbs[0].maximum_value;

        const uint256_t max_r1_inner = max_b0_inner + max_b1_inner;
        const uint256_t max_r2_inner = max_c0_inner + max_c1_inner + max_c2_inner;
        const uint256_t max_r3_inner = max_d0_inner + max_d1_inner + max_d2_inner + max_d3_inner;
        const uint256_t max_lo_temp = max_r0_inner + (max_r1_inner << NUM_LIMB_BITS);
        const uint256_t max_hi_temp = max_r2_inner + (max_r3_inner << NUM_LIMB_BITS);
        return std::pair<uint256_t, uint256_t>(max_lo_temp, max_hi_temp);
    };

    /**
     * Step 1: Compute the maximum potential value of our product limbs
     *
     * max_lo = maximum value of limb products that span the range 0 - 2^{3t}
     * max_hi = maximum value of limb products that span the range 2^{2t} - 2^{5t}
     * (t = NUM_LIMB_BITS)
     **/
    uint256_t max_lo = 0;
    uint256_t max_hi = 0;

    // Compute max values of quotient product limb products
    uint256_t max_b0 = (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_b1 = (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_c0 = (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_c1 = (neg_modulus_limbs_u256[2] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_c2 = (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[2].maximum_value);
    uint256_t max_d0 = (neg_modulus_limbs_u256[3] * quotient.binary_basis_limbs[0].maximum_value);
    uint256_t max_d1 = (neg_modulus_limbs_u256[2] * quotient.binary_basis_limbs[1].maximum_value);
    uint256_t max_d2 = (neg_modulus_limbs_u256[1] * quotient.binary_basis_limbs[2].maximum_value);
    uint256_t max_d3 = (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[3].maximum_value);

    // max_r0 = terms from 0 - 2^2t
    // max_r1 = terms from 2^t - 2^3t
    // max_r2 = terms from 2^2t - 2^4t
    // max_r3 = terms from 2^3t - 2^5t
    uint256_t max_r0 = (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[0].maximum_value);
    max_r0 += (neg_modulus_limbs_u256[0] * quotient.binary_basis_limbs[0].maximum_value);
    const uint256_t max_r1 = max_b0 + max_b1;
    const uint256_t max_r2 = max_c0 + max_c1 + max_c2;
    const uint256_t max_r3 = max_d0 + max_d1 + max_d2 + max_d3;

    // update max_lo, max_hi with quotient limb product terms.
    max_lo += max_r0 + (max_r1 << NUM_LIMB_BITS);
    max_hi += max_r2 + (max_r3 << NUM_LIMB_BITS);

    // Compute maximum value of addition terms in `to_add` and add to max_lo, max_hi
    uint256_t max_a0(0);
    uint256_t max_a1(0);
    for (size_t i = 0; i < to_add.size(); ++i) {
        max_a0 += to_add[i].binary_basis_limbs[0].maximum_value +
                  (to_add[i].binary_basis_limbs[1].maximum_value << NUM_LIMB_BITS);
        max_a1 += to_add[i].binary_basis_limbs[2].maximum_value +
                  (to_add[i].binary_basis_limbs[3].maximum_value << NUM_LIMB_BITS);
    }
    max_lo += max_a0;
    max_hi += max_a1;

    // Compute the maximum value of our multiplication products and add to max_lo, max_hi
    for (size_t i = 0; i < num_multiplications; ++i) {
        const auto [product_lo, product_hi] = get_product_maximum(left[i], right[i]);
        max_lo += product_lo;
        max_hi += product_hi;
    }

    // Compute the maximum number of bits in `max_lo` and `max_hi` - this defines the range constraint values we will
    // need to apply to validate our product
    uint64_t max_lo_bits = (max_lo.get_msb() + 1);
    uint64_t max_hi_bits = max_hi.get_msb() + 1;
    // TurboPlonk range checks only work for even bit ranges, so make sure these values are even
    // TODO: This neccessary anymore? TurboPlonk range checks now work with odd bit ranges...
    if ((max_lo_bits & 1ULL) == 1ULL) {
        ++max_lo_bits;
    }
    if ((max_hi_bits & 1ULL) == 1ULL) {
        ++max_hi_bits;
    }

    // Compute the limb products for (left * right + quotient * (-p mod modulus_u512))
    field_t b0 = left[0].binary_basis_limbs[1].element.madd(
        right[0].binary_basis_limbs[0].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[0]);
    field_t b1 = left[0].binary_basis_limbs[0].element.madd(
        right[0].binary_basis_limbs[1].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[1]);
    field_t c0 = left[0].binary_basis_limbs[1].element.madd(
        right[0].binary_basis_limbs[1].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[1]);
    field_t c1 = left[0].binary_basis_limbs[2].element.madd(
        right[0].binary_basis_limbs[0].element, quotient.binary_basis_limbs[2].element * neg_modulus_limbs[0]);
    field_t c2 = left[0].binary_basis_limbs[0].element.madd(
        right[0].binary_basis_limbs[2].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[2]);
    field_t d0 = left[0].binary_basis_limbs[3].element.madd(
        right[0].binary_basis_limbs[0].element, quotient.binary_basis_limbs[3].element * neg_modulus_limbs[0]);
    field_t d1 = left[0].binary_basis_limbs[2].element.madd(
        right[0].binary_basis_limbs[1].element, quotient.binary_basis_limbs[2].element * neg_modulus_limbs[1]);
    field_t d2 = left[0].binary_basis_limbs[1].element.madd(
        right[0].binary_basis_limbs[2].element, quotient.binary_basis_limbs[1].element * neg_modulus_limbs[2]);
    field_t d3 = left[0].binary_basis_limbs[0].element.madd(
        right[0].binary_basis_limbs[3].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[3]);

    // For remaining multiplications, check to see if we already have a cached product computed.
    // If not compute it and add to the cache
    for (size_t i = 1; i < num_multiplications; ++i) {
        if (!caches[i - 1].cache_exists) {
            field_t lo_2 = left[i].binary_basis_limbs[0].element * right[i].binary_basis_limbs[0].element;
            lo_2 = left[i].binary_basis_limbs[1].element.madd(right[i].binary_basis_limbs[0].element * shift_1, lo_2);
            lo_2 = left[i].binary_basis_limbs[0].element.madd(right[i].binary_basis_limbs[1].element * shift_1, lo_2);
            field_t hi_2 = left[i].binary_basis_limbs[1].element * right[i].binary_basis_limbs[1].element;
            hi_2 = left[i].binary_basis_limbs[2].element.madd(right[i].binary_basis_limbs[0].element, hi_2);
            hi_2 = left[i].binary_basis_limbs[0].element.madd(right[i].binary_basis_limbs[2].element, hi_2);
            hi_2 = left[i].binary_basis_limbs[3].element.madd(right[i].binary_basis_limbs[0].element * shift_1, hi_2);
            hi_2 = left[i].binary_basis_limbs[2].element.madd(right[i].binary_basis_limbs[1].element * shift_1, hi_2);
            hi_2 = left[i].binary_basis_limbs[1].element.madd(right[i].binary_basis_limbs[2].element * shift_1, hi_2);
            hi_2 = left[i].binary_basis_limbs[0].element.madd(right[i].binary_basis_limbs[3].element * shift_1, hi_2);

            caches[i - 1].lo_cache = lo_2;
            caches[i - 1].hi_cache = hi_2;
            caches[i - 1].prime_cache = left[i].prime_basis_limb * right[i].prime_basis_limb;
            caches[i - 1].cache_exists = true;
        }
    }

    /**
     * Compute "limb accumulators"
     * `limb_0_accumulator` contains contributions in the range 0 - 2^{3t}
     * `limb_2_accumulator` contains contributiosn in the range 2^{2t} - 2^{5t} (t = MAX_NUM_LIMB_BITS)
     * Actual range will vary a few bits because of lazy reduction techniques
     *
     * We store these vaues in an "accumuator" vector in order to efficiently add them into a sum.
     * i.e. limb_0 =- field_t::accumulate(limb_0_accumulator)
     * This costs us fewer gates than addition operations because we can add 2 values into a sum in a single TurboPlonk
     *gate.
     **/
    std::vector<field_t<C>> limb_0_accumulator;
    std::vector<field_t<C>> limb_2_accumulator;
    std::vector<field_t<C>> prime_limb_accumulator;

    // add cached products into the limb accumulators.
    // We negate the cache values because the accumulator values itself will be negated
    // TODO: why do we do this double negation exactly? seems a bit pointless. I think it stems from the fact that the
    // accumulators originaly tracked the remainder term (which is negated)
    for (size_t i = 1; i < num_multiplications; ++i) {
        limb_0_accumulator.emplace_back(-caches[i - 1].lo_cache);
        limb_2_accumulator.emplace_back(-caches[i - 1].hi_cache);
        prime_limb_accumulator.emplace_back(-caches[i - 1].prime_cache);
    }

    // Update the accumulators with the remainder terms. First check we actually have remainder terms!
    //(not present when we're checking a product is 0 mod p). See `assert_is_in_field`
    bool no_remainders = remainders.size() == 0;
    if (!no_remainders) {
        limb_0_accumulator.emplace_back(remainders[0].binary_basis_limbs[0].element);
        limb_2_accumulator.emplace_back(remainders[0].binary_basis_limbs[2].element);
        prime_limb_accumulator.emplace_back(remainders[0].prime_basis_limb);
    }
    for (size_t i = 1; i < remainders.size(); ++i) {
        limb_0_accumulator.emplace_back(remainders[i].binary_basis_limbs[0].element);
        limb_0_accumulator.emplace_back(remainders[i].binary_basis_limbs[1].element * shift_1);
        limb_2_accumulator.emplace_back(remainders[i].binary_basis_limbs[2].element);
        limb_2_accumulator.emplace_back(remainders[i].binary_basis_limbs[3].element * shift_1);
        prime_limb_accumulator.emplace_back(remainders[i].prime_basis_limb);
    }
    // Update limb accumulators with linear addition terms
    for (const auto& add : to_add) {
        limb_0_accumulator.emplace_back(-add.binary_basis_limbs[0].element);
        limb_0_accumulator.emplace_back(-add.binary_basis_limbs[1].element * shift_1);
        limb_2_accumulator.emplace_back(-add.binary_basis_limbs[2].element);
        limb_2_accumulator.emplace_back(-add.binary_basis_limbs[3].element * shift_1);
        prime_limb_accumulator.emplace_back(-add.prime_basis_limb);
    }

    // Accumulate the accumulators! At this point we know that `accumulated_lo` and `accumulated_hi` have not
    // overflowed, as the terms inside `limb_0_accumulator/ilmb_2_accumulator` are roughtly in the range 0 - 2^{3t}
    // which is << native modulus
    field_t<C> accumulated_lo = field_t<C>::accumulate(limb_0_accumulator);
    field_t<C> accumulated_hi = field_t<C>::accumulate(limb_2_accumulator);

    // If our accumulators are constant, instantiate them as witnesses.
    // TODO: why do we need this? We should be able to handle these values as constants. Needs investigation
    if (accumulated_lo.is_constant()) {
        accumulated_lo = field_t<C>::from_witness_index(ctx, ctx->put_constant_variable(accumulated_lo.get_value()));
    }
    if (accumulated_hi.is_constant()) {
        accumulated_hi = field_t<C>::from_witness_index(ctx, ctx->put_constant_variable(accumulated_hi.get_value()));
    }

    // Compute our 4 remainder limbs. Add our accumuated_lo and accumulated_hi values into the remainder term
    field_t<C> remainder1 = no_remainders ? field_t<C>::from_witness_index(ctx, ctx->zero_idx)
                                          : remainders[0].binary_basis_limbs[1].element;
    if (remainder1.is_constant()) {
        remainder1 = field_t<C>::from_witness_index(ctx, ctx->put_constant_variable(remainder1.get_value()));
    }
    field_t<C> remainder3 = no_remainders ? field_t<C>::from_witness_index(ctx, ctx->zero_idx)
                                          : remainders[0].binary_basis_limbs[3].element;
    if (remainder3.is_constant()) {
        remainder3 = field_t<C>::from_witness_index(ctx, ctx->put_constant_variable(remainder3.get_value()));
    }
    field_t<C> remainder_limbs[4]{
        accumulated_lo,
        remainder1,
        accumulated_hi,
        remainder3,
    };
    field_t<C> remainder_prime_limb = field_t<C>::accumulate(prime_limb_accumulator);

    // The following code should be identical to the `evaluate_multiply_add` method.]

    // We wish to show that left*right - quotient*remainder = 0 mod 2^t, we do this by collecting the limb products
    // into two separate variables - carry_lo and carry_hi, which are still small enough not to wrap mod r
    // Their first t/2 bits will equal, respectively, the first and second t/2 bits of the expresssion
    // Thus it will suffice to check that each of them begins with t/2 zeroes. We do this by in fact assigning
    // to these variables those expressions divided by 2^{t/2}. Since we have bounds on their ranage that are
    // smaller than r, We can range check the divisions by the original range bounds divided by 2^{t/2}
    field_t r0 = left[0].binary_basis_limbs[0].element.madd(
        right[0].binary_basis_limbs[0].element, quotient.binary_basis_limbs[0].element * neg_modulus_limbs[0]);
    field_t r1 = b0.add_two(b1, -remainder_limbs[1]);
    const field_t r2 = c0.add_two(c1, c2);
    const field_t r3 = d0 + d1.add_two(d2, d3);

    field_t carry_lo_0 = r0 * shift_right_2;
    field_t carry_lo_1 = r1 * (shift_1 * shift_right_2);
    field_t carry_lo_2 = -(remainder_limbs[0] * shift_right_2);
    field_t carry_lo = carry_lo_0.add_two(carry_lo_1, carry_lo_2);

    field_t t1 = carry_lo.add_two(-remainder_limbs[2], -(remainder_limbs[3] * shift_1));
    field_t carry_hi_0 = r2 * shift_right_2;
    field_t carry_hi_1 = r3 * (shift_1 * shift_right_2);
    field_t carry_hi_2 = t1 * shift_right_2;
    field_t carry_hi = carry_hi_0.add_two(carry_hi_1, carry_hi_2);

    barretenberg::fr neg_prime = -barretenberg::fr(uint256_t(target_basis.modulus));

    field_t<C> linear_terms(ctx, barretenberg::fr(0));

    linear_terms += -remainder_prime_limb;

    // This is where we show our identity is zero mod r (to use Chinese Remainder Theorem we show it's zero mod r and
    // mod 2^t)
    field_t<C>::evaluate_polynomial_identity(
        left[0].prime_basis_limb, right[0].prime_basis_limb, quotient.prime_basis_limb * neg_prime, linear_terms);

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
        const auto accumulators = ctx->decompose_into_base4_accumulators(
            carry_combined.witness_index, static_cast<size_t>(carry_lo_msb + carry_hi_msb));
        field_t<C> accumulator_midpoint =
            field_t<C>::from_witness_index(ctx, accumulators[static_cast<size_t>((carry_hi_msb / 2) - 1)]);
        carry_hi.assert_equal(accumulator_midpoint, "bigfield multiply range check failed");
    }
}

/**
 * `evaluate_square_add`
 *
 * This is extremely similar to `evaluate_multiply_add`,
 * but we need fewer gates to compute the limb products of (a * a) vs (a * b)
 * TODO: reduce code duplication! Most of this code is re-used from evaluate_multiply_add
 **/
template <typename C, typename T>
void bigfield<C, T>::evaluate_square_add(const bigfield& left,
                                         const std::vector<bigfield>& to_add,
                                         const bigfield& quotient,
                                         const bigfield& remainder)
{
    if (C::type == waffle::PLOOKUP) {
        evaluate_multiply_add(left, left, to_add, quotient, { remainder });
        return;
    }
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
        const auto accumulators = ctx->decompose_into_base4_accumulators(
            carry_combined.witness_index, static_cast<size_t>(carry_lo_msb + carry_hi_msb));
        field_t accumulator_midpoint =
            field_t<C>::from_witness_index(ctx, accumulators[static_cast<size_t>((carry_hi_msb / 2) - 1)]);
        carry_hi.assert_equal(accumulator_midpoint, "bigfield multiply range check failed");
    }
}

} // namespace stdlib
} // namespace plonk