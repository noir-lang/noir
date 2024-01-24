#include "barretenberg/stdlib/recursion/honk/verifier/decider_recursive_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Flavor>
DeciderRecursiveVerifier_<Flavor>::DeciderRecursiveVerifier_(Builder* builder)
    : builder(builder)
{}

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor, produced for a relaxed instance (ϕ, \vec{β*},
 * e*).
 *
 */
template <typename Flavor>
std::array<typename Flavor::GroupElement, 2> DeciderRecursiveVerifier_<Flavor>::verify_proof(
    const bb::plonk::proof& proof)
{
    using Sumcheck = ::bb::honk::sumcheck::SumcheckVerifier<Flavor>;
    using Curve = typename Flavor::Curve;
    using ZeroMorph = ::bb::honk::pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using Transcript = typename Flavor::Transcript;
    using Instance = typename ::bb::honk::VerifierInstance_<Flavor>;

    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;
    transcript = std::make_shared<Transcript>(builder, proof.proof_data);
    auto inst = std::make_unique<Instance>();

    const auto instance_size = transcript->template receive_from_prover<uint32_t>("instance_size");
    const auto public_input_size = transcript->template receive_from_prover<uint32_t>("public_input_size");
    const auto log_instance_size = static_cast<size_t>(numeric::get_msb(uint32_t(instance_size.get_value())));

    for (size_t i = 0; i < uint32_t(public_input_size.get_value()); ++i) {
        auto public_input_i = transcript->template receive_from_prover<FF>("public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    auto eta = transcript->template receive_from_prover<FF>("eta");
    auto beta = transcript->template receive_from_prover<FF>("beta");
    auto gamma = transcript->template receive_from_prover<FF>("gamma");
    auto public_input_delta = transcript->template receive_from_prover<FF>("public_input_delta");
    auto lookup_grand_product_delta = transcript->template receive_from_prover<FF>("lookup_grand_product_delta");
    inst->relation_parameters =
        RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };

    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] = transcript->template receive_from_prover<FF>("alpha" + std::to_string(idx));
    }

    inst->target_sum = transcript->template receive_from_prover<FF>("target_sum");

    inst->gate_challenges = std::vector<FF>(log_instance_size);
    for (size_t idx = 0; idx < log_instance_size; idx++) {
        inst->gate_challenges[idx] =
            transcript->template receive_from_prover<FF>("gate_challenge_" + std::to_string(idx));
    }
    auto comm_view = inst->witness_commitments.get_all();
    auto witness_labels = inst->commitment_labels.get_witness();
    for (size_t idx = 0; idx < witness_labels.size(); idx++) {
        comm_view[idx] = transcript->template receive_from_prover<Commitment>(witness_labels[idx]);
    }

    inst->verification_key = std::make_shared<VerificationKey>(inst->instance_size, inst->public_input_size);
    auto vk_view = inst->verification_key->get_all();
    auto vk_labels = inst->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        vk_view[idx] = transcript->template receive_from_prover<Commitment>(vk_labels[idx]);
    }

    VerifierCommitments commitments{ inst->verification_key, inst->witness_commitments };

    auto sumcheck = Sumcheck(log_instance_size, transcript, inst->target_sum);

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(inst->relation_parameters, inst->alphas, inst->gate_challenges);

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
                                            commitments.get_to_be_shifted(),
                                            claimed_evaluations.get_unshifted(),
                                            claimed_evaluations.get_shifted(),
                                            multivariate_challenge,
                                            transcript);

    return pairing_points;
}

template class DeciderRecursiveVerifier_<bb::honk::flavor::UltraRecursive_<GoblinUltraCircuitBuilder>>;
template class DeciderRecursiveVerifier_<bb::honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>>;
} // namespace bb::stdlib::recursion::honk
