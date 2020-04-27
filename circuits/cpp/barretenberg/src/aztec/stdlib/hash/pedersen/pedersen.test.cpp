#include "pedersen.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <numeric/random/engine.hpp>
#include <common/test.hpp>

namespace test_stdlib_pedersen {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::byte_array<waffle::TurboComposer> byte_array;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

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

    EXPECT_EQ(out.get_value(), hash_output.x);

    fr compress_native = crypto::pedersen::compress_native(left.get_value(), right.get_value());
    EXPECT_EQ(out.get_value(), compress_native);
}

HEAVY_TEST(stdlib_pedersen, test_pedersen_large)
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

HEAVY_TEST(stdlib_pedersen, test_pedersen_large_unrolled)
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

    waffle::UnrolledTurboProver prover = composer.create_unrolled_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::UnrolledTurboVerifier verifier = composer.create_unrolled_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_pedersen, test_compress_byte_array)
{
    const size_t num_input_bytes = 351;

    waffle::TurboComposer composer = waffle::TurboComposer();

    std::vector<uint8_t> input;
    input.reserve(num_input_bytes);
    for (size_t i = 0; i < num_input_bytes; ++i) {
        input.push_back(engine.get_random_uint8());
    }

    std::vector<uint8_t> native_output = crypto::pedersen::compress_native(input);

    byte_array circuit_input(&composer, input);
    byte_array result = plonk::stdlib::pedersen::compress(circuit_input);
    byte_array expected(&composer, native_output);

    EXPECT_EQ(result.get_value(), expected.get_value());

    waffle::UnrolledTurboProver prover = composer.create_unrolled_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::UnrolledTurboVerifier verifier = composer.create_unrolled_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_multi_compress)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    for (size_t i = 0; i < 7; ++i) {
        std::vector<barretenberg::fr> inputs;
        inputs.push_back(barretenberg::fr::random_element());
        inputs.push_back(barretenberg::fr::random_element());
        inputs.push_back(barretenberg::fr::random_element());
        inputs.push_back(barretenberg::fr::random_element());

        if (i == 1) {
            inputs[0] = barretenberg::fr(0);
        }
        if (i == 2) {
            inputs[1] = barretenberg::fr(0);
            inputs[2] = barretenberg::fr(0);
        }
        if (i == 3) {
            inputs[3] = barretenberg::fr(0);
        }
        if (i == 4) {
            inputs[0] = barretenberg::fr(0);
            inputs[3] = barretenberg::fr(0);
        }
        if (i == 5) {
            inputs[0] = barretenberg::fr(0);
            inputs[1] = barretenberg::fr(0);
            inputs[2] = barretenberg::fr(0);
            inputs[3] = barretenberg::fr(0);
        }
        if (i == 6) {
            inputs[1] = barretenberg::fr(1);
        }
        std::vector<field_t> witnesses;
        for (auto input : inputs) {
            witnesses.push_back(witness_t(&composer, input));
        }

        barretenberg::fr expected = crypto::pedersen::compress_native(inputs);

        field_t result = plonk::stdlib::pedersen::compress(witnesses, true);
        EXPECT_EQ(result.get_value(), expected);
    }

    waffle::UnrolledTurboProver prover = composer.create_unrolled_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::UnrolledTurboVerifier verifier = composer.create_unrolled_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_compress_eight)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    std::array<barretenberg::fr, 8> inputs;
    std::array<plonk::stdlib::field_t<waffle::TurboComposer>, 8> witness_inputs;

    for (size_t i = 0; i < 8; ++i) {
        inputs[i] = barretenberg::fr::random_element();
        witness_inputs[i] = witness_t(&composer, inputs[i]);
    }

    barretenberg::fr expected = crypto::pedersen::compress_eight_native(inputs);
    auto result = plonk::stdlib::pedersen::compress_eight(witness_inputs);

    EXPECT_EQ(result.get_value(), expected);
}
} // namespace test_stdlib_pedersen