#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::plonk;

namespace {
auto& engine = numeric::get_debug_randomness();
}

class StandardPlonkComposer : public ::testing::Test {
  public:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(StandardPlonkComposer, BaseCase)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    fr a = fr::one();
    builder.add_public_variable(a);
    auto prover = composer.create_prover(builder);
    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, ComposerFromSerializedKeys)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    fr a = fr::one();
    builder.add_public_variable(a);

    auto pk_buf = to_buffer(*composer.compute_proving_key(builder));
    auto vk_buf = to_buffer(*composer.compute_verification_key(builder));
    auto pk_data = from_buffer<plonk::proving_key_data>(pk_buf);
    auto vk_data = from_buffer<plonk::verification_key_data>(vk_buf);

    auto crs = std::make_unique<bb::srs::factories::FileCrsFactory<curve::BN254>>("../srs_db/ignition");
    auto proving_key =
        std::make_shared<plonk::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.circuit_size + 1));
    auto verification_key = std::make_shared<plonk::verification_key>(std::move(vk_data), crs->get_verifier_crs());

    auto builder2 = StandardCircuitBuilder();
    auto composer2 = StandardComposer();
    builder2.add_public_variable(a);

    auto prover = composer2.create_prover(builder2);
    auto verifier = composer2.create_verifier(builder2);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, TestAddGateProofs)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    fr a = fr::one();
    uint32_t a_idx = builder.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    // uint32_t a_idx = builder.add_variable(a);
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    builder.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });

    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ b_idx, a_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, TestMulGateProofs)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    std::array<fr, 7> q{ fr::random_element(), fr::random_element(), fr::random_element(), fr::random_element(),
                         fr::random_element(), fr::random_element(), fr::random_element() };
    std::array<fr, 7> q_inv{
        q[0].invert(), q[1].invert(), q[2].invert(), q[3].invert(), q[4].invert(), q[5].invert(), q[6].invert(),
    };

    fr a = fr::random_element();
    fr b = fr::random_element();
    fr c = -((((q[0] * a) + (q[1] * b)) + q[3]) * q_inv[2]);
    fr d = -((((q[4] * (a * b)) + q[6]) * q_inv[5]));

    uint32_t a_idx = builder.add_variable(a);
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);

    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });

    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    builder.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    builder.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, RangeConstraint)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();

    for (size_t i = 0; i < 10; ++i) {
        uint32_t value = engine.get_random_uint32();
        fr witness_value = fr{ value, 0, 0, 0 }.to_montgomery_form();
        uint32_t witness_index = builder.add_variable(witness_value);

        // include non-nice numbers of bits, that will bleed over gate boundaries
        size_t extra_bits = 2 * (i % 4);

        std::vector<uint32_t> accumulators = builder.decompose_into_base4_accumulators(witness_index, 32 + extra_bits);

        for (uint32_t j = 0; j < 16; ++j) {
            uint32_t result = (value >> (30U - (2 * j)));
            fr source = builder.get_variable(accumulators[j + (extra_bits >> 1)]).from_montgomery_form();
            auto expected = static_cast<uint32_t>(source.data[0]);
            EXPECT_EQ(result, expected);
        }
        for (uint32_t j = 1; j < 16; ++j) {
            uint32_t left = (value >> (30U - (2 * j)));
            uint32_t right = (value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);
        }
    }

    uint32_t zero_idx = builder.add_variable(fr::zero());
    uint32_t one_idx = builder.add_variable(fr::one());
    builder.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, RangeConstraintFail)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();

    uint64_t value = 0xffffff;
    uint32_t witness_index = builder.add_variable(fr(value));

    builder.decompose_into_base4_accumulators(witness_index, 23);

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, false);
}

