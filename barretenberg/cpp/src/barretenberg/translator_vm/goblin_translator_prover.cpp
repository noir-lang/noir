#include "goblin_translator_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::honk {

/**
 * Create GoblinTranslatorProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */

/**
 * Create GoblinTranslatorProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */

GoblinTranslatorProver::GoblinTranslatorProver(const std::shared_ptr<typename Flavor::ProvingKey>& input_key,
                                               const std::shared_ptr<CommitmentKey>& commitment_key,
                                               const std::shared_ptr<Transcript>& transcript)
    : transcript(transcript)
    , key(input_key)
    , commitment_key(commitment_key)
{
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == flavor_get_label(*key, key_poly));
        prover_poly = key_poly.share();
    }
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == flavor_get_label(*key, key_poly) + "_shift");
        prover_poly = key_poly.shifted();
    }
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/810): resolve weirdness around concatenated range
    // constraints
    prover_polynomials.concatenated_range_constraints_0 = key->concatenated_range_constraints_0;
    prover_polynomials.concatenated_range_constraints_1 = key->concatenated_range_constraints_1;
    prover_polynomials.concatenated_range_constraints_2 = key->concatenated_range_constraints_2;
    prover_polynomials.concatenated_range_constraints_3 = key->concatenated_range_constraints_3;
}

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
    const auto accumulated_result =
        BF(uint256_t(key->accumulators_binary_limbs_0[1]) + uint256_t(key->accumulators_binary_limbs_1[1]) * SHIFT +
           uint256_t(key->accumulators_binary_limbs_2[1]) * SHIFTx2 +
           uint256_t(key->accumulators_binary_limbs_3[1]) * SHIFTx3);
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
    // Commit to all wire polynomials
    auto wire_polys = key->get_wires();
    auto labels = commitment_labels.get_wires();
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
    FF gamma = transcript->get_challenge("gamma");
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

    relation_parameters.accumulated_result = { key->accumulators_binary_limbs_0[1],
                                               key->accumulators_binary_limbs_1[1],
                                               key->accumulators_binary_limbs_2[1],
                                               key->accumulators_binary_limbs_3[1] };

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
    grand_product_library::compute_grand_products<Flavor>(key, prover_polynomials, relation_parameters);

    transcript->send_to_verifier(commitment_labels.z_perm, commitment_key->commit(key->z_perm));
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void GoblinTranslatorProver::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);
    FF alpha = transcript->get_challenge("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha, gate_challenges);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
void GoblinTranslatorProver::execute_zeromorph_rounds()
{
    using ZeroMorph = pcs::zeromorph::ZeroMorphProver_<Curve>;
    ZeroMorph::prove(prover_polynomials.get_unshifted(),
                     prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript,
                     prover_polynomials.get_concatenated_constraints(),
                     sumcheck_output.claimed_evaluations.get_concatenated_constraints(),
                     prover_polynomials.get_concatenation_groups());
}

plonk::proof& GoblinTranslatorProver::export_proof()
{
    proof.proof_data = transcript->export_proof();
    return proof;
}

plonk::proof& GoblinTranslatorProver::construct_proof()
{
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

} // namespace bb::honk
