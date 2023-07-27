#include "acir_format.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "recursion_constraint.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace proof_system::plonk;

namespace acir_format::test {
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
        .witness = 1,
        .num_bits = 32,
    };
    RangeConstraint range_b{
        .witness = 2,
        .num_bits = 32,
    };

    LogicConstraint logic_constraint{
        .a = 1,
        .b = 2,
        .result = 3,
        .num_bits = 32,
        .is_xor_gate = 1,
    };
    poly_triple expr_a{
        .a = 3,
        .b = 4,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = -1,
        .q_o = 0,
        .q_c = -10,
    };
    poly_triple expr_b{
        .a = 4,
        .b = 5,
        .c = 6,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,
    };
    poly_triple expr_c{
        .a = 4,
        .b = 6,
        .c = 4,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,

    };
    poly_triple expr_d{
        .a = 6,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = -1,
        .q_r = 0,
        .q_o = 0,
        .q_c = 1,
    };

    acir_format constraint_system{ .varnum = 7,
                                   .public_inputs = { 2, 3 },
                                   .logic_constraints = { logic_constraint },
                                   .range_constraints = { range_a, range_b },
                                   .sha256_constraints = {},
                                   .schnorr_constraints = {},
                                   .ecdsa_k1_constraints = {},
                                   .ecdsa_r1_constraints = {},
                                   .blake2s_constraints = {},
                                   .keccak_constraints = {},
                                   .keccak_var_constraints = {},
                                   .pedersen_constraints = {},
                                   .hash_to_field_constraints = {},
                                   .fixed_base_scalar_mul_constraints = {},
                                   .recursion_constraints = {},
                                   .constraints = { expr_a, expr_b, expr_c, expr_d },
                                   .block_constraints = {} };

    uint256_t inverse_of_five = fr(5).invert();
    auto builder = create_circuit_with_witness(constraint_system,
                                               {
                                                   5,
                                                   10,
                                                   15,
                                                   5,
                                                   inverse_of_five,
                                                   1,
                                               });

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
    std::vector<RecursionConstraint> recursion_constraints;

    // witness count starts at 1 (Composer reserves 1st witness to be the zero-valued zero_idx)
    size_t witness_offset = 1;
    std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> output_aggregation_object;
    std::vector<fr, ContainerSlabAllocator<fr>> witness;

    size_t circuit_idx = 0;
    for (auto& inner_circuit : inner_circuits) {
        const bool has_input_aggregation_object = circuit_idx > 0;

        auto inner_composer = Composer();
        auto inner_prover = inner_composer.create_prover(inner_circuit);
        auto inner_proof = inner_prover.construct_proof();
        auto inner_verifier = inner_composer.create_verifier(inner_circuit);

        const bool has_nested_proof = inner_verifier.key->contains_recursive_proof;
        const size_t num_inner_public_inputs = inner_circuit.get_public_inputs().size();

        transcript::StandardTranscript transcript(inner_proof.proof_data,
                                                  Composer::create_manifest(num_inner_public_inputs),
                                                  transcript::HashType::PlookupPedersenBlake3s,
                                                  16);

        const std::vector<barretenberg::fr> proof_witnesses = export_transcript_in_recursion_format(transcript);
        const std::vector<barretenberg::fr> key_witnesses = export_key_in_recursion_format(inner_verifier.key);

        const uint32_t key_hash_start_idx = static_cast<uint32_t>(witness_offset);
        const uint32_t public_input_start_idx = key_hash_start_idx + 1;
        const uint32_t output_aggregation_object_start_idx =
            static_cast<uint32_t>(public_input_start_idx + num_inner_public_inputs + (has_nested_proof ? 16 : 0));
        const uint32_t proof_indices_start_idx = output_aggregation_object_start_idx + 16;
        const uint32_t key_indices_start_idx = static_cast<uint32_t>(proof_indices_start_idx + proof_witnesses.size());

        std::vector<uint32_t> proof_indices;
        std::vector<uint32_t> key_indices;
        std::vector<uint32_t> inner_public_inputs;
        std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> input_aggregation_object = {};
        std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object = {};
        if (has_input_aggregation_object) {
            input_aggregation_object = output_aggregation_object;
        }
        for (size_t i = 0; i < 16; ++i) {
            output_aggregation_object[i] = (static_cast<uint32_t>(i + output_aggregation_object_start_idx));
        }
        if (has_nested_proof) {
            for (size_t i = 0; i < 16; ++i) {
                nested_aggregation_object[i] = inner_circuit.recursive_proof_public_input_indices[i];
            }
        }
        for (size_t i = 0; i < proof_witnesses.size(); ++i) {
            proof_indices.emplace_back(static_cast<uint32_t>(i + proof_indices_start_idx));
        }
        const size_t key_size = key_witnesses.size();
        for (size_t i = 0; i < key_size; ++i) {
            key_indices.emplace_back(static_cast<uint32_t>(i + key_indices_start_idx));
        }
        for (size_t i = 0; i < num_inner_public_inputs; ++i) {
            inner_public_inputs.push_back(static_cast<uint32_t>(i + public_input_start_idx));
        }

        RecursionConstraint recursion_constraint{
            .key = key_indices,
            .proof = proof_indices,
            .public_inputs = inner_public_inputs,
            .key_hash = key_hash_start_idx,
            .input_aggregation_object = input_aggregation_object,
            .output_aggregation_object = output_aggregation_object,
            .nested_aggregation_object = nested_aggregation_object,
        };
        recursion_constraints.push_back(recursion_constraint);
        for (size_t i = 0; i < proof_indices_start_idx - witness_offset; ++i) {
            witness.emplace_back(0);
        }
        for (const auto& wit : proof_witnesses) {
            witness.emplace_back(wit);
        }
        for (const auto& wit : key_witnesses) {
            witness.emplace_back(wit);
        }
        witness_offset = key_indices_start_idx + key_witnesses.size();
        circuit_idx++;
    }

    std::vector<uint32_t> public_inputs(output_aggregation_object.begin(), output_aggregation_object.end());

    acir_format constraint_system{ .varnum = static_cast<uint32_t>(witness.size() + 1),
                                   .public_inputs = public_inputs,
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
                                   .fixed_base_scalar_mul_constraints = {},
                                   .recursion_constraints = recursion_constraints,
                                   .constraints = {},
                                   .block_constraints = {} };

    auto outer_circuit = create_circuit_with_witness(constraint_system, witness);

    return outer_circuit;
}