TEST_F(StandardPlonkComposer, AndConstraint)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();

    for (size_t i = 0; i < /*10*/ 1; ++i) {
        uint32_t left_value = engine.get_random_uint32();

        fr left_witness_value = fr{ left_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t left_witness_index = builder.add_variable(left_witness_value);

        uint32_t right_value = engine.get_random_uint32();
        fr right_witness_value = fr{ right_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t right_witness_index = builder.add_variable(right_witness_value);

        uint32_t out_value = left_value & right_value;
        // include non-nice numbers of bits, that will bleed over gate boundaries
        size_t extra_bits = 2 * (i % 4);

        auto accumulators = builder.create_and_constraint(left_witness_index, right_witness_index, 32 + extra_bits);
        // builder.create_and_constraint(left_witness_index, right_witness_index, 32 + extra_bits);

        for (uint32_t j = 0; j < 16; ++j) {
            uint32_t left_expected = (left_value >> (30U - (2 * j)));
            uint32_t right_expected = (right_value >> (30U - (2 * j)));
            uint32_t out_expected = left_expected & right_expected;

            fr left_source = builder.get_variable(accumulators.left[j + (extra_bits >> 1)]).from_montgomery_form();
            auto left_result = static_cast<uint32_t>(left_source.data[0]);

            fr right_source = builder.get_variable(accumulators.right[j + (extra_bits >> 1)]).from_montgomery_form();
            auto right_result = static_cast<uint32_t>(right_source.data[0]);

            fr out_source = builder.get_variable(accumulators.out[j + (extra_bits >> 1)]).from_montgomery_form();
            auto out_result = static_cast<uint32_t>(out_source.data[0]);

            EXPECT_EQ(left_result, left_expected);
            EXPECT_EQ(right_result, right_expected);
            EXPECT_EQ(out_result, out_expected);
        }
        for (uint32_t j = 1; j < 16; ++j) {
            uint32_t left = (left_value >> (30U - (2 * j)));
            uint32_t right = (left_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (right_value >> (30U - (2 * j)));
            right = (right_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (out_value >> (30U - (2 * j)));
            right = (out_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);
        }
    }

    uint32_t zero_idx = builder.add_variable(fr::zero());
    uint32_t one_idx = builder.add_variable(fr::one());
    builder.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, XorConstraint)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();

    for (size_t i = 0; i < /*10*/ 1; ++i) {
        uint32_t left_value = engine.get_random_uint32();

        fr left_witness_value = fr{ left_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t left_witness_index = builder.add_variable(left_witness_value);

        uint32_t right_value = engine.get_random_uint32();
        fr right_witness_value = fr{ right_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t right_witness_index = builder.add_variable(right_witness_value);

        uint32_t out_value = left_value ^ right_value;
        // include non-nice numbers of bits, that will bleed over gate boundaries
        size_t extra_bits = 2 * (i % 4);

        auto accumulators = builder.create_xor_constraint(left_witness_index, right_witness_index, 32 + extra_bits);

        for (uint32_t j = 0; j < 16; ++j) {
            uint32_t left_expected = (left_value >> (30U - (2 * j)));
            uint32_t right_expected = (right_value >> (30U - (2 * j)));
            uint32_t out_expected = left_expected ^ right_expected;

            fr left_source = builder.get_variable(accumulators.left[j + (extra_bits >> 1)]).from_montgomery_form();
            auto left_result = static_cast<uint32_t>(left_source.data[0]);

            fr right_source = builder.get_variable(accumulators.right[j + (extra_bits >> 1)]).from_montgomery_form();
            auto right_result = static_cast<uint32_t>(right_source.data[0]);

            fr out_source = builder.get_variable(accumulators.out[j + (extra_bits >> 1)]).from_montgomery_form();
            auto out_result = static_cast<uint32_t>(out_source.data[0]);

            EXPECT_EQ(left_result, left_expected);
            EXPECT_EQ(right_result, right_expected);
            EXPECT_EQ(out_result, out_expected);
        }
        for (uint32_t j = 1; j < 16; ++j) {
            uint32_t left = (left_value >> (30U - (2 * j)));
            uint32_t right = (left_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (right_value >> (30U - (2 * j)));
            right = (right_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (out_value >> (30U - (2 * j)));
            right = (out_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);
        }
    }

    uint32_t zero_idx = builder.add_variable(fr::zero());
    uint32_t one_idx = builder.add_variable(fr::one());
    builder.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, BigAddGateWithBitExtract)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();

    const auto generate_constraints = [&](uint32_t quad_value) {
        uint32_t quad_accumulator_left =
            (engine.get_random_uint32() & 0x3fffffff) - quad_value; // make sure this won't overflow
        uint32_t quad_accumulator_right = (4 * quad_accumulator_left) + quad_value;

        uint32_t left_idx = builder.add_variable(uint256_t(quad_accumulator_left));
        uint32_t right_idx = builder.add_variable(uint256_t(quad_accumulator_right));

        uint32_t input = engine.get_random_uint32();
        uint32_t output = input + (quad_value > 1 ? 1 : 0);

        builder.create_big_add_gate_with_bit_extraction({ builder.add_variable(uint256_t(input)),
                                                          builder.add_variable(uint256_t(output)),
                                                          right_idx,
                                                          left_idx,
                                                          fr(6),
                                                          -fr(6),
                                                          fr::zero(),
                                                          fr::zero(),
                                                          fr::zero() });
    };

    generate_constraints(0);
    generate_constraints(1);
    generate_constraints(2);
    generate_constraints(3);

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, TestRangeConstraintFail)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    uint32_t witness_index = builder.add_variable(fr::neg_one());
    builder.decompose_into_base4_accumulators(witness_index, 32);

    auto prover = composer.create_prover(builder);

    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, false);
}

TEST_F(StandardPlonkComposer, TestCheckCircuitCorrect)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    fr a = fr::one();
    uint32_t a_idx = builder.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    // uint32_t a_idx = builder.add_variable(a);
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);

    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });

    bool result = CircuitChecker::check(builder);
    EXPECT_EQ(result, true);
}

TEST_F(StandardPlonkComposer, TestCheckCircuitBroken)
{
    auto builder = StandardCircuitBuilder();
    auto composer = StandardComposer();
    fr a = fr::one();
    uint32_t a_idx = builder.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c + 1;
    // uint32_t a_idx = builder.add_variable(a);
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);

    builder.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    builder.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });

    bool result = CircuitChecker::check(builder);
    EXPECT_EQ(result, false);
}
