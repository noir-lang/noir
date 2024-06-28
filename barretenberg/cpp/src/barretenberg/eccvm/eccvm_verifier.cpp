#include "./eccvm_verifier.hpp"
#include "barretenberg/commitment_schemes/shplonk/shplonk.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

/**
 * @brief This function verifies an ECCVM Honk proof for given program settings.
 */
bool ECCVMVerifier::verify_proof(const HonkProof& proof)
{
    using Curve = typename Flavor::Curve;
    using ZeroMorph = ZeroMorphVerifier_<Curve>;
    using Shplonk = ShplonkVerifier_<Curve>;

    RelationParameters<FF> relation_parameters;
    transcript = std::make_shared<Transcript>(proof);
    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");
    ASSERT(circuit_size == key->circuit_size);

    for (auto [comm, label] : zip_view(commitments.get_wires(), commitment_labels.get_wires())) {
        comm = transcript->template receive_from_prover<Commitment>(label);
    }

    // Get challenge for sorted list batching and wire four memory records
    auto [beta, gamma] = transcript->template get_challenges<FF>("beta", "gamma");

    auto beta_sqr = beta * beta;
    relation_parameters.gamma = gamma;
    relation_parameters.beta = beta;
    relation_parameters.beta_sqr = beta * beta;
    relation_parameters.beta_cube = beta_sqr * beta;
    relation_parameters.eccvm_set_permutation_delta =
        gamma * (gamma + beta_sqr) * (gamma + beta_sqr + beta_sqr) * (gamma + beta_sqr + beta_sqr + beta_sqr);
    relation_parameters.eccvm_set_permutation_delta = relation_parameters.eccvm_set_permutation_delta.invert();

    // Get commitment to permutation and lookup grand products
    commitments.lookup_inverses =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_inverses);
    commitments.z_perm = transcript->template receive_from_prover<Commitment>(commitment_labels.z_perm);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(static_cast<size_t>(numeric::get_msb(key->circuit_size)));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Reduce the multivariate evaluation claims produced by sumcheck to a single univariate opening claim
    auto multivariate_to_univariate_opening_claim = ZeroMorph::verify(circuit_size,
                                                                      commitments.get_unshifted(),
                                                                      commitments.get_to_be_shifted(),
                                                                      claimed_evaluations.get_unshifted(),
                                                                      claimed_evaluations.get_shifted(),
                                                                      multivariate_challenge,
                                                                      key->pcs_verification_key->get_g1_identity(),
                                                                      transcript);

    // Execute transcript consistency univariate opening round
    auto hack_commitment = transcript->template receive_from_prover<Commitment>("Translation:hack_commitment");

    FF evaluation_challenge_x = transcript->template get_challenge<FF>("Translation:evaluation_challenge_x");

    // Construct arrays of commitments and evaluations to be batched, the evaluations being received from the prover
    const size_t NUM_UNIVARIATES = 6;
    std::array<Commitment, NUM_UNIVARIATES> transcript_commitments = {
        commitments.transcript_op, commitments.transcript_Px, commitments.transcript_Py,
        commitments.transcript_z1, commitments.transcript_z2, hack_commitment
    };
    std::array<FF, NUM_UNIVARIATES> transcript_evaluations = {
        transcript->template receive_from_prover<FF>("Translation:op"),
        transcript->template receive_from_prover<FF>("Translation:Px"),
        transcript->template receive_from_prover<FF>("Translation:Py"),
        transcript->template receive_from_prover<FF>("Translation:z1"),
        transcript->template receive_from_prover<FF>("Translation:z2"),
        transcript->template receive_from_prover<FF>("Translation:hack_evaluation")
    };

    // Get the batching challenge for commitments and evaluations
    FF ipa_batching_challenge = transcript->template get_challenge<FF>("Translation:ipa_batching_challenge");

    // Compute the batched commitment and batched evaluation for the univariate opening claim
    auto batched_commitment = transcript_commitments[0];
    auto batched_transcript_eval = transcript_evaluations[0];
    auto batching_scalar = ipa_batching_challenge;
    for (size_t idx = 1; idx < transcript_commitments.size(); ++idx) {
        batched_commitment = batched_commitment + transcript_commitments[idx] * batching_scalar;
        batched_transcript_eval += batching_scalar * transcript_evaluations[idx];
        batching_scalar *= ipa_batching_challenge;
    }

    std::array<OpeningClaim<Curve>, 2> opening_claims = { multivariate_to_univariate_opening_claim,
                                                          { { evaluation_challenge_x, batched_transcript_eval },
                                                            batched_commitment } };

    // Construct and verify the combined opening claim
    auto batched_opening_claim =
        Shplonk::reduce_verification(key->pcs_verification_key->get_g1_identity(), opening_claims, transcript);

    bool batched_opening_verified = PCS::reduce_verify(key->pcs_verification_key, batched_opening_claim, transcript);

    return sumcheck_verified.value() && batched_opening_verified;
}
} // namespace bb