TEST(RecursionConstraint, TestBasicDoubleRecursionConstraints)
{
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());

    layer_1_circuits.push_back(create_inner_circuit());

    auto layer_2_circuit = create_outer_circuit(layer_1_circuits);

    info("circuit gates = ", layer_2_circuit.get_num_gates());

    auto layer_2_composer = Composer();
    auto prover = layer_2_composer.create_ultra_with_keccak_prover(layer_2_circuit);
    info("prover gates = ", prover.circuit_size);
    auto proof = prover.construct_proof();
    auto verifier = layer_2_composer.create_ultra_with_keccak_verifier(layer_2_circuit);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(RecursionConstraint, TestOneOuterRecursiveCircuit)
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

    auto layer_3_composer = Composer();
    auto prover = layer_3_composer.create_ultra_with_keccak_prover(layer_3_circuit);
    info("prover gates = ", prover.circuit_size);
    auto proof = prover.construct_proof();
    auto verifier = layer_3_composer.create_ultra_with_keccak_verifier(layer_3_circuit);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(RecursionConstraint, TestFullRecursiveComposition)
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

    auto layer_3_composer = Composer();
    auto prover = layer_3_composer.create_ultra_with_keccak_prover(layer_3_circuit);
    info("prover gates = ", prover.circuit_size);
    auto proof = prover.construct_proof();
    auto verifier = layer_3_composer.create_ultra_with_keccak_verifier(layer_3_circuit);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
} // namespace acir_format::test
