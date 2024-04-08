#include "./eccvm_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

/**
 * @brief This function verifies an ECCVM Honk proof for given program settings.
 */
bool ECCVMVerifier::verify_proof(const HonkProof& proof)
{
    using ZeroMorph = ZeroMorphVerifier_<PCS>;

    RelationParameters<FF> relation_parameters;
    transcript = std::make_shared<Transcript>(proof);
    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Utility for extracting commitments from transcript
    const auto receive_commitment = [&](const std::string& label) {
        return transcript->template receive_from_prover<Commitment>(label);
    };

    // Get commitments to VM wires
    commitments.transcript_add = receive_commitment(commitment_labels.transcript_add);
    commitments.transcript_mul = receive_commitment(commitment_labels.transcript_mul);
    commitments.transcript_eq = receive_commitment(commitment_labels.transcript_eq);
    commitments.transcript_collision_check = receive_commitment(commitment_labels.transcript_collision_check);
    commitments.transcript_msm_transition = receive_commitment(commitment_labels.transcript_msm_transition);
    commitments.transcript_pc = receive_commitment(commitment_labels.transcript_pc);
    commitments.transcript_msm_count = receive_commitment(commitment_labels.transcript_msm_count);
    commitments.transcript_Px = receive_commitment(commitment_labels.transcript_Px);
    commitments.transcript_Py = receive_commitment(commitment_labels.transcript_Py);
    commitments.transcript_z1 = receive_commitment(commitment_labels.transcript_z1);
    commitments.transcript_z2 = receive_commitment(commitment_labels.transcript_z2);
    commitments.transcript_z1zero = receive_commitment(commitment_labels.transcript_z1zero);
    commitments.transcript_z2zero = receive_commitment(commitment_labels.transcript_z2zero);
    commitments.transcript_op = receive_commitment(commitment_labels.transcript_op);
    commitments.transcript_accumulator_x = receive_commitment(commitment_labels.transcript_accumulator_x);
    commitments.transcript_accumulator_y = receive_commitment(commitment_labels.transcript_accumulator_y);
    commitments.transcript_msm_x = receive_commitment(commitment_labels.transcript_msm_x);
    commitments.transcript_msm_y = receive_commitment(commitment_labels.transcript_msm_y);
    commitments.precompute_pc = receive_commitment(commitment_labels.precompute_pc);
    commitments.precompute_point_transition = receive_commitment(commitment_labels.precompute_point_transition);
    commitments.precompute_round = receive_commitment(commitment_labels.precompute_round);
    commitments.precompute_scalar_sum = receive_commitment(commitment_labels.precompute_scalar_sum);
    commitments.precompute_s1hi = receive_commitment(commitment_labels.precompute_s1hi);
    commitments.precompute_s1lo = receive_commitment(commitment_labels.precompute_s1lo);
    commitments.precompute_s2hi = receive_commitment(commitment_labels.precompute_s2hi);
    commitments.precompute_s2lo = receive_commitment(commitment_labels.precompute_s2lo);
    commitments.precompute_s3hi = receive_commitment(commitment_labels.precompute_s3hi);
    commitments.precompute_s3lo = receive_commitment(commitment_labels.precompute_s3lo);
    commitments.precompute_s4hi = receive_commitment(commitment_labels.precompute_s4hi);
    commitments.precompute_s4lo = receive_commitment(commitment_labels.precompute_s4lo);
    commitments.precompute_skew = receive_commitment(commitment_labels.precompute_skew);
    commitments.precompute_dx = receive_commitment(commitment_labels.precompute_dx);
    commitments.precompute_dy = receive_commitment(commitment_labels.precompute_dy);
    commitments.precompute_tx = receive_commitment(commitment_labels.precompute_tx);
    commitments.precompute_ty = receive_commitment(commitment_labels.precompute_ty);
    commitments.msm_transition = receive_commitment(commitment_labels.msm_transition);
    commitments.msm_add = receive_commitment(commitment_labels.msm_add);
    commitments.msm_double = receive_commitment(commitment_labels.msm_double);
    commitments.msm_skew = receive_commitment(commitment_labels.msm_skew);
    commitments.msm_accumulator_x = receive_commitment(commitment_labels.msm_accumulator_x);
    commitments.msm_accumulator_y = receive_commitment(commitment_labels.msm_accumulator_y);
    commitments.msm_pc = receive_commitment(commitment_labels.msm_pc);
    commitments.msm_size_of_msm = receive_commitment(commitment_labels.msm_size_of_msm);
    commitments.msm_count = receive_commitment(commitment_labels.msm_count);
    commitments.msm_round = receive_commitment(commitment_labels.msm_round);
    commitments.msm_add1 = receive_commitment(commitment_labels.msm_add1);
    commitments.msm_add2 = receive_commitment(commitment_labels.msm_add2);
    commitments.msm_add3 = receive_commitment(commitment_labels.msm_add3);
    commitments.msm_add4 = receive_commitment(commitment_labels.msm_add4);
    commitments.msm_x1 = receive_commitment(commitment_labels.msm_x1);
    commitments.msm_y1 = receive_commitment(commitment_labels.msm_y1);
    commitments.msm_x2 = receive_commitment(commitment_labels.msm_x2);
    commitments.msm_y2 = receive_commitment(commitment_labels.msm_y2);
    commitments.msm_x3 = receive_commitment(commitment_labels.msm_x3);
    commitments.msm_y3 = receive_commitment(commitment_labels.msm_y3);
    commitments.msm_x4 = receive_commitment(commitment_labels.msm_x4);
    commitments.msm_y4 = receive_commitment(commitment_labels.msm_y4);
    commitments.msm_collision_x1 = receive_commitment(commitment_labels.msm_collision_x1);
    commitments.msm_collision_x2 = receive_commitment(commitment_labels.msm_collision_x2);
    commitments.msm_collision_x3 = receive_commitment(commitment_labels.msm_collision_x3);
    commitments.msm_collision_x4 = receive_commitment(commitment_labels.msm_collision_x4);
    commitments.msm_lambda1 = receive_commitment(commitment_labels.msm_lambda1);
    commitments.msm_lambda2 = receive_commitment(commitment_labels.msm_lambda2);
    commitments.msm_lambda3 = receive_commitment(commitment_labels.msm_lambda3);
    commitments.msm_lambda4 = receive_commitment(commitment_labels.msm_lambda4);
    commitments.msm_slice1 = receive_commitment(commitment_labels.msm_slice1);
    commitments.msm_slice2 = receive_commitment(commitment_labels.msm_slice2);
    commitments.msm_slice3 = receive_commitment(commitment_labels.msm_slice3);
    commitments.msm_slice4 = receive_commitment(commitment_labels.msm_slice4);
    commitments.transcript_accumulator_empty = receive_commitment(commitment_labels.transcript_accumulator_empty);
    commitments.transcript_reset_accumulator = receive_commitment(commitment_labels.transcript_reset_accumulator);
    commitments.precompute_select = receive_commitment(commitment_labels.precompute_select);
    commitments.lookup_read_counts_0 = receive_commitment(commitment_labels.lookup_read_counts_0);
    commitments.lookup_read_counts_1 = receive_commitment(commitment_labels.lookup_read_counts_1);

    // Get challenge for sorted list batching and wire four memory records
    auto [beta, gamma] = transcript->template get_challenges<FF>("beta", "gamma");

    relation_parameters.gamma = gamma;
    auto beta_sqr = beta * beta;
    relation_parameters.beta = beta;
    relation_parameters.beta_sqr = beta_sqr;
    relation_parameters.beta_cube = beta_sqr * beta;
    relation_parameters.eccvm_set_permutation_delta =
        gamma * (gamma + beta_sqr) * (gamma + beta_sqr + beta_sqr) * (gamma + beta_sqr + beta_sqr + beta_sqr);
    relation_parameters.eccvm_set_permutation_delta = relation_parameters.eccvm_set_permutation_delta.invert();

    // Get commitment to permutation and lookup grand products
    commitments.lookup_inverses = receive_commitment(commitment_labels.lookup_inverses);
    commitments.z_perm = receive_commitment(commitment_labels.z_perm);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    bool multivariate_opening_verified = ZeroMorph::verify(commitments.get_unshifted(),
                                                           commitments.get_to_be_shifted(),
                                                           claimed_evaluations.get_unshifted(),
                                                           claimed_evaluations.get_shifted(),
                                                           multivariate_challenge,
                                                           key->pcs_verification_key,
                                                           transcript);
    // Execute transcript consistency univariate opening round
    // TODO(#768): Find a better way to do this. See issue for details.
    bool univariate_opening_verified = false;
    {
        auto hack_commitment = receive_commitment("Translation:hack_commitment");

        FF evaluation_challenge_x = transcript->template get_challenge<FF>("Translation:evaluation_challenge_x");

        // Construct arrays of commitments and evaluations to be batched
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

        // Get another challenge for batching the univariate claims
        FF ipa_batching_challenge = transcript->template get_challenge<FF>("Translation:ipa_batching_challenge");

        // Construct batched commitment and batched evaluation
        auto batched_commitment = transcript_commitments[0];
        auto batched_transcript_eval = transcript_evaluations[0];
        auto batching_scalar = ipa_batching_challenge;
        for (size_t idx = 1; idx < transcript_commitments.size(); ++idx) {
            batched_commitment = batched_commitment + transcript_commitments[idx] * batching_scalar;
            batched_transcript_eval += batching_scalar * transcript_evaluations[idx];
            batching_scalar *= ipa_batching_challenge;
        }

        // Construct and verify batched opening claim
        OpeningClaim<Curve> batched_univariate_claim = { { evaluation_challenge_x, batched_transcript_eval },
                                                         batched_commitment };
        univariate_opening_verified =
            PCS::reduce_verify(key->pcs_verification_key, batched_univariate_claim, transcript);
    }

    return sumcheck_verified.value() && multivariate_opening_verified && univariate_opening_verified;
}
} // namespace bb
