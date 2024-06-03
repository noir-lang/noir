#include "block_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace acir_format;
using Composer = plonk::UltraComposer;

class UltraPlonkRAM : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

class MegaHonk : public ::testing::Test {
  public:
    using Flavor = MegaFlavor;
    using Builder = Flavor::CircuitBuilder;
    using Prover = UltraProver_<Flavor>;
    using Verifier = UltraVerifier_<Flavor>;
    using VerificationKey = Flavor::VerificationKey;

    // Construct and verify an MegaHonk proof for the provided circuit
    static bool prove_and_verify(Builder& circuit)
    {
        Prover prover{ circuit };
        auto proof = prover.construct_proof();

        auto verification_key = std::make_shared<VerificationKey>(prover.instance->proving_key);
        Verifier verifier{ verification_key };

        return verifier.verify_proof(proof);
    }

  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
size_t generate_block_constraint(BlockConstraint& constraint, WitnessVector& witness_values)
{
    size_t witness_len = 0;
    witness_values.emplace_back(1);
    witness_len++;

    fr two = fr::one() + fr::one();
    poly_triple a0{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = two,
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    fr three = fr::one() + two;
    poly_triple a1{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 0,
        .q_r = 0,
        .q_o = 0,
        .q_c = three,
    };
    poly_triple r1{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one(),
    };
    poly_triple r2{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = two,
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one(),
    };
    poly_triple y{
        .a = 1,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(2);
    witness_len++;
    poly_triple z{
        .a = 2,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(3);
    witness_len++;
    MemOp op1{
        .access_type = 0,
        .index = r1,
        .value = y,
    };
    MemOp op2{
        .access_type = 0,
        .index = r2,
        .value = z,
    };
    constraint = BlockConstraint{
        .init = { a0, a1 },
        .trace = { op1, op2 },
        .type = BlockType::ROM,
    };

    return witness_len;
}

TEST_F(UltraPlonkRAM, TestBlockConstraint)
{
    BlockConstraint block;
    WitnessVector witness_values;
    size_t num_variables = generate_block_constraint(block, witness_values);
    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .recursive = false,
        .num_acir_opcodes = 7,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .aes128_constraints = {},
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
        .multi_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = { block },
    };

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness_values);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(MegaHonk, Databus)
{
    BlockConstraint block;
    WitnessVector witness_values;
    size_t num_variables = generate_block_constraint(block, witness_values);
    block.type = BlockType::CallData;

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .recursive = false,
        .num_acir_opcodes = 1,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .aes128_constraints = {},
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
        .multi_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = { block },
    };

    // Construct a bberg circuit from the acir representation
    auto circuit = acir_format::create_circuit<Builder>(constraint_system, 0, witness_values);

    EXPECT_TRUE(prove_and_verify(circuit));
}

TEST_F(MegaHonk, DatabusReturn)
{
    BlockConstraint block;
    WitnessVector witness_values;
    size_t num_variables = generate_block_constraint(block, witness_values);
    block.type = BlockType::CallData;

    poly_triple rd_index{
        .a = static_cast<uint32_t>(num_variables),
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(0);
    ++num_variables;
    auto fr_five = fr(5);
    poly_triple rd_read{
        .a = static_cast<uint32_t>(num_variables),
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(fr_five);
    poly_triple five{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 0,
        .q_r = 0,
        .q_o = 0,
        .q_c = fr(fr_five),
    };
    ++num_variables;
    MemOp op_rd{
        .access_type = 0,
        .index = rd_index,
        .value = rd_read,
    };
    // Initialize the data_bus as [5] and read its value into rd_read
    auto return_data = BlockConstraint{
        .init = { five },
        .trace = { op_rd },
        .type = BlockType::ReturnData,
    };

    // Assert that call_data[0]+call_data[1] == return_data[0]
    poly_triple assert_equal{
        .a = 1,
        .b = 2,
        .c = rd_read.a,
        .q_m = 0,
        .q_l = 1,
        .q_r = 1,
        .q_o = fr::neg_one(),
        .q_c = 0,
    };

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .recursive = false,
        .num_acir_opcodes = 1,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .aes128_constraints = {},
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
        .multi_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = { assert_equal },
        .quad_constraints = {},
        .block_constraints = { block },
    };

    // Construct a bberg circuit from the acir representation
    auto circuit = acir_format::create_circuit<Builder>(constraint_system, 0, witness_values);

    EXPECT_TRUE(prove_and_verify(circuit));
}
