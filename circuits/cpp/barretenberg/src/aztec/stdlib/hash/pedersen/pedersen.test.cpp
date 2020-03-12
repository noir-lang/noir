#include <gtest/gtest.h>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include "pedersen.hpp"

namespace test_stdlib_pedersen {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

TEST(stdlib_pedersen, test_pedersen)
{

    waffle::TurboComposer composer = waffle::TurboComposer();

    fr left_in = fr::random_element();
    fr right_in = fr::random_element();
    // ensure left has skew 1, right has skew 0
    if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
        left_in += fr::one();
    }
    if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
        right_in += fr::one();
    }
    field_t left = public_witness_t(&composer, left_in);
    field_t right = witness_t(&composer, right_in);

    composer.fix_witness(left.witness_index, left.get_value());
    composer.fix_witness(right.witness_index, right.get_value());

    field_t out = plonk::stdlib::pedersen::compress(left, right);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);

    bool left_skew = false;
    bool right_skew = false;

    uint64_t left_wnafs[255] = { 0 };
    uint64_t right_wnafs[255] = { 0 };

    if ((left_in.from_montgomery_form().data[0] & 1) == 0) {
        fr two = fr::one() + fr::one();
        left_in = left_in - two;
    }
    if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
        fr two = fr::one() + fr::one();
        right_in = right_in - two;
    }
    fr converted_left = left_in.from_montgomery_form();
    fr converted_right = right_in.from_montgomery_form();

    uint64_t* left_scalar = &(converted_left.data[0]);
    uint64_t* right_scalar = &(converted_right.data[0]);

    barretenberg::wnaf::fixed_wnaf<255, 1, 2>(left_scalar, &left_wnafs[0], left_skew, 0);
    barretenberg::wnaf::fixed_wnaf<255, 1, 2>(right_scalar, &right_wnafs[0], right_skew, 0);

    const auto compute_split_scalar = [](uint64_t* wnafs, const size_t range) {
        grumpkin::fr result = grumpkin::fr::zero();
        grumpkin::fr three = grumpkin::fr{ 3, 0, 0, 0 }.to_montgomery_form();
        for (size_t i = 0; i < range; ++i) {
            uint64_t entry = wnafs[i];
            grumpkin::fr prev = result + result;
            prev = prev + prev;
            if ((entry & 0xffffff) == 0) {
                if (((entry >> 31UL) & 1UL) == 1UL) {
                    result = prev - grumpkin::fr::one();
                } else {
                    result = prev + grumpkin::fr::one();
                }
            } else {
                if (((entry >> 31UL) & 1UL) == 1UL) {
                    result = prev - three;
                } else {
                    result = prev + three;
                }
            }
        }
        return result;
    };

    grumpkin::fr grumpkin_scalars[4]{ compute_split_scalar(&left_wnafs[0], 126),
                                      compute_split_scalar(&left_wnafs[126], 2),
                                      compute_split_scalar(&right_wnafs[0], 126),
                                      compute_split_scalar(&right_wnafs[126], 2) };
    if (left_skew) {
        grumpkin_scalars[1] += grumpkin::fr::one();
    }
    if (right_skew) {
        grumpkin_scalars[3] += grumpkin::fr::one();
    }

    grumpkin::g1::affine_element grumpkin_points[4]{
        crypto::pedersen::get_generator(0),
        crypto::pedersen::get_generator(1),
        crypto::pedersen::get_generator(2),
        crypto::pedersen::get_generator(3),
    };

    grumpkin::g1::element result_points[4]{
        grumpkin_points[0] * grumpkin_scalars[0],
        grumpkin_points[1] * grumpkin_scalars[1],
        grumpkin_points[2] * grumpkin_scalars[2],
        grumpkin_points[3] * grumpkin_scalars[3],
    };

    grumpkin::g1::element hash_output_left;
    grumpkin::g1::element hash_output_right;

    hash_output_left = result_points[0] + result_points[1];
    hash_output_right = result_points[2] + result_points[3];

    grumpkin::g1::element hash_output;
    hash_output = hash_output_left + hash_output_right;
    hash_output = hash_output.normalize();

    EXPECT_EQ((out.get_value() == hash_output.x), true);

    fr compress_native = crypto::pedersen::compress_native(left.get_value(), right.get_value());
    EXPECT_EQ((out.get_value() == compress_native), true);
}

TEST(stdlib_pedersen, test_pedersen_large)
{

    waffle::TurboComposer composer = waffle::TurboComposer();

    fr left_in = fr::random_element();
    fr right_in = fr::random_element();
    // ensure left has skew 1, right has skew 0
    if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
        left_in += fr::one();
    }
    if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
        right_in += fr::one();
    }
    field_t left = witness_t(&composer, left_in);
    field_t right = witness_t(&composer, right_in);

    for (size_t i = 0; i < 256; ++i) {
        left = plonk::stdlib::pedersen::compress(left, right);
    }

    composer.set_public_input(left.witness_index);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

} // namespace test_stdlib_pedersen