#include "bigint_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {

class BigIntTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
using fr = field<Bn254FrParams>;

std::tuple<BigIntFromLeBytes, BigIntFromLeBytes, BigIntOperation, BigIntToLeBytes>
generate_big_int_op_constraint_with_modulus(
    BigIntOperationType op, fr lhs, fr rhs, WitnessVector& witness_values, const std::vector<uint32_t>& modulus)
{
    // CAUTION We assume here the operands and the result fit into one byte!
    // So trying to divide 7/2 won't work, but 8/2 will do.
    auto lhs_id = static_cast<uint32_t>(witness_values.size());
    witness_values.push_back(lhs);
    auto rhs_id = static_cast<uint32_t>(witness_values.size());
    witness_values.push_back(rhs);
    BigIntFromLeBytes from_le_bytes_constraint_bigint_lhs{
        .inputs = { lhs_id },
        .modulus = modulus,
        .result = lhs_id,
    };
    BigIntFromLeBytes from_le_bytes_constraint_bigint_rhs{
        .inputs = { rhs_id },
        .modulus = modulus,
        .result = rhs_id,
    };

    auto result = static_cast<uint32_t>(witness_values.size());
    BigIntOperation constraint{
        .lhs = lhs_id,
        .rhs = rhs_id,
        .result = result,
        .opcode = op,
    };
    // Expecting the result to be just one byte long
    BigIntToLeBytes to_bytes{
        .input = result,
        .result = { static_cast<uint32_t>(witness_values.size()) },
    };
    // overflow is NOT supported, you have to make sure there is no overflow/underflow.
    fr value = 0;
    switch (op) {
    case Add:
        value = witness_values[lhs_id] + witness_values[rhs_id];
        break;
    case Sub:
        value = witness_values[lhs_id] - witness_values[rhs_id];
        break;
    case Mul:
        value = witness_values[lhs_id] * witness_values[rhs_id];
        break;
    case Div:
        value = witness_values[lhs_id] / witness_values[rhs_id];
        break;
    default:
        ASSERT(false);
        break;
    }

    witness_values.push_back(value);
    return { from_le_bytes_constraint_bigint_lhs, from_le_bytes_constraint_bigint_rhs, constraint, to_bytes };
}

std::tuple<BigIntFromLeBytes, BigIntFromLeBytes, BigIntOperation, BigIntToLeBytes> generate_big_int_op_constraint(
    BigIntOperationType op, fr lhs, fr rhs, WitnessVector& witness_values)
{
    // modulus is bn254/fq
    return generate_big_int_op_constraint_with_modulus(
        op,
        lhs,
        rhs,
        witness_values,
        {
            0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97,
            0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30,
        });
}

std::tuple<BigIntFromLeBytes, BigIntFromLeBytes, BigIntOperation, BigIntToLeBytes>
generate_big_int_op_constraint_secpk1_fr(BigIntOperationType op, fr lhs, fr rhs, WitnessVector& witness_values)
{
    return generate_big_int_op_constraint_with_modulus(
        op, lhs, rhs, witness_values, { 0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48,
                                        0xAF, 0xE6, 0xDC, 0xAE, 0xBA, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF });
}

std::tuple<BigIntFromLeBytes, BigIntFromLeBytes, BigIntOperation, BigIntToLeBytes>
generate_big_int_op_constraint_secpk1_fq(BigIntOperationType op, fr lhs, fr rhs, WitnessVector& witness_values)
{
    return generate_big_int_op_constraint_with_modulus(
        op, lhs, rhs, witness_values, { 0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF });
}
void apply_constraints(AcirFormat& constraint_system,
                       std::tuple<BigIntFromLeBytes, BigIntFromLeBytes, BigIntOperation, BigIntToLeBytes> constraints)
{
    constraint_system.bigint_from_le_bytes_constraints.push_back(get<0>(constraints));
    constraint_system.bigint_from_le_bytes_constraints.push_back(get<1>(constraints));
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<3>(constraints));
    constraint_system.bigint_operations.push_back(get<2>(constraints));
}

