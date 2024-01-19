#include "decider_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {

template <typename Flavor>
DeciderVerifier_<Flavor>::DeciderVerifier_(const std::shared_ptr<Transcript>& transcript,
                                           const std::shared_ptr<VerificationKey>& verifier_key)
    : key(verifier_key)
    , transcript(transcript)
{}
template <typename Flavor>
DeciderVerifier_<Flavor>::DeciderVerifier_()
    : pcs_verification_key(std::make_unique<VerifierCommitmentKey>(0, bb::srs::get_crs_factory()))
    , transcript(std::make_shared<Transcript>())
{}

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor, produced for a relaxed instance (ϕ, \vec{β*},
 * e*).
 *
 */
template <typename Flavor> bool DeciderVerifier_<Flavor>::verify_proof(const plonk::proof& proof)
{
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using Curve = typename Flavor::Curve;
    using ZeroMorph = pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using Instance = VerifierInstance_<Flavor>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;

    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;
    transcript = std::make_shared<Transcript>(proof.proof_data);
    auto inst = std::make_unique<Instance>();

    inst->instance_size = transcript->template receive_from_prover<uint32_t>("instance_size");
    inst->log_instance_size = static_cast<size_t>(numeric::get_msb(inst->instance_size));
    inst->public_input_size = transcript->template receive_from_prover<uint32_t>("public_input_size");

    for (size_t i = 0; i < inst->public_input_size; ++i) {
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

    inst->gate_challenges = std::vector<FF>(inst->log_instance_size);
    for (size_t idx = 0; idx < inst->log_instance_size; idx++) {
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

    auto sumcheck = SumcheckVerifier<Flavor>(inst->log_instance_size, transcript, inst->target_sum);

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(inst->relation_parameters, inst->alphas, inst->gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
                                            commitments.get_to_be_shifted(),
                                            claimed_evaluations.get_unshifted(),
                                            claimed_evaluations.get_shifted(),
                                            multivariate_challenge,
                                            transcript);

    auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);

    return sumcheck_verified.value() && verified;
}

template class DeciderVerifier_<honk::flavor::Ultra>;
template class DeciderVerifier_<honk::flavor::GoblinUltra>;

} // namespace bb::honk
