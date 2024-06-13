#include "honk_recursion_constraint.hpp"
#include "acir_format.hpp"
#include "acir_format_mocks.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace acir_format;
using namespace bb;

class AcirHonkRecursionConstraint : public ::testing::Test {

  public:
    using ProverInstance = ProverInstance_<UltraFlavor>;
    using Prover = bb::UltraProver;
    using VerificationKey = UltraFlavor::VerificationKey;
    using Verifier = bb::UltraVerifier;

    Builder create_inner_circuit()
    {
        /**
         * constraints produced by Noir program:
         * fn main(x : u32, y : pub u32) {
         * let z = x ^ y;
         *
         * constrain z != 10;
         * }
         **/
        RangeConstraint range_a{
            .witness = 0,
            .num_bits = 32,
        };
        RangeConstraint range_b{
            .witness = 1,
            .num_bits = 32,
        };

        LogicConstraint logic_constraint{
            .a = 0,
            .b = 1,
            .result = 2,
            .num_bits = 32,
            .is_xor_gate = 1,
        };
        poly_triple expr_a{
            .a = 2,
            .b = 3,
            .c = 0,
            .q_m = 0,
            .q_l = 1,
            .q_r = -1,
            .q_o = 0,
            .q_c = -10,
        };
        poly_triple expr_b{
            .a = 3,
            .b = 4,
            .c = 5,
            .q_m = 1,
            .q_l = 0,
            .q_r = 0,
            .q_o = -1,
            .q_c = 0,
        };
        poly_triple expr_c{
            .a = 3,
            .b = 5,
            .c = 3,
            .q_m = 1,
            .q_l = 0,
            .q_r = 0,
            .q_o = -1,
            .q_c = 0,

        };
        poly_triple expr_d{
            .a = 5,
            .b = 0,
            .c = 0,
            .q_m = 0,
            .q_l = -1,
            .q_r = 0,
            .q_o = 0,
            .q_c = 1,
        };

        AcirFormat constraint_system{
            .varnum = 6,
            .recursive = true,
            .num_acir_opcodes = 7,
            .public_inputs = { 1, 2 },
            .logic_constraints = { logic_constraint },
            .range_constraints = { range_a, range_b },
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
            .poly_triple_constraints = { expr_a, expr_b, expr_c, expr_d },
            .quad_constraints = {},
            .block_constraints = {},
            .original_opcode_indices = create_empty_original_opcode_indices(),
        };
        mock_opcode_indices(constraint_system);

        uint256_t inverse_of_five = fr(5).invert();
        WitnessVector witness{
            5, 10, 15, 5, inverse_of_five, 1,
        };
        auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness, /*honk recursion*/ true);