std::tuple<BigIntOperation, BigIntToLeBytes> generate_big_int_op_constraint_with_id(BigIntOperationType op,
                                                                                    uint32_t lhs_id,
                                                                                    uint32_t rhs_id,
                                                                                    WitnessVector& witness_values)
{
    // lhs_id, rhs_id are big int it, so we can generate the operation directly
    auto result = static_cast<uint32_t>(witness_values.size());
    BigIntOperation constraint{
        .lhs = lhs_id,
        .rhs = rhs_id,
        .result = result,
        .opcode = op,
    };
    // Expecting the result to be just one byte long
    BigIntToLeBytes to_bytes{
        .input = result,
        .result = { static_cast<uint32_t>(witness_values.size()) },
    };
    // overflow is NOT supported, you have to make sure there is no overflow/underflow.
    fr value = 0;
    switch (op) {
    case Add:
        value = witness_values[lhs_id] + witness_values[rhs_id];
        break;
    case Sub:
        value = witness_values[lhs_id] - witness_values[rhs_id];
        break;
    case Mul:
        value = witness_values[lhs_id] * witness_values[rhs_id];
        break;
    case Div:
        value = witness_values[lhs_id] / witness_values[rhs_id];
        break;
    default:
        ASSERT(false);
        break;
    }

    witness_values.push_back(value);
    return { constraint, to_bytes };
}

// Based on TestBigIntConstraintSimple, we generate constraints for multiple operations at the same time.
TEST_F(BigIntTests, TestBigIntConstraintMultiple)
{
    WitnessVector witness;
    auto contraints = generate_big_int_op_constraint(BigIntOperationType::Add, fr(3), fr(1), witness);
    auto contraints2 = generate_big_int_op_constraint(BigIntOperationType::Add, fr(3), fr(1), witness);
    auto contraints3 = generate_big_int_op_constraint(BigIntOperationType::Sub, fr(5), fr(2), witness);
    auto contraints4 = generate_big_int_op_constraint(BigIntOperationType::Mul, fr(5), fr(3), witness);
    auto contraints5 = generate_big_int_op_constraint(BigIntOperationType::Div, fr(8), fr(2), witness);
    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(witness.size() + 1),
        .recursive = false,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},
    };
    apply_constraints(constraint_system, contraints);
    apply_constraints(constraint_system, contraints2);
    apply_constraints(constraint_system, contraints3);
    apply_constraints(constraint_system, contraints4);
    apply_constraints(constraint_system, contraints5);
    constraint_system.varnum = static_cast<uint32_t>(witness.size() + 1);

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(BigIntTests, TestBigIntConstraintSimple)
{
    // 3 + 3 = 6
    // 3 = bigint(1) = from_bytes(w(1))
    // 6 = bigint(2) = to_bytes(w(2))
    BigIntOperation add_constraint{
        .lhs = 1,
        .rhs = 1,
        .result = 2,
        .opcode = BigIntOperationType::Add,
    };

    BigIntFromLeBytes from_le_bytes_constraint_bigint1{
        .inputs = { 1 },
        .modulus = { 0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97, 
  0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30, },
        .result = 1,
    };

    BigIntToLeBytes result2_to_le_bytes{
        .input = 2, .result = { 2 }, // 3+3=6
    };

    AcirFormat constraint_system{
        .varnum = 5,
        .recursive = false,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = { from_le_bytes_constraint_bigint1 },
        .bigint_to_le_bytes_constraints = { result2_to_le_bytes },
        .bigint_operations = { add_constraint },
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},

    };

    WitnessVector witness{
        0, 3, 6, 3, 0,
    };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);
    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

