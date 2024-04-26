#include "goblin_translator_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

GoblinTranslatorProver::GoblinTranslatorProver(CircuitBuilder& circuit_builder,
                                               const std::shared_ptr<Transcript>& transcript)
    : dyadic_circuit_size(Flavor::compute_dyadic_circuit_size(circuit_builder))
    , mini_circuit_dyadic_size(Flavor::compute_mini_circuit_dyadic_size(circuit_builder))
    , transcript(transcript)
{
    BB_OP_COUNT_TIME();

    // Compute total number of gates, dyadic circuit size, etc.
    key = std::make_shared<ProvingKey>(circuit_builder);
    compute_witness(circuit_builder);
    compute_commitment_key(key->circuit_size);
}

/**
 * @brief Compute witness polynomials
 *
 */
void GoblinTranslatorProver::compute_witness(CircuitBuilder& circuit_builder)
{
    if (computed_witness) {
        return;
    }

    // Populate the wire polynomials from the wire vectors in the circuit constructor. Note: In goblin translator wires
    // come as is, since they have to reflect the structure of polynomials in the first 4 wires, which we've commited to
    for (auto [wire_poly, wire] : zip_view(key->polynomials.get_wires(), circuit_builder.wires)) {
        for (size_t i = 0; i < circuit_builder.num_gates; ++i) {
            wire_poly[i] = circuit_builder.get_variable(wire[i]);
        }
    }

    // We construct concatenated versions of range constraint polynomials, where several polynomials are concatenated
    // into one. These polynomials are not commited to.
    bb::compute_concatenated_polynomials<Flavor>(key->polynomials);

    // We also contruct ordered polynomials, which have the same values as concatenated ones + enough values to bridge
    // the range from 0 to maximum range defined by the range constraint.
    bb::compute_goblin_translator_range_constraint_ordered_polynomials<Flavor>(key->polynomials,
                                                                               mini_circuit_dyadic_size);

    computed_witness = true;
}

std::shared_ptr<GoblinTranslatorProver::CommitmentKey> GoblinTranslatorProver::compute_commitment_key(
    size_t circuit_size)
{
    if (commitment_key) {
        return commitment_key;
    }

    commitment_key = std::make_shared<CommitmentKey>(circuit_size);
    return commitment_key;
};

/**
 * @brief Add circuit size and values used in the relations to the transcript
 *
 */
void GoblinTranslatorProver::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);
    const auto SHIFT = uint256_t(1) << Flavor::NUM_LIMB_BITS;
    const auto SHIFTx2 = uint256_t(1) << (Flavor::NUM_LIMB_BITS * 2);
    const auto SHIFTx3 = uint256_t(1) << (Flavor::NUM_LIMB_BITS * 3);
    const auto accumulated_result = BF(uint256_t(key->polynomials.accumulators_binary_limbs_0[1]) +
                                       uint256_t(key->polynomials.accumulators_binary_limbs_1[1]) * SHIFT +
                                       uint256_t(key->polynomials.accumulators_binary_limbs_2[1]) * SHIFTx2 +
                                       uint256_t(key->polynomials.accumulators_binary_limbs_3[1]) * SHIFTx3);
    transcript->send_to_verifier("circuit_size", circuit_size);
    transcript->send_to_verifier("evaluation_input_x", key->evaluation_input_x);
    transcript->send_to_verifier("accumulated_result", accumulated_result);
}

/**
 * @brief Compute commitments to the first three wires
 *
 */
void GoblinTranslatorProver::execute_wire_and_sorted_constraints_commitments_round()
{
    // Commit to all wire polynomials and ordered range constraint polynomials
    auto wire_polys = key->polynomials.get_wires_and_ordered_range_constraints();
    auto labels = commitment_labels.get_wires_and_ordered_range_constraints();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

/**
 * @brief Compute permutation product polynomial and commitments
 *
 */
void GoblinTranslatorProver::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    FF gamma = transcript->template get_challenge<FF>("gamma");
    const size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
    relation_parameters.beta = 0;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = 0;
    relation_parameters.lookup_grand_product_delta = 0;
    auto uint_evaluation_input = uint256_t(key->evaluation_input_x);
    relation_parameters.evaluation_input_x = { uint_evaluation_input.slice(0, NUM_LIMB_BITS),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                               uint_evaluation_input };

    relation_parameters.accumulated_result = { key->polynomials.accumulators_binary_limbs_0[1],
                                               key->polynomials.accumulators_binary_limbs_1[1],
                                               key->polynomials.accumulators_binary_limbs_2[1],
                                               key->polynomials.accumulators_binary_limbs_3[1] };

    std::vector<uint256_t> uint_batching_challenge_powers;
    auto batching_challenge_v = key->batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(batching_challenge_v);
    auto running_power = batching_challenge_v * batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);
    running_power *= batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);
    running_power *= batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);

    for (size_t i = 0; i < 4; i++) {
        relation_parameters.batching_challenge_v[i] = {
            uint_batching_challenge_powers[i].slice(0, NUM_LIMB_BITS),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
            uint_batching_challenge_powers[i]
        };
    }
    // Compute constraint permutation grand product
    compute_grand_products<Flavor>(key->polynomials, relation_parameters);

    transcript->send_to_verifier(commitment_labels.z_perm, commitment_key->commit(key->polynomials.z_perm));
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void GoblinTranslatorProver::execute_relation_check_rounds()
{
    using Sumcheck = SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    sumcheck_output = sumcheck.prove(key->polynomials, relation_parameters, alpha, gate_challenges);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
void GoblinTranslatorProver::execute_zeromorph_rounds()
{
    using ZeroMorph = ZeroMorphProver_<PCS>;
    ZeroMorph::prove(key->polynomials.get_unshifted_without_concatenated(),
                     key->polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted_without_concatenated(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript,
                     key->polynomials.get_concatenated_constraints(),
                     sumcheck_output.claimed_evaluations.get_concatenated_constraints(),
                     key->polynomials.get_concatenation_groups());
}

HonkProof& GoblinTranslatorProver::export_proof()
{
    proof = transcript->export_proof();
    return proof;
}

HonkProof& GoblinTranslatorProver::construct_proof()
{
    BB_OP_COUNT_TIME_NAME("GoblinTranslatorProver::construct_proof");

    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute first three wire commitments
    execute_wire_and_sorted_constraints_commitments_round();

    // Fiat-Shamir: gamma
    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_zeromorph_rounds();

    return export_proof();
}

} // namespace bb
