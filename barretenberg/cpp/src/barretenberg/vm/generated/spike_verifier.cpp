

#include "./spike_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

SpikeVerifier::SpikeVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

SpikeVerifier::SpikeVerifier(SpikeVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

SpikeVerifier& SpikeVerifier::operator=(SpikeVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

using FF = SpikeFlavor::FF;

// Evaluate the given public input column over the multivariate challenge points
[[maybe_unused]] FF evaluate_public_input_column(std::vector<FF> points, std::vector<FF> challenges)
{
    Polynomial<FF> polynomial(points);
    return polynomial.evaluate_mle(challenges);
}

/**
 * @brief This function verifies an Spike Honk proof for given program settings.
 *
 */
bool SpikeVerifier::verify_proof(const HonkProof& proof, const std::vector<FF>& public_inputs)
{
    using Flavor = SpikeFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using PCS = Flavor::PCS;
    // using ZeroMorph = ZeroMorphVerifier_<PCS>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.Spike_kernel_inputs =
        transcript->template receive_from_prover<Commitment>(commitment_labels.Spike_kernel_inputs);
    commitments.Spike_x = transcript->template receive_from_prover<Commitment>(commitment_labels.Spike_x);

    // Get commitments to inverses

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    FF public_column_evaluation = evaluate_public_input_column(public_inputs, multivariate_challenge);
    if (public_column_evaluation != claimed_evaluations.Spike_kernel_inputs) {
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    // NOTE: temporarily disabled - facing integration issues
    // auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
    //                                         commitments.get_to_be_shifted(),
    //                                         claimed_evaluations.get_unshifted(),
    //                                         claimed_evaluations.get_shifted(),
    //                                         multivariate_challenge,
    //                                         transcript);

    // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
    // return sumcheck_verified.value() && verified;
    return sumcheck_verified.value();
}

} // namespace bb