// Based on TestBigIntConstraintMultiple, we generate constraints re-using the bigfields created by the first two
// operations
TEST_F(BigIntTests, TestBigIntConstraintReuse)
{
    WitnessVector witness;
    auto contraints = generate_big_int_op_constraint_secpk1_fr(BigIntOperationType::Add, fr(3), fr(1), witness);
    auto contraints2 = generate_big_int_op_constraint_secpk1_fr(BigIntOperationType::Sub, fr(5), fr(2), witness);
    auto contraints3 = generate_big_int_op_constraint_with_id(BigIntOperationType::Mul, 0, 5, witness);
    auto contraints4 = generate_big_int_op_constraint_with_id(BigIntOperationType::Div, 0, 1, witness);
    auto contraints5 = generate_big_int_op_constraint_with_id(BigIntOperationType::Sub, 7, 1, witness);

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(witness.size() + 1),
        .recursive = false,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},
    };
    apply_constraints(constraint_system, contraints);
    apply_constraints(constraint_system, contraints2);
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints3));
    constraint_system.bigint_operations.push_back(get<0>(contraints3));
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints4));
    constraint_system.bigint_operations.push_back(get<0>(contraints4));
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints5));
    constraint_system.bigint_operations.push_back(get<0>(contraints5));
    constraint_system.varnum = static_cast<uint32_t>(witness.size() + 1);

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(BigIntTests, TestBigIntConstraintReuse2)
{
    WitnessVector witness;
    auto contraints = generate_big_int_op_constraint_secpk1_fq(BigIntOperationType::Add, fr(3), fr(1), witness);
    auto contraints2 = generate_big_int_op_constraint_secpk1_fq(BigIntOperationType::Sub, fr(5), fr(2), witness);
    auto contraints3 = generate_big_int_op_constraint_with_id(BigIntOperationType::Add, 0, 5, witness);
    auto contraints4 = generate_big_int_op_constraint_with_id(BigIntOperationType::Sub, 0, 1, witness);
    auto contraints5 = generate_big_int_op_constraint_with_id(BigIntOperationType::Sub, 7, 1, witness);

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(witness.size() + 1),
        .recursive = false,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},
    };
    apply_constraints(constraint_system, contraints);
    apply_constraints(constraint_system, contraints2);
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints3));
    constraint_system.bigint_operations.push_back(get<0>(contraints3));
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints4));
    constraint_system.bigint_operations.push_back(get<0>(contraints4));
    constraint_system.bigint_to_le_bytes_constraints.push_back(get<1>(contraints5));
    constraint_system.bigint_operations.push_back(get<0>(contraints5));
    constraint_system.varnum = static_cast<uint32_t>(witness.size() + 1);

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(BigIntTests, TestBigIntDIV)
{
    // 6 / 3 = 2
    // 6 = bigint(1) = from_bytes(w(1))
    // 3 = bigint(2) = from_bytes(w(2))
    // 2 = bigint(3) = to_bytes(w(3))
    BigIntOperation div_constraint{
        .lhs = 1,
        .rhs = 2,
        .result = 3,
        .opcode = BigIntOperationType::Div,
    };

    BigIntFromLeBytes from_le_bytes_constraint_bigint1{
        .inputs = { 1 },
        .modulus = { 0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
                     0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF },
        .result = 1,
    };
    BigIntFromLeBytes from_le_bytes_constraint_bigint2{
        .inputs = { 2 },
        .modulus = { 0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
                     0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF },
        .result = 2,
    };

    BigIntToLeBytes result3_to_le_bytes{
        .input = 3, .result = { 3 }, //
    };

    AcirFormat constraint_system{
        .varnum = 5,
        .recursive = false,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = { from_le_bytes_constraint_bigint1, from_le_bytes_constraint_bigint2 },
        .bigint_to_le_bytes_constraints = { result3_to_le_bytes },
        .bigint_operations = { div_constraint },
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},

    };

    WitnessVector witness{
        0, 6, 3, 2, 0,
    };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);
    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(CircuitChecker::check(builder));

    auto builder2 = create_circuit(constraint_system, /*size_hint*/ 0, WitnessVector{});
    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier2 = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier2.verify_proof(proof), true);
}
} // namespace acir_format::tests
