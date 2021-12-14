#include "schnorr.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>

#include "../../primitives/composers/composers.hpp"

namespace plonk {
namespace stdlib {
namespace schnorr {

template <typename C> wnaf_record<C> convert_field_into_wnaf(C* context, const field_t<C>& limb)
{
    constexpr size_t num_wnaf_bits = 129;
    uint256_t value = limb.get_value();

    bool skew = false;
    uint64_t wnaf_entries[129] = { 0 };
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 1>(&value.data[0], &wnaf_entries[0], skew, 0);

    std::vector<bool_t<C>> wnaf_bits;
    bool_t<C> wnaf_skew(witness_t<C>(context, skew));
    field_t<C> two(context, 2);
    field_t<C> one(context, 1);
    field_t<C> accumulator(context, 1);

    for (size_t i = 0; i < 128; ++i) {
        uint64_t predicate = (wnaf_entries[i + 1] >> 31U) & 1U;
        bool_t<C> wnaf_bit = witness_t<C>(context, !predicate); // false = -1, true = +1
        wnaf_bits.push_back(wnaf_bit);

        accumulator = accumulator + accumulator;
        accumulator = accumulator + (field_t<C>(wnaf_bit) * two - one);
    }
    accumulator = accumulator - field_t<C>(wnaf_skew);
    accumulator.assert_equal(limb);
    wnaf_record<C> result;
    result.bits = wnaf_bits;
    result.skew = wnaf_skew;
    return result;
}

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

template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const field_t<C>& low_bits, const field_t<C>& high_bits)
{
    C* context = pub_key.x.context;

    // N.B. this method does not currently work if low_bits == 0 or high_bits == 0
    field_t<C> zero_test = (low_bits * high_bits);
    zero_test.assert_is_not_zero();

    const auto low_wnaf = plonk::stdlib::schnorr::convert_field_into_wnaf(context, low_bits);
    const auto high_wnaf = plonk::stdlib::schnorr::convert_field_into_wnaf(context, high_bits);
    point<C> high_output = plonk::stdlib::schnorr::variable_base_mul(pub_key, pub_key, high_wnaf);
    point<C> output = plonk::stdlib::schnorr::variable_base_mul(pub_key, high_output, low_wnaf);
    return output;
}

template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const point<C>& current_accumulator, const wnaf_record<C>& wnaf)
{
    field_t<C> two(pub_key.x.context, 2);

    grumpkin::g1::affine_element collision_offset = crypto::pedersen::get_generator_data(DEFAULT_GEN_1).generator;
    grumpkin::g1::affine_element collision_end = collision_offset * grumpkin::fr(uint256_t(1) << 129);

    const bool init = current_accumulator.x.get_value() == pub_key.x.get_value();

    // if init == false, check pub_key != collision_offset
    // if init == true we assume this has already been checked
    if (init) {
        field_t<C> zero_test = ((pub_key.x - collision_offset.x) * (pub_key.y - collision_offset.y));
        zero_test.assert_is_not_zero();
    }
    point<C> accumulator{ collision_offset.x, collision_offset.y };
    for (size_t i = 0; i < 129; ++i) {
        if (!init && i == 1) {
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
        field_t<C> x1 = accumulator.x;
        field_t<C> y1 = accumulator.y;

        field_t<C> x2 = (i == 0) ? pub_key.x : pub_key.x;
        field_t<C> y2 = (i == 0) ? pub_key.y : pub_key.y.madd(field_t<C>(wnaf.bits[i - 1]) * two, -pub_key.y);

        field_t<C> lambda1 = (y2 - y1) / (x2 - x1);
        field_t<C> x3 = lambda1.madd(lambda1, -(x2 + x1));

        field_t<C> lambda2 = -lambda1 - (y1 * two) / (x3 - x1);

        field_t<C> x4 = lambda2.madd(lambda2, -(x3 + x1));
        field_t<C> y4 = lambda2.madd(x1 - x4, -y1);

        accumulator.x = x4;
        accumulator.y = y4;
    }

    field_t<C> add_lambda = (accumulator.y + pub_key.y) / (accumulator.x - pub_key.x);
    field_t<C> x_add = add_lambda.madd(add_lambda, -(accumulator.x + pub_key.x));
    field_t<C> y_add = add_lambda.madd((pub_key.x - x_add), pub_key.y);
    bool_t<C> add_predicate = wnaf.skew;
    accumulator.x = ((x_add - accumulator.x).madd(field_t<C>(add_predicate), accumulator.x));
    accumulator.y = ((y_add - accumulator.y).madd(field_t<C>(add_predicate), accumulator.y));

    point<C> collision_mask{ collision_end.x, -collision_end.y };

    field_t<C> lambda = (accumulator.y - collision_mask.y) / (accumulator.x - collision_mask.x);
    field_t<C> x3 = lambda.madd(lambda, -(collision_mask.x + accumulator.x));
    field_t<C> y3 = lambda.madd(collision_mask.x - x3, -collision_mask.y);

    accumulator.x = x3;
    accumulator.y = y3;
    return accumulator;
}

template <typename C>
bool verify_signature(const byte_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig)
{
    point<C> R_1 = group<C>::fixed_base_scalar_mul(sig.s_lo, sig.s_hi);
    point<C> R_2 = variable_base_mul(pub_key, sig.e_lo, sig.e_hi);

    // check R_1 != R_2
    (R_1.x - R_2.x).assert_is_not_zero();
    field_t<C> lambda = (R_1.y - R_2.y) / (R_1.x - R_2.x);
    field_t<C> x_3 = lambda * lambda - (R_1.x + R_2.x);

    byte_array<C> hash_input(x_3);
    hash_input.write(message);
    byte_array<C> output = blake2s(hash_input);
    field_t<C> output_hi(output.slice(0, 16));
    field_t<C> output_lo(output.slice(16, 16));
    output_lo.assert_equal(sig.e_lo, "verify signature failed");
    output_hi.assert_equal(sig.e_hi, "verify signature failed");
    bool valid = (output_lo.get_value() == sig.e_lo.get_value());
    valid = valid && (output_hi.get_value() == sig.e_hi.get_value());
    return valid;
}

template wnaf_record<waffle::TurboComposer> convert_field_into_wnaf<waffle::TurboComposer>(
    waffle::TurboComposer* context, const field_t<waffle::TurboComposer>& limb);

template point<waffle::TurboComposer> variable_base_mul(const point<waffle::TurboComposer>& pub_key,
                                                        const field_t<waffle::TurboComposer>& low_bits,
                                                        const field_t<waffle::TurboComposer>& high_bits);

template point<waffle::TurboComposer> variable_base_mul<waffle::TurboComposer>(
    const point<waffle::TurboComposer>&,
    const point<waffle::TurboComposer>&,
    const wnaf_record<waffle::TurboComposer>&);

template bool verify_signature<waffle::TurboComposer>(const byte_array<waffle::TurboComposer>&,
                                                      const point<waffle::TurboComposer>&,
                                                      const signature_bits<waffle::TurboComposer>&);
template bool verify_signature<waffle::PlookupComposer>(const byte_array<waffle::PlookupComposer>&,
                                                        const point<waffle::PlookupComposer>&,
                                                        const signature_bits<waffle::PlookupComposer>&);

template signature_bits<waffle::TurboComposer> convert_signature<waffle::TurboComposer>(
    waffle::TurboComposer*, const crypto::schnorr::signature&);
template signature_bits<waffle::PlookupComposer> convert_signature<waffle::PlookupComposer>(
    waffle::PlookupComposer*, const crypto::schnorr::signature&);
} // namespace schnorr
} // namespace stdlib
} // namespace plonk