#include "acir_format.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "fixed_base_scalar_mul.hpp"

#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {
using group_ct = proof_system::plonk::stdlib::group<Builder>;

size_t generate_scalar_mul_constraints(FixedBaseScalarMul& scalar_mul_constraint,
                                       WitnessVector& witness_values,
                                       uint256_t low_value,
                                       uint256_t high_value,
                                       grumpkin::g1::affine_element expected)
{
    uint32_t offset = 1;

    uint32_t low_index = offset;
    witness_values.emplace_back(low_value);
    offset += 1;

    uint32_t high_index = offset;
    witness_values.emplace_back(high_value);
    offset += 1;

    uint32_t pub_key_x_index = offset;
    witness_values.emplace_back(expected.x);
    offset += 1;

    uint32_t pub_key_y_index = offset;
    witness_values.emplace_back(expected.y);
    offset += 1;

    scalar_mul_constraint = FixedBaseScalarMul{
        .low = low_index,
        .high = high_index,
        .pub_key_x = pub_key_x_index,
        .pub_key_y = pub_key_y_index,
    };

    return offset;
}

size_t generate_fixed_base_scalar_mul_fixtures(FixedBaseScalarMul& scalar_mul_constraint,
                                               WitnessVector& witness_values,
                                               grumpkin::fr low,
                                               grumpkin::fr high)
{

    auto two_pow_128 = grumpkin::fr(2).pow(128);
    grumpkin::g1::element expected_projective = (grumpkin::g1::one * low) + grumpkin::g1::one * (high * two_pow_128);
    grumpkin::g1::affine_element expected = expected_projective.normalize();
    return generate_scalar_mul_constraints(scalar_mul_constraint, witness_values, low, high, expected);
}

TEST(FixedBaseScalarMul, TestSimpleScalarMul)
{
    FixedBaseScalarMul scalar_mul_constraint;
    WitnessVector witness_values;
    auto low = grumpkin::fr(1);
    auto high = grumpkin::fr(2);
    size_t num_variables = generate_fixed_base_scalar_mul_fixtures(scalar_mul_constraint, witness_values, low, high);
    acir_format constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .pedersen_constraints = {},
        .hash_to_field_constraints = {},
        .fixed_base_scalar_mul_constraints = { scalar_mul_constraint },
        .recursion_constraints = {},
        .constraints = {},
        .block_constraints = {},
    };

    auto builder = create_circuit_with_witness(constraint_system, witness_values);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
TEST(FixedBaseScalarMul, TestLimbLargerThan2Pow128)
{
    FixedBaseScalarMul scalar_mul_constraint;
    WitnessVector witness_values;
    grumpkin::fr low = grumpkin::fr(2).pow(129);
    grumpkin::fr high = 1;
    size_t num_variables = generate_fixed_base_scalar_mul_fixtures(scalar_mul_constraint, witness_values, low, high);
    acir_format constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .pedersen_constraints = {},
        .hash_to_field_constraints = {},
        .fixed_base_scalar_mul_constraints = { scalar_mul_constraint },
        .recursion_constraints = {},
        .constraints = {},
        .block_constraints = {},
    };

    auto builder = create_circuit_with_witness(constraint_system, witness_values);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), false);
}

} // namespace acir_format::tests
