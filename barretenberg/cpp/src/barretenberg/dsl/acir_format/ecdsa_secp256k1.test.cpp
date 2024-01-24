#include "ecdsa_secp256k1.hpp"
#include "acir_format.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace acir_format;
using curve_ct = stdlib::secp256k1<Builder>;

class ECDSASecp256k1 : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

size_t generate_ecdsa_constraint(EcdsaSecp256k1Constraint& ecdsa_constraint, WitnessVector& witness_values)
{
    std::string message_string = "Instructions unclear, ask again later.";

    // hash the message since the dsl ecdsa gadget uses the prehashed message
    // NOTE: If the hash being used outputs more than 32 bytes, then big-field will panic
    std::vector<uint8_t> message_buffer;
    std::copy(message_string.begin(), message_string.end(), std::back_inserter(message_buffer));
    auto hashed_message = sha256::sha256(message_buffer);

    crypto::ecdsa_key_pair<curve_ct::fr, curve_ct::g1> account;
    account.private_key = curve_ct::fr::random_element();
    account.public_key = curve_ct::g1::one * account.private_key;

    crypto::ecdsa_signature signature =
        crypto::ecdsa_construct_signature<Sha256Hasher, curve_ct::fq, curve_ct::fr, curve_ct::g1>(message_string,
                                                                                                  account);

    uint256_t pub_x_value = account.public_key.x;
    uint256_t pub_y_value = account.public_key.y;

    std::vector<uint32_t> message_in;
    std::vector<uint32_t> pub_x_indices_in;
    std::vector<uint32_t> pub_y_indices_in;
    std::vector<uint32_t> signature_in;
    size_t offset = 0;
    for (size_t i = 0; i < hashed_message.size(); ++i) {
        message_in.emplace_back(i + offset);
        const auto byte = static_cast<uint8_t>(hashed_message[i]);
        witness_values.emplace_back(byte);
    }
    offset += message_in.size();

    for (size_t i = 0; i < 32; ++i) {
        pub_x_indices_in.emplace_back(i + offset);
        witness_values.emplace_back(pub_x_value.slice(248 - i * 8, 256 - i * 8));
    }
    offset += pub_x_indices_in.size();
    for (size_t i = 0; i < 32; ++i) {
        pub_y_indices_in.emplace_back(i + offset);
        witness_values.emplace_back(pub_y_value.slice(248 - i * 8, 256 - i * 8));
    }
    offset += pub_y_indices_in.size();
    for (size_t i = 0; i < 32; ++i) {
        signature_in.emplace_back(i + offset);
        witness_values.emplace_back(signature.r[i]);
    }
    offset += signature.r.size();
    for (size_t i = 0; i < 32; ++i) {
        signature_in.emplace_back(i + offset);
        witness_values.emplace_back(signature.s[i]);
    }
    offset += signature.s.size();

    witness_values.emplace_back(1);
    const auto result_in = static_cast<uint32_t>(offset);
    offset += 1;
    witness_values.emplace_back(1);

    ecdsa_constraint = EcdsaSecp256k1Constraint{ .hashed_message = message_in,
                                                 .signature = signature_in,
                                                 .pub_x_indices = pub_x_indices_in,
                                                 .pub_y_indices = pub_y_indices_in,
                                                 .result = result_in };
    return offset;
}

TEST_F(ECDSASecp256k1, TestECDSAConstraintSucceed)
{
    EcdsaSecp256k1Constraint ecdsa_k1_constraint;
    WitnessVector witness_values;
    size_t num_variables = generate_ecdsa_constraint(ecdsa_k1_constraint, witness_values);
    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = { ecdsa_k1_constraint },
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .ec_double_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
        .constraints = {},
        .block_constraints = {},
    };

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness_values);

    EXPECT_EQ(builder.get_variable(ecdsa_k1_constraint.result), 1);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

// Test that the verifier can create an ECDSA circuit.
// The ECDSA circuit requires that certain dummy data is valid
// even though we are just building the circuit.
TEST_F(ECDSASecp256k1, TestECDSACompilesForVerifier)
{
    EcdsaSecp256k1Constraint ecdsa_k1_constraint;
    WitnessVector witness_values;
    size_t num_variables = generate_ecdsa_constraint(ecdsa_k1_constraint, witness_values);
    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = { ecdsa_k1_constraint },
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .ec_double_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
        .constraints = {},
        .block_constraints = {},
    };

    auto builder = create_circuit(constraint_system);
}

TEST_F(ECDSASecp256k1, TestECDSAConstraintFail)
{
    EcdsaSecp256k1Constraint ecdsa_k1_constraint;
    WitnessVector witness_values;
    size_t num_variables = generate_ecdsa_constraint(ecdsa_k1_constraint, witness_values);

    // set result value to be false
    witness_values[witness_values.size() - 1] = 0;

    // tamper with signature
    witness_values[witness_values.size() - 20] += 1;

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = { ecdsa_k1_constraint },
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .ec_double_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
        .constraints = {},
        .block_constraints = {},
    };

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness_values);
    EXPECT_EQ(builder.get_variable(ecdsa_k1_constraint.result), 0);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
