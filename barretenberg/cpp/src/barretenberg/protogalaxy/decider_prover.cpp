#include "decider_prover.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::honk {

/**
 * Create DeciderProver_ from an accumulator.
 *
 * @param accumulator Relaxed instance (ϕ, ω, \vec{β}, e) whose proof we want to generate, produced by Protogalaxy
 * folding prover
 *
 * @tparam a type of UltraFlavor
 * */
template <UltraFlavor Flavor>
DeciderProver_<Flavor>::DeciderProver_(const std::shared_ptr<Instance>& inst,
                                       const std::shared_ptr<CommitmentKey>& commitment_key,
                                       const std::shared_ptr<Transcript>& transcript)
    : accumulator(std::move(inst))
    , transcript(transcript)
    , commitment_key(commitment_key)
{}

/**
 * @brief Add  ϕ, \vec{β}, e to the transcript. These are produced in the last round of folding that was carried out
 * before deciding.
 */
template <UltraFlavor Flavor> void DeciderProver_<Flavor>::execute_preamble_round()
{
    const auto accumulator_size = static_cast<uint32_t>(accumulator->instance_size);
    const auto num_public_inputs = static_cast<uint32_t>(accumulator->public_inputs.size());
    transcript->send_to_verifier("accumulator_size", accumulator_size);
    transcript->send_to_verifier("public_input_size", num_public_inputs);

    for (size_t i = 0; i < accumulator->public_inputs.size(); ++i) {
        auto public_input_i = accumulator->public_inputs[i];
        transcript->send_to_verifier("public_input_" + std::to_string(i), public_input_i);
    }

    transcript->send_to_verifier("eta", accumulator->relation_parameters.eta);
    transcript->send_to_verifier("beta", accumulator->relation_parameters.beta);
    transcript->send_to_verifier("gamma", accumulator->relation_parameters.gamma);
    transcript->send_to_verifier("public_input_delta", accumulator->relation_parameters.public_input_delta);
    transcript->send_to_verifier("lookup_grand_product_delta",
                                 accumulator->relation_parameters.lookup_grand_product_delta);
    size_t alpha_idx = 0;
    for (auto alpha : accumulator->alphas) {
        transcript->send_to_verifier("alpha_" + std::to_string(alpha_idx), alpha);
    }

    transcript->send_to_verifier("target_sum", accumulator->target_sum);
    for (size_t idx = 0; idx < accumulator->gate_challenges.size(); idx++) {
        transcript->send_to_verifier("gate_challenge_" + std::to_string(idx), accumulator->gate_challenges[idx]);
    }

    auto comm_view = accumulator->witness_commitments.get_all();
    auto witness_labels = accumulator->commitment_labels.get_witness();
    for (size_t idx = 0; idx < witness_labels.size(); idx++) {
        transcript->send_to_verifier(witness_labels[idx], comm_view[idx]);
    }

    auto vk_view = accumulator->verification_key->get_all();
    auto vk_labels = accumulator->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        transcript->send_to_verifier(vk_labels[idx], vk_view[idx]);
    }
}

/**
 * @brief Run Sumcheck to establish that ∑_i pow(\vec{β*})f_i(ω) = e*. This results in u = (u_1,...,u_d) sumcheck round
 * challenges and all evaluations at u being calculated.
 *
 */
template <UltraFlavor Flavor> void DeciderProver_<Flavor>::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;
    auto instance_size = accumulator->instance_size;
    auto sumcheck = Sumcheck(instance_size, transcript);
    sumcheck_output = sumcheck.prove(accumulator);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
template <UltraFlavor Flavor> void DeciderProver_<Flavor>::execute_zeromorph_rounds()
{
    ZeroMorph::prove(accumulator->prover_polynomials.get_unshifted(),
                     accumulator->prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript);
}

template <UltraFlavor Flavor> plonk::proof& DeciderProver_<Flavor>::export_proof()
{
    proof.proof_data = transcript->proof_data;
    return proof;
}

template <UltraFlavor Flavor> plonk::proof& DeciderProver_<Flavor>::construct_proof()
{
    // Add ϕ, \vec{β*}, e* to transcript
    execute_preamble_round();

    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_zeromorph_rounds();

    return export_proof();
}

template class DeciderProver_<honk::flavor::Ultra>;
template class DeciderProver_<honk::flavor::GoblinUltra>;

} // namespace bb::honk
