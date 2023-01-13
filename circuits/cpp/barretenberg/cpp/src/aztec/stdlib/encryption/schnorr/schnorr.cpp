#include "schnorr.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>

#include "../../primitives/composers/composers.hpp"

namespace plonk {
namespace stdlib {
namespace schnorr {

/**
 * @brief Expand a 128-bits integer in a form amenable to doing elliptic curve arithmetic in circuits.
 *
 * @details The output wnaf_record records the expansion coefficients
 *   limb % 129 = 2^128 + 2^127 w_1 + ... + 2 w_127 + w_128 - skew
 * where each w_i lies in {-1, 1} and skew is 0 or 1. The boolean `skew` could also be called `is_even`; the even
 * 129-bit non-negative integers are those with skew == 1, while the odd ones have skew==0.
 *
 * @warning While it is possible to express any 129-bit value in this form, this function only works correctly
 * on 128-bit values, since the same is true for fixed_wnaf<129, 1, 1>. This is illusrated in the tests.
 *
 *
 * TurboPLONK: ~260 gates.
 */
template <typename C> wnaf_record<C> convert_field_into_wnaf(C* context, const field_t<C>& limb)
{
    constexpr size_t num_wnaf_bits = 129;
    uint256_t value = limb.get_value();

    bool skew = false;
    uint64_t wnaf_entries[129] = { 0 };

    // compute wnaf representation of value natively
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 1>(&value.data[0], &wnaf_entries[0], skew, 0);

    std::vector<bool_t<C>> wnaf_bits;
    bool_t<C> wnaf_skew(witness_t<C>(context, skew));
    field_t<C> two(context, 2);
    field_t<C> one(context, 1);
    field_t<C> accumulator(context, 1);

    // set accumulator = 2^{128} + \sum_{i=0}^{127} 2^i w_{128-i}, where w_i = 2 * wnaf_entries[i+1] - 1
    for (size_t i = 0; i < 128; ++i) {
        // accumulator = 2 * accumulator + 1 (resp. -1) if the 32nd bit of wnaf_entries[i+1] is 0 (resp. 1).

        // extract sign bit of wnaf_entries[i+1] (32nd entry in list of bits)
        uint64_t predicate = (wnaf_entries[i + 1] >> 31U) & 1U;
        // type of !predicate below is bool
        bool_t<C> wnaf_bit = witness_t<C>(context, !predicate);
        wnaf_bits.push_back(wnaf_bit);

        // !predicate == false ~> -1; true ~> +1
        accumulator = accumulator + accumulator;
        accumulator = accumulator + (field_t<C>(wnaf_bit) * two - one);
    }

    // subtract 1 from accumulator if there is skew
    accumulator = accumulator - field_t<C>(wnaf_skew);

    accumulator.assert_equal(limb);
    wnaf_record<C> result;
    result.bits = wnaf_bits;
    result.skew = wnaf_skew;
    return result;
}

/**
 * @brief Instantiate a witness containing the signature (s, e) as a quadruple of
 * field_t elements (s_lo, s_hi, e_lo, e_hi).
 */
template <typename C> signature_bits<C> convert_signature(C* context, const crypto::schnorr::signature& signature)
{
    signature_bits<C> sig{
        field_t<C>(),
        field_t<C>(),
        field_t<C>(),
        field_t<C>(),
    };

    uint256_t s_bigint(0);
    uint256_t e_bigint(0);

    for (size_t i = 0; i < 32; ++i) {
        for (size_t j = 7; j < 8; --j) {
            uint8_t s_shift = static_cast<uint8_t>(signature.s[i] >> j);
            uint8_t e_shift = static_cast<uint8_t>(signature.e[i] >> j);
            bool s_bit = (s_shift & 1U) == 1U;
            bool e_bit = (e_shift & 1U) == 1U;
            s_bigint += s_bigint;
            e_bigint += e_bigint;

            s_bigint += static_cast<uint64_t>(s_bit);
            e_bigint += static_cast<uint64_t>(e_bit);
        }
    }

    sig.s_lo = witness_t<C>(context, s_bigint.slice(0, 128));
    sig.s_hi = witness_t<C>(context, s_bigint.slice(128, 256));
    sig.e_lo = witness_t<C>(context, e_bigint.slice(0, 128));
    sig.e_hi = witness_t<C>(context, e_bigint.slice(128, 256));

    return sig;
}

/**
 * @brief Compute [(low_bits + 2^128 high_bits)]pub_key.
 *
 * @details This method cannot handle the case where either of low_bits, high_bits is zero.
 * This assumption is backed by a constraint (see the tests for an illustration).
 */
template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const field_t<C>& low_bits, const field_t<C>& high_bits)
{
    C* context = pub_key.x.context;

    // N.B. this method does not currently work if low_bits == 0 or high_bits == 0
    field_t<C> zero_test = (low_bits * high_bits);
    zero_test.assert_is_not_zero();

    const auto low_wnaf = plonk::stdlib::schnorr::convert_field_into_wnaf(context, low_bits);
    const auto high_wnaf = plonk::stdlib::schnorr::convert_field_into_wnaf(context, high_bits);
    // current_accumulator is pub_key, so init is true, so high_output is [high_wnaf]pub_key
    point<C> high_output = plonk::stdlib::schnorr::variable_base_mul(pub_key, pub_key, high_wnaf);
    // compute output = [low_wnaf]pub_key + [2^128]high_output.
    point<C> output = plonk::stdlib::schnorr::variable_base_mul(pub_key, high_output, low_wnaf);
    return output;
}

/**
 * @brief Multiply a point of Grumpkin by a scalar described as a wnaf record, possibly offsetting by another point.
 *
 * @param pub_key A point of Grumpkin known to the prover in terms of the generator grumpkin::g1::one.
 * @param current_accumulator A point of the curve that will remain unchanged.
 * @param wnaf A wnaf_record<C>, a collection of bool_t<C>'s typically recording an expansion of an element of
 * field_t<C> in the form 2^{128} + 2^{127} w_1 + ... + 2 w_127 + w_128 - skew.
 *
 * @details Let W be the scalar represented by wnaf. If pub_key = ± current_accumulator, this function returns
 * [W]pub_key. Otherwise, it returns [W]pub_key + [2^128]current_accumulator. These two cases are distinguished
 * between a boolean `init`. The idea here is that, if `pub_key==±current_accumulator`, then the function is being
 * called for the first time.
 *
 * @warning This function should not be used on its own, as its security depends on the manner in which it is
 * expected to be used.
 */
template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const point<C>& current_accumulator, const wnaf_record<C>& wnaf)
{
    // Check if the pub_key is a points on the curve.
    pub_key.on_curve();

    // The account circuit constrains `pub_key` to lie on Grumpkin. Presently, the only values that are passed in the
    // second argument as `current_accumulator` are `pub_key` and a point which is the output of the present function.
    // We therefore assume that `current_accumulator` lies on Grumpkin as well.
    grumpkin::g1::affine_element pub_key_native(pub_key.x.get_value(), pub_key.y.get_value());
    grumpkin::g1::affine_element current_accumulator_native(current_accumulator.x.get_value(),
                                                            current_accumulator.y.get_value());

    field_t<C> two(pub_key.x.context, 2);

    // Various elliptic curve point additions that follow assume that the two points are distinct and not mutually
    // inverse. collision_offset is chosen to prevent a malicious prover from exploiting this assumption.
    grumpkin::g1::affine_element collision_offset = crypto::pedersen::get_generator_data(DEFAULT_GEN_1).generator;
    grumpkin::g1::affine_element collision_end = collision_offset * grumpkin::fr(uint256_t(1) << 129);

    const bool init = current_accumulator.x.get_value() == pub_key.x.get_value();

    // if init == true, check pub_key != collision_offset (ruling out 3 other points at the same time),
    // if init == false we assume this has already been checked in an earlier call wherein init==true.
    if (init) {
        field_t<C> zero_test = ((pub_key.x - collision_offset.x) * (pub_key.y - collision_offset.y));
        zero_test.assert_is_not_zero("pub_key and collision_offset have a coordinate in common.");
    } else {
        // Check if the current_accumulator is a point on the curve only if init is false.
        current_accumulator.on_curve();
    }

    point<C> accumulator{ collision_offset.x, collision_offset.y };

    /*
     * Let w_i = 2 wnaf.bits[i-1] - 1 for i = 1, ..., 128.
     * The integer represented by the digits w_i and a skew bit `skew` in {0, 1} is
     *   W := 2^{128} + 2^{127} w_1 + ... + 2 w_127 + w_128 - skew
     *      = 2^{128} + \sum_{k=0}^{127}2^{k}w_{128-k}  - skew.
     * When init == true, the for loop that follows sets
     *         accumulator = [W+skew]pub_key + [2^{129}]collision_offset
     * When init == false, the for loop that follows sets
     *         accumulator = [W+skew]pub_key + [2^{129}]collision_offset + [2^{128}]current_accumulator.
     * We describe the accumulation process in the loop.
     *
     * Defining w_{-1} = 0, W_{0} = 1, and W_{i+1} = 2 W_{i} + w_i for i = 1, ..., 128, we have
     *    W_1   =                                   2 + w_0
     *    W_2   =                       4 +     2 w_0 + w_1
     *    W_i   = 2^i + 2^{i-1} w_0 + ... + 2 w_{i-2} + w_{i-1}
     *    W_128 = W + skew
     *
     * Let A_0 = collision_offset. For i = 0, ..., 127, let
     *   A_{i+1} = 2^{i+1} collision_offset + [W_{i}]pub_key and A'_{i+1} = A_{i+1} + [2^{i}]current_accumulator.
     * Suppose we are at the end of the loop with loop variable i.
     *   - If `init==true`, then the value of `accumulator` is A_{i+i}.
     *   - If `init==false`, then the value of `accumulator` is A'_{i+1}.
     * In both cases, setting the final accumulator value is that claimed above.
     *
     * Note that all divisons are safe, i.e., failing contsraints will be imposed if any denominator is zero.
     */
    for (size_t i = 0; i < 129; ++i) {
        if (!init && i == 1) {
            // set accumulator = accumulator + current_accumulator.
            field_t<C> x1 = accumulator.x;
            field_t<C> y1 = accumulator.y;

            field_t<C> x2 = current_accumulator.x;
            field_t<C> y2 = current_accumulator.y;

            field_t<C> lambda1 = (y2 - y1) / (x2 - x1);
            field_t<C> x3 = lambda1.madd(lambda1, -(x2 + x1));
            field_t<C> y3 = lambda1.madd((x1 - x3), -y1);
            accumulator.x = x3;
            accumulator.y = y3;
        }

        // if i == 0: set accumulator = [2]accumulator + pub_key
        // otherwise, set accumulator = [2]accumulator + [w_i]pub_key.

        // // Set P_3 = accumulator + pub_key or P_3 = accumulator - pub_key, depending on the current wnaf bit.

        field_t<C> x1 = accumulator.x;
        field_t<C> y1 = accumulator.y;

        field_t<C> x2 = (i == 0) ? pub_key.x : pub_key.x;
        field_t<C> y2 = (i == 0) ? pub_key.y : pub_key.y.madd(field_t<C>(wnaf.bits[i - 1]) * two, -pub_key.y);
        field_t<C> lambda1 = (y2 - y1) / (x2 - x1);
        field_t<C> x3 = lambda1.madd(lambda1, -(x2 + x1));

        // // Set P_4 = P_3 + accumulator.
        // // We save gates by not using the formula lambda2 = (y3 - y1) / (x3 - x1), which would require computing
        // // y_3. Instead we use another formula for lambda2 derived using the substitution y3 = lambda1(x1 - x3) - y1.
        field_t<C> lambda2 = -lambda1 - (y1 * two) / (x3 - x1);
        field_t<C> x4 = lambda2.madd(lambda2, -(x3 + x1));
        field_t<C> y4 = lambda2.madd(x1 - x4, -y1);

        accumulator.x = x4;
        accumulator.y = y4;
    }

    // At this point, accumulator is [W + skew]pub + [2^{129}]collision_mask.
    // If wnaf_skew, subtract pub_key frorm accumulator.
    field_t<C> add_lambda = (accumulator.y + pub_key.y) / (accumulator.x - pub_key.x);
    field_t<C> x_add = add_lambda.madd(add_lambda, -(accumulator.x + pub_key.x));
    field_t<C> y_add = add_lambda.madd((pub_key.x - x_add), pub_key.y);
    bool_t<C> add_predicate = wnaf.skew;
    accumulator.x = ((x_add - accumulator.x).madd(field_t<C>(add_predicate), accumulator.x));
    accumulator.y = ((y_add - accumulator.y).madd(field_t<C>(add_predicate), accumulator.y));

    // subtract [2^{129}]collision_offset from accumulator.
    point<C> collision_mask{ collision_end.x, -collision_end.y };

    field_t<C> lambda = (accumulator.y - collision_mask.y) / (accumulator.x - collision_mask.x);
    field_t<C> x3 = lambda.madd(lambda, -(collision_mask.x + accumulator.x));
    field_t<C> y3 = lambda.madd(collision_mask.x - x3, -collision_mask.y);

    accumulator.x = x3;
    accumulator.y = y3;
    return accumulator;
}

/**
 * @brief Verify a signature (s, e)  i.e., compute e' = hash(([s]g + [e]pub).x | message) and check that e' == e.
 *
 * @details TurboPlonk: ~10850 gates (~4k for variable_base_mul, ~6k for blake2s) for a string of length < 32.
 */
template <typename C>
void verify_signature(const byte_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig)
{
    // Compute [s]g, where s = (s_lo, s_hi) and g = G1::one.
    point<C> R_1 = group<C>::fixed_base_scalar_mul(sig.s_lo, sig.s_hi);
    // Compute [e]pub, where e = (e_lo, e_hi)
    point<C> R_2 = variable_base_mul(pub_key, sig.e_lo, sig.e_hi);

    // check R_1 != R_2
    (R_1.x - R_2.x).assert_is_not_zero("Cannot add points in Schnorr verification.");
    // Compute x-coord of R_1 + R_2 = [s]g + [e]pub.
    field_t<C> lambda = (R_1.y - R_2.y) / (R_1.x - R_2.x);
    field_t<C> x_3 = lambda * lambda - (R_1.x + R_2.x);

    // build input (pedersen(([s]g + [e]pub).x | pub.x | pub.y) | message) to hash function
    // pedersen hash ([r].x | pub.x) to make sure the size of `hash_input` is <= 64 bytes for a 32 byte message
    byte_array<C> hash_input(stdlib::pedersen<C>::compress({ x_3, pub_key.x, pub_key.y }));
    hash_input.write(message);

    // compute  e' = hash(([s]g + [e]pub).x | message)
    byte_array<C> output = blake2s(hash_input);

    // verify that e' == e
    field_t<C> output_hi(output.slice(0, 16));
    field_t<C> output_lo(output.slice(16, 16));
    output_lo.assert_equal(sig.e_lo, "verify signature failed");
    output_hi.assert_equal(sig.e_hi, "verify signature failed");
}

template wnaf_record<waffle::TurboComposer> convert_field_into_wnaf<waffle::TurboComposer>(
    waffle::TurboComposer* context, const field_t<waffle::TurboComposer>& limb);

template wnaf_record<waffle::UltraComposer> convert_field_into_wnaf<waffle::UltraComposer>(
    waffle::UltraComposer* context, const field_t<waffle::UltraComposer>& limb);

template point<waffle::TurboComposer> variable_base_mul(const point<waffle::TurboComposer>& pub_key,
                                                        const field_t<waffle::TurboComposer>& low_bits,
                                                        const field_t<waffle::TurboComposer>& high_bits);

template point<waffle::TurboComposer> variable_base_mul<waffle::TurboComposer>(
    const point<waffle::TurboComposer>&,
    const point<waffle::TurboComposer>&,
    const wnaf_record<waffle::TurboComposer>&);

template void verify_signature<waffle::TurboComposer>(const byte_array<waffle::TurboComposer>&,
                                                      const point<waffle::TurboComposer>&,
                                                      const signature_bits<waffle::TurboComposer>&);
template void verify_signature<waffle::UltraComposer>(const byte_array<waffle::UltraComposer>&,
                                                      const point<waffle::UltraComposer>&,
                                                      const signature_bits<waffle::UltraComposer>&);

template signature_bits<waffle::TurboComposer> convert_signature<waffle::TurboComposer>(
    waffle::TurboComposer*, const crypto::schnorr::signature&);
template signature_bits<waffle::UltraComposer> convert_signature<waffle::UltraComposer>(
    waffle::UltraComposer*, const crypto::schnorr::signature&);
} // namespace schnorr
} // namespace stdlib
} // namespace plonk
