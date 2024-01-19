#include "recursion_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace bb::plonk;

class AcirRecursionConstraint : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
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

    acir_format constraint_system{ .varnum = 6,
                                   .public_inputs = { 1, 2 },
                                   .logic_constraints = { logic_constraint },
                                   .range_constraints = { range_a, range_b },
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
                                   .constraints = { expr_a, expr_b, expr_c, expr_d },
                                   .block_constraints = {} };

    uint256_t inverse_of_five = fr(5).invert();
    WitnessVector witness{
        5, 10, 15, 5, inverse_of_five, 1,
    };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

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

    size_t witness_offset = 0;
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
                                                  transcript::HashType::PedersenBlake3s,
                                                  16);

        std::vector<bb::fr> proof_witnesses = export_transcript_in_recursion_format(transcript);
        // - Save the public inputs so that we can set their values.
        // - Then truncate them from the proof because the ACIR API expects proofs without public inputs
        std::vector<bb::fr> inner_public_input_values(
            proof_witnesses.begin(), proof_witnesses.begin() + static_cast<std::ptrdiff_t>(num_inner_public_inputs));

        // We want to make sure that we do not remove the nested aggregation object in the case of the proof we want to
        // recursively verify contains a recursive proof itself. We are safe to keep all the inner public inputs
        // as in these tests the outer circuits do not have public inputs themselves
        if (!has_nested_proof) {
            proof_witnesses.erase(proof_witnesses.begin(),
                                  proof_witnesses.begin() + static_cast<std::ptrdiff_t>(num_inner_public_inputs));
        }

        const std::vector<bb::fr> key_witnesses = export_key_in_recursion_format(inner_verifier.key);

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
        // In the case of a nested proof we keep the nested aggregation object attached to the proof,
        // thus we do not explicitly have to keep the public inputs while setting up the initial recursion constraint.
        // They will later be attached as public inputs when creating the circuit.
        if (!has_nested_proof) {
            for (size_t i = 0; i < num_inner_public_inputs; ++i) {
                inner_public_inputs.push_back(static_cast<uint32_t>(i + public_input_start_idx));
            }
        }

        RecursionConstraint recursion_constraint{
            .key = key_indices,
            .proof = proof_indices,
            .public_inputs = inner_public_inputs,
            .key_hash = key_hash_start_idx,
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

        // Set the values for the inner public inputs
        // Note: this is confusing, but we minus one here due to the fact that the
        // witness values have not taken into account that zero is taken up by the zero_idx
        //
        // We once again have to check whether we have a nested proof, because if we do have one
        // then we could get a segmentation fault as `inner_public_inputs` was never filled with values.
        if (!has_nested_proof) {
            for (size_t i = 0; i < num_inner_public_inputs; ++i) {
                witness[inner_public_inputs[i]] = inner_public_input_values[i];
            }
        }

        witness_offset = key_indices_start_idx + key_witnesses.size();
        circuit_idx++;
    }

    acir_format constraint_system{ .varnum = static_cast<uint32_t>(witness.size()),
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
                                   .recursion_constraints = recursion_constraints,
                                   .constraints = {},
                                   .block_constraints = {} };

    auto outer_circuit = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    return outer_circuit;
}

TEST_F(AcirRecursionConstraint, TestBasicDoubleRecursionConstraints)
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

TEST_F(AcirRecursionConstraint, TestOneOuterRecursiveCircuit)
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

TEST_F(AcirRecursionConstraint, TestFullRecursiveComposition)
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
