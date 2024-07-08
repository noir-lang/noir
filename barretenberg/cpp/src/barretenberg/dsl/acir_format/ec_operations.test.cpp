#include "ec_operations.hpp"
#include "acir_format.hpp"
#include "acir_format_mocks.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {

using Composer = plonk::UltraComposer;
using curve_ct = bb::stdlib::secp256k1<Builder>;

class EcOperations : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

size_t generate_ec_add_constraint(EcAdd& ec_add_constraint, WitnessVector& witness_values)
{
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    witness_values.push_back(0);
    auto g1 = grumpkin::g1::affine_one;
    cycle_group_ct input_point(g1);
    // Doubling
    cycle_group_ct result = input_point.dbl();
    // add: x,y,x2,y2
    witness_values.push_back(g1.x);
    witness_values.push_back(g1.y);
    witness_values.push_back(g1.x);
    witness_values.push_back(g1.y);
    witness_values.push_back(result.x.get_value());
    witness_values.push_back(result.y.get_value());
    witness_values.push_back(fr(0));
    witness_values.push_back(fr(0));
    ec_add_constraint = EcAdd{
        .input1_x = 1,
        .input1_y = 2,
        .input1_infinite = 7,
        .input2_x = 3,
        .input2_y = 4,
        .input2_infinite = 7,
        .result_x = 5,
        .result_y = 6,
        .result_infinite = 8,
    };
    return witness_values.size();
}

TEST_F(EcOperations, TestECOperations)
{
    EcAdd ec_add_constraint;

    WitnessVector witness_values;
    size_t num_variables = generate_ec_add_constraint(ec_add_constraint, witness_values);

    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables + 1),
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
        .ec_add_constraints = { ec_add_constraint },
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},
        .original_opcode_indices = create_empty_original_opcode_indices(),
    };
    mock_opcode_indices(constraint_system);

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness_values);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();

    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(EcOperations, TestECMultiScalarMul)
{
    MultiScalarMul msm_constrain;

    WitnessVector witness_values;
    witness_values.emplace_back(fr(0));

    witness_values = {
        // dummy
        fr(0),
        // g1: x,y,infinite
        fr(1),
        fr("0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c"),
        fr(0),
        // low, high scalars
        fr(1),
        fr(0),
        // result
        fr("0x06ce1b0827aafa85ddeb49cdaa36306d19a74caa311e13d46d8bc688cdbffffe"),
        fr("0x1c122f81a3a14964909ede0ba2a6855fc93faf6fa1a788bf467be7e7a43f80ac"),
        fr(0),
    };
    msm_constrain = MultiScalarMul{
        .points = { WitnessConstant<fr>{
                        .index = 1,
                        .value = fr(0),
                        .is_constant = false,
                    },
                    WitnessConstant<fr>{
                        .index = 2,
                        .value = fr(0),
                        .is_constant = false,
                    },
                    WitnessConstant<fr>{
                        .index = 3,
                        .value = fr(0),
                        .is_constant = false,
                    },
                    WitnessConstant<fr>{
                        .index = 1,
                        .value = fr(0),
                        .is_constant = false,
                    },
                    WitnessConstant<fr>{
                        .index = 2,
                        .value = fr(0),
                        .is_constant = false,
                    },
                    WitnessConstant<fr>{
                        .index = 3,
                        .value = fr(0),
                        .is_constant = false,
                    } },
        .scalars = { WitnessConstant<fr>{
                         .index = 4,
                         .value = fr(0),
                         .is_constant = false,
                     },
                     WitnessConstant<fr>{
                         .index = 5,
                         .value = fr(0),
                         .is_constant = false,
                     },
                     WitnessConstant<fr>{
                         .index = 4,
                         .value = fr(0),
                         .is_constant = false,
                     },
                     WitnessConstant<fr>{
                         .index = 5,
                         .value = fr(0),
                         .is_constant = false,
                     } },
        .out_point_x = 6,
        .out_point_y = 7,
        .out_point_is_infinite = 0,
    };
    auto res_x = fr("0x06ce1b0827aafa85ddeb49cdaa36306d19a74caa311e13d46d8bc688cdbffffe");
    auto assert_equal = poly_triple{
        .a = 6,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::neg_one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = res_x,
    };

    size_t num_variables = witness_values.size();
    AcirFormat constraint_system{
        .varnum = static_cast<uint32_t>(num_variables + 1),
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
        .multi_scalar_mul_constraints = { msm_constrain },
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = { assert_equal },
        .quad_constraints = {},
        .block_constraints = {},
        .original_opcode_indices = create_empty_original_opcode_indices(),
    };
    mock_opcode_indices(constraint_system);

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness_values);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();

    EXPECT_TRUE(CircuitChecker::check(builder));
    auto verifier = composer.create_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

} // namespace acir_format::tests
