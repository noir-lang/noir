#include "acir_format.hpp"
#include "recursion_constraint.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace proof_system::plonk;

acir_format::Composer create_inner_circuit()
{
    /**
     * constraints produced by Noir program:
     * fn main(x : u32, y : pub u32) {
     * let z = x ^ y;
     *
     * constrain z != 10;
     * }
     **/
    acir_format::RangeConstraint range_a{
        .witness = 1,
        .num_bits = 32,
    };
    acir_format::RangeConstraint range_b{
        .witness = 2,
        .num_bits = 32,
    };

    acir_format::LogicConstraint logic_constraint{
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

    acir_format::acir_format constraint_system{
        .varnum = 7,
        .public_inputs = { 2, 3 },
        .fixed_base_scalar_mul_constraints = {},
        .logic_constraints = { logic_constraint },
        .range_constraints = { range_a, range_b },
        .schnorr_constraints = {},
        .ecdsa_constraints = {},
        .sha256_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .hash_to_field_constraints = {},
        .pedersen_constraints = {},
        .block_constraints = {},
        .recursion_constraints = {},
        .constraints = { expr_a, expr_b, expr_c, expr_d },
    };

    uint256_t inverse_of_five = fr(5).invert();
    auto composer = acir_format::create_circuit_with_witness(constraint_system,
                                                             {
                                                                 5,
                                                                 10,
                                                                 15,
                                                                 5,
                                                                 inverse_of_five,
                                                                 1,
                                                             });

    return composer;
}

/**
 * @brief Create a circuit that recursively verifies one or more inner circuits
 *
 * @param inner_composers
 * @return acir_format::Composer
 */
acir_format::Composer create_outer_circuit(std::vector<acir_format::Composer>& inner_composers)
{
    std::vector<acir_format::RecursionConstraint> recursion_constraints;

    // witness count starts at 1 (Composer reserves 1st witness to be the zero-valued zero_idx)
    size_t witness_offset = 1;
    std::array<uint32_t, acir_format::RecursionConstraint::AGGREGATION_OBJECT_SIZE> output_aggregation_object;
    std::vector<fr, ContainerSlabAllocator<fr>> witness;

    for (size_t i = 0; i < inner_composers.size(); ++i) {
        const bool has_input_aggregation_object = i > 0;

        auto& inner_composer = inner_composers[i];
        auto inner_prover = inner_composer.create_prover();
        auto inner_proof = inner_prover.construct_proof();
        auto inner_verifier = inner_composer.create_verifier();

        const bool has_nested_proof = inner_verifier.key->contains_recursive_proof;
        const size_t num_inner_public_inputs = inner_composer.get_public_inputs().size();

        transcript::StandardTranscript transcript(inner_proof.proof_data,
                                                  acir_format::Composer::create_manifest(num_inner_public_inputs),
                                                  transcript::HashType::PlookupPedersenBlake3s,
                                                  16);

        const std::vector<barretenberg::fr> proof_witnesses =
            acir_format::export_transcript_in_recursion_format(transcript);
        const std::vector<barretenberg::fr> key_witnesses =
            acir_format::export_key_in_recursion_format(inner_verifier.key);

        const uint32_t key_hash_start_idx = static_cast<uint32_t>(witness_offset);
        const uint32_t public_input_start_idx = key_hash_start_idx + 1;
        const uint32_t output_aggregation_object_start_idx =
            static_cast<uint32_t>(public_input_start_idx + num_inner_public_inputs + (has_nested_proof ? 16 : 0));
        const uint32_t proof_indices_start_idx = output_aggregation_object_start_idx + 16;
        const uint32_t key_indices_start_idx = static_cast<uint32_t>(proof_indices_start_idx + proof_witnesses.size());

        std::vector<uint32_t> proof_indices;
        std::vector<uint32_t> key_indices;
        std::vector<uint32_t> inner_public_inputs;
        std::array<uint32_t, acir_format::RecursionConstraint::AGGREGATION_OBJECT_SIZE> input_aggregation_object = {};
        std::array<uint32_t, acir_format::RecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object = {};
        if (has_input_aggregation_object) {
            input_aggregation_object = output_aggregation_object;
        }
        for (size_t i = 0; i < 16; ++i) {
            output_aggregation_object[i] = (static_cast<uint32_t>(i + output_aggregation_object_start_idx));
        }
        if (has_nested_proof) {
            for (size_t i = 0; i < 16; ++i) {
                nested_aggregation_object[i] = inner_composer.recursive_proof_public_input_indices[i];
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

        acir_format::RecursionConstraint recursion_constraint{
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
    }

    std::vector<uint32_t> public_inputs(output_aggregation_object.begin(), output_aggregation_object.end());

    acir_format::acir_format constraint_system{
        .varnum = static_cast<uint32_t>(witness.size() + 1),
        .public_inputs = public_inputs,
        .fixed_base_scalar_mul_constraints = {},
        .logic_constraints = {},
        .range_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_constraints = {},
        .sha256_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .hash_to_field_constraints = {},
        .pedersen_constraints = {},
        .block_constraints = {},
        .recursion_constraints = recursion_constraints,
        .constraints = {},
    };

    auto composer = acir_format::create_circuit_with_witness(constraint_system, witness);

    return composer;
}

TEST(RecursionConstraint, TestBasicDoubleRecursionConstraints)
{
    std::vector<acir_format::Composer> layer_1_composers;
    layer_1_composers.push_back(create_inner_circuit());

    layer_1_composers.push_back(create_inner_circuit());

    auto layer_2_composer = create_outer_circuit(layer_1_composers);

    std::cout << "composer gates = " << layer_2_composer.get_num_gates() << std::endl;
    auto prover = layer_2_composer.create_ultra_with_keccak_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    auto proof = prover.construct_proof();
    auto verifier = layer_2_composer.create_ultra_with_keccak_verifier();
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
    std::vector<acir_format::Composer> layer_1_composers;
    layer_1_composers.push_back(create_inner_circuit());
    std::cout << "created first inner circuit\n";
    std::vector<acir_format::Composer> layer_2_composers;

    layer_2_composers.push_back(create_inner_circuit());
    std::cout << "created second inner circuit\n";

    layer_2_composers.push_back(create_outer_circuit(layer_1_composers));
    std::cout << "created first outer circuit\n";

    auto layer_3_composer = create_outer_circuit(layer_2_composers);
    std::cout << "created second outer circuit\n";

    std::cout << "composer gates = " << layer_3_composer.get_num_gates() << std::endl;
    auto prover = layer_3_composer.create_ultra_with_keccak_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    auto proof = prover.construct_proof();
    auto verifier = layer_3_composer.create_ultra_with_keccak_verifier();
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(RecursionConstraint, TestFullRecursiveComposition)
{
    std::vector<acir_format::Composer> layer_b_1_composers;
    layer_b_1_composers.push_back(create_inner_circuit());
    std::cout << "created first inner circuit\n";

    std::vector<acir_format::Composer> layer_b_2_composers;
    layer_b_2_composers.push_back(create_inner_circuit());
    std::cout << "created second inner circuit\n";

    std::vector<acir_format::Composer> layer_2_composers;
    layer_2_composers.push_back(create_outer_circuit(layer_b_1_composers));
    std::cout << "created first outer circuit\n";

    layer_2_composers.push_back(create_outer_circuit(layer_b_2_composers));
    std::cout << "created second outer circuit\n";

    auto layer_3_composer = create_outer_circuit(layer_2_composers);
    std::cout << "created third outer circuit\n";

    std::cout << "composer gates = " << layer_3_composer.get_num_gates() << std::endl;
    auto prover = layer_3_composer.create_ultra_with_keccak_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    auto proof = prover.construct_proof();
    auto verifier = layer_3_composer.create_ultra_with_keccak_verifier();
    EXPECT_EQ(verifier.verify_proof(proof), true);
}