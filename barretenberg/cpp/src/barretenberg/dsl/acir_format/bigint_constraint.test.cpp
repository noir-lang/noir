#include "bigint_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {

class BigIntTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(BigIntTests, TestBigIntConstraintDummy)
{
    // Dummy Test: to be updated when big ints opcodes are implemented
    BigIntOperation add_constraint{
        .lhs = 1,
        .rhs = 2,
        .result = 3,
        .opcode = BigIntOperationType::Add,
    };
    BigIntOperation neg_constraint{
        .lhs = 1,
        .rhs = 2,
        .result = 3,
        .opcode = BigIntOperationType::Neg,
    };
    BigIntOperation mul_constraint{
        .lhs = 1,
        .rhs = 2,
        .result = 3,
        .opcode = BigIntOperationType::Mul,
    };
    BigIntOperation div_constraint{
        .lhs = 1,
        .rhs = 2,
        .result = 3,
        .opcode = BigIntOperationType::Div,
    };
    BigIntFromLeBytes from_le_bytes_constraint{
        .inputs = { 0 },
        .modulus = { 23 },
        .result = 1,
    };

    AcirFormat constraint_system{
        .varnum = 4,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
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
        .bigint_from_le_bytes_constraints = { from_le_bytes_constraint },
        .bigint_operations = { add_constraint, neg_constraint, mul_constraint, div_constraint },
        .constraints = {},
        .block_constraints = {},

    };

    WitnessVector witness{ 0, 0, 1 };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

} // namespace acir_format::tests