        return builder;
    }

    /**
     * @brief Create a circuit that recursively verifies one or more inner circuits
     *
     * @param inner_circuits
     * @return Composer
     */
    Builder create_outer_circuit(std::vector<Builder>& inner_circuits)
    {
        std::vector<HonkRecursionConstraint> honk_recursion_constraints;

        size_t witness_offset = 0;
        std::vector<fr, ContainerSlabAllocator<fr>> witness;

        for (auto& inner_circuit : inner_circuits) {

            auto instance = std::make_shared<ProverInstance>(inner_circuit);
            Prover prover(instance);
            auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
            Verifier verifier(verification_key);
            auto inner_proof = prover.construct_proof();

            const size_t num_inner_public_inputs = inner_circuit.get_public_inputs().size();

            std::vector<fr> proof_witnesses = inner_proof;
            // where the inner public inputs start (after circuit_size, num_pub_inputs, pub_input_offset)
            const size_t inner_public_input_offset = 3;
            // - Save the public inputs so that we can set their values.
            // - Then truncate them from the proof because the ACIR API expects proofs without public inputs
            std::vector<fr> inner_public_input_values(
                proof_witnesses.begin() + static_cast<std::ptrdiff_t>(inner_public_input_offset),
                proof_witnesses.begin() +
                    static_cast<std::ptrdiff_t>(inner_public_input_offset + num_inner_public_inputs -
                                                RecursionConstraint::AGGREGATION_OBJECT_SIZE));

            // We want to make sure that we do not remove the nested aggregation object.
            proof_witnesses.erase(proof_witnesses.begin() + static_cast<std::ptrdiff_t>(inner_public_input_offset),
                                  proof_witnesses.begin() +
                                      static_cast<std::ptrdiff_t>(inner_public_input_offset + num_inner_public_inputs -
                                                                  RecursionConstraint::AGGREGATION_OBJECT_SIZE));

            std::vector<bb::fr> key_witnesses = verification_key->to_field_elements();

            // This is the structure of proof_witnesses and key_witnesses concatenated, which is what we end up putting
            // in witness:
            // [ circuit size, num_pub_inputs, pub_input_offset, public_input_0, public_input_1, agg_obj_0,
            // agg_obj_1, ..., agg_obj_15, rest of proof..., vkey_0, vkey_1, vkey_2, vkey_3...]
            const uint32_t public_input_start_idx =
                static_cast<uint32_t>(inner_public_input_offset + witness_offset); // points to public_input_0
            const uint32_t proof_indices_start_idx =
                static_cast<uint32_t>(public_input_start_idx + num_inner_public_inputs -
                                      RecursionConstraint::AGGREGATION_OBJECT_SIZE); // points to agg_obj_0
            const uint32_t key_indices_start_idx =
                static_cast<uint32_t>(proof_indices_start_idx + proof_witnesses.size() -
                                      inner_public_input_offset); // would point to vkey_3 without the -
                                                                  // inner_public_input_offset, points to vkey_0

            std::vector<uint32_t> proof_indices;
            std::vector<uint32_t> key_indices;
            std::vector<uint32_t> inner_public_inputs;
            for (size_t i = 0; i < inner_public_input_offset; ++i) { // go over circuit size, num_pub_inputs, pub_offset
                proof_indices.emplace_back(static_cast<uint32_t>(i + witness_offset));
            }
            for (size_t i = 0; i < proof_witnesses.size() - inner_public_input_offset;
                 ++i) { // goes over agg_obj_0, agg_obj_1, ..., agg_obj_15 and rest of proof
                proof_indices.emplace_back(static_cast<uint32_t>(i + proof_indices_start_idx));
            }
            const size_t key_size = key_witnesses.size();
            for (size_t i = 0; i < key_size; ++i) {
                key_indices.emplace_back(static_cast<uint32_t>(i + key_indices_start_idx));
            }
            // We keep the nested aggregation object attached to the proof,
            // thus we do not explicitly have to keep the public inputs while setting up the initial recursion
            // constraint. They will later be attached as public inputs when creating the circuit.
            for (size_t i = 0; i < num_inner_public_inputs - RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                inner_public_inputs.push_back(static_cast<uint32_t>(i + public_input_start_idx));
            }

            HonkRecursionConstraint honk_recursion_constraint{
                .key = key_indices,
                .proof = proof_indices,
                .public_inputs = inner_public_inputs,
            };
            honk_recursion_constraints.push_back(honk_recursion_constraint);

            // Setting the witness vector which just appends proof witnesses and key witnesses.
            // We need to reconstruct the proof witnesses in the same order as the proof indices, with this structure:
            // [ circuit size, num_pub_inputs, pub_input_offset, public_input_0, public_input_1, agg_obj_0,
            // agg_obj_1, ..., agg_obj_15, rest of proof..., vkey_0, vkey_1, vkey_2, vkey_3...]
            size_t idx = 0;
            for (const auto& wit : proof_witnesses) {
                witness.emplace_back(wit);
                idx++;
                if (idx ==
                    inner_public_input_offset) { // before this is true, the loop adds the first three into witness
                    for (size_t i = 0; i < proof_indices_start_idx - public_input_start_idx;
                         ++i) { // adds the inner public inputs
                        witness.emplace_back(0);
                    }
                } // after this, it adds the agg obj and rest of proof
            }

            for (const auto& wit : key_witnesses) {
                witness.emplace_back(wit);
            }

            // Set the values for the inner public inputs
            // TODO(maxim): check this is wrong I think
            // Note: this is confusing, but we minus one here due to the fact that the
            // witness values have not taken into account that zero is taken up by the zero_idx
            //
            // We once again have to check whether we have a nested proof, because if we do have one
            // then we could get a segmentation fault as `inner_public_inputs` was never filled with values.
            for (size_t i = 0; i < num_inner_public_inputs - RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                witness[inner_public_inputs[i]] = inner_public_input_values[i];
            }

            witness_offset = key_indices_start_idx + key_witnesses.size();
        }

        std::vector<size_t> honk_recursion_opcode_indices(honk_recursion_constraints.size());
        std::iota(honk_recursion_opcode_indices.begin(), honk_recursion_opcode_indices.end(), 0);

        AcirFormat constraint_system{
            .varnum = static_cast<uint32_t>(witness.size()),
            .recursive = false,
            .num_acir_opcodes = static_cast<uint32_t>(honk_recursion_constraints.size()),
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
            .honk_recursion_constraints = honk_recursion_constraints,
            .bigint_from_le_bytes_constraints = {},
            .bigint_to_le_bytes_constraints = {},
            .bigint_operations = {},
            .poly_triple_constraints = {},
            .quad_constraints = {},
            .block_constraints = {},
            .original_opcode_indices = create_empty_original_opcode_indices(),
        };
        mock_opcode_indices(constraint_system);
        auto outer_circuit = create_circuit(constraint_system, /*size_hint*/ 0, witness, /*honk recursion*/ true);

        return outer_circuit;
    }

  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(AcirHonkRecursionConstraint, TestBasicDoubleHonkRecursionConstraints)
{
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());

    layer_1_circuits.push_back(create_inner_circuit());

    auto layer_2_circuit = create_outer_circuit(layer_1_circuits);

    info("circuit gates = ", layer_2_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_2_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirHonkRecursionConstraint, TestOneOuterRecursiveCircuit)
{
    /**
     * We want to test the following:
     * 1. circuit that verifies a proof of another circuit
     * 2. the above, but the inner circuit contains a recursive proof output that we have to aggregate
     * 3. the above, but the outer circuit verifies 2 proofs, the aggregation outputs from the 2 proofs (+ the recursive
     * proof output from 2) are aggregated together
     *
     * A = basic circuit
     * B = circuit that verifies proof of A
     * C = circuit that verifies proof of B and a proof of A
     *
     * Layer 1 = proof of A
     * Layer 2 = verifies proof of A and proof of B
     * Layer 3 = verifies proof of C
     *
     * Attempt at a visual graphic
     * ===========================
     *
     *     C
     *     ^
     *     |
     *     | - B
     *     ^   ^
     *     |   |
     *     |    -A
     *     |
     *      - A
     *
     * ===========================
     *
     * Final aggregation object contains aggregated proofs for 2 instances of A and 1 instance of B
     */
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());
    info("created first inner circuit");

    std::vector<Builder> layer_2_circuits;
    layer_2_circuits.push_back(create_inner_circuit());
    info("created second inner circuit");

    layer_2_circuits.push_back(create_outer_circuit(layer_1_circuits));
    info("created first outer circuit");

    auto layer_3_circuit = create_outer_circuit(layer_2_circuits);
    info("created second outer circuit");
    info("number of gates in layer 3 = ", layer_3_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_3_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirHonkRecursionConstraint, TestFullRecursiveComposition)
{
    std::vector<Builder> layer_b_1_circuits;
    layer_b_1_circuits.push_back(create_inner_circuit());
    info("created first inner circuit");

    std::vector<Builder> layer_b_2_circuits;
    layer_b_2_circuits.push_back(create_inner_circuit());
    info("created second inner circuit");

    std::vector<Builder> layer_2_circuits;
    layer_2_circuits.push_back(create_outer_circuit(layer_b_1_circuits));
    info("created first outer circuit");

    layer_2_circuits.push_back(create_outer_circuit(layer_b_2_circuits));
    info("created second outer circuit");

    auto layer_3_circuit = create_outer_circuit(layer_2_circuits);
    info("created third outer circuit");
    info("number of gates in layer 3 circuit = ", layer_3_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_3_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
