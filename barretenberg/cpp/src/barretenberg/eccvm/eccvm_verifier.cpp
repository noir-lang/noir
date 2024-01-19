#include "./eccvm_verifier.hpp"
#include "barretenberg/commitment_schemes/gemini/gemini.hpp"
#include "barretenberg/commitment_schemes/shplonk/shplonk.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {
template <typename Flavor>
ECCVMVerifier_<Flavor>::ECCVMVerifier_(const std::shared_ptr<typename Flavor::VerificationKey>& verifier_key)
    : key(verifier_key)
{}

template <typename Flavor>
ECCVMVerifier_<Flavor>::ECCVMVerifier_(ECCVMVerifier_&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

template <typename Flavor> ECCVMVerifier_<Flavor>& ECCVMVerifier_<Flavor>::operator=(ECCVMVerifier_&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    pcs_fr_elements.clear();
    return *this;
}

/**
 * @brief This function verifies an ECCVM Honk proof for given program settings.
 *
 */
template <typename Flavor> bool ECCVMVerifier_<Flavor>::verify_proof(const plonk::proof& proof)
{
    using FF = typename Flavor::FF;
    using GroupElement = typename Flavor::GroupElement;
    using Commitment = typename Flavor::Commitment;
    using PCS = typename Flavor::PCS;
    using Curve = typename Flavor::Curve;
    using Gemini = pcs::gemini::GeminiVerifier_<Curve>;
    using Shplonk = pcs::shplonk::ShplonkVerifier_<Curve>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Transcript = typename Flavor::Transcript;
    using OpeningClaim = typename pcs::OpeningClaim<Curve>;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof.proof_data);

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
    auto [beta, gamma] = challenges_to_field_elements<FF>(transcript->get_challenges("beta", "gamma"));

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
    FF alpha = transcript->get_challenge("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, purported_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Execute Gemini/Shplonk verification:

    // Construct inputs for Gemini verifier:
    // - Multivariate opening point u = (u_0, ..., u_{d-1})
    // - batched unshifted and to-be-shifted polynomial commitments
    auto batched_commitment_unshifted = GroupElement::zero();
    auto batched_commitment_to_be_shifted = GroupElement::zero();
    const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    // Compute powers of batching challenge rho
    FF rho = transcript->get_challenge("rho");
    std::vector<FF> rhos = pcs::gemini::powers_of_rho(rho, NUM_POLYNOMIALS);

    // Compute batched multivariate evaluation
    FF batched_evaluation = FF::zero();
    size_t evaluation_idx = 0;
    for (auto& value : purported_evaluations.get_unshifted()) {
        batched_evaluation += value * rhos[evaluation_idx];
        ++evaluation_idx;
    }
    for (auto& value : purported_evaluations.get_shifted()) {
        batched_evaluation += value * rhos[evaluation_idx];
        ++evaluation_idx;
    }

    // Construct batched commitment for NON-shifted polynomials
    size_t commitment_idx = 0;
    for (auto& commitment : commitments.get_unshifted()) {
        // TODO(@zac-williamson)(https://github.com/AztecProtocol/barretenberg/issues/820) ensure ECCVM polynomial
        // commitments are never points at infinity
        if (commitment.y != 0) {
            batched_commitment_unshifted += commitment * rhos[commitment_idx];
        } else {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/820)
        }
        ++commitment_idx;
    }

    // Construct batched commitment for to-be-shifted polynomials
    for (auto& commitment : commitments.get_to_be_shifted()) {
        // TODO(@zac-williamson) ensure ECCVM polynomial commitments are never points at infinity (#2214)
        if (commitment.y != 0) {
            batched_commitment_to_be_shifted += commitment * rhos[commitment_idx];
        } else {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/820)
        }
        ++commitment_idx;
    }

    // Produce a Gemini claim consisting of:
    // - d+1 commitments [Fold_{r}^(0)], [Fold_{-r}^(0)], and [Fold^(l)], l = 1:d-1
    // - d+1 evaluations a_0_pos, and a_l, l = 0:d-1
    auto gemini_claim = Gemini::reduce_verification(multivariate_challenge,
                                                    batched_evaluation,
                                                    batched_commitment_unshifted,
                                                    batched_commitment_to_be_shifted,
                                                    transcript);

    // Produce a Shplonk claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    auto shplonk_claim = Shplonk::reduce_verification(pcs_verification_key, gemini_claim, transcript);

    // Verify the Shplonk claim with KZG or IPA
    auto multivariate_opening_verified = PCS::verify(pcs_verification_key, shplonk_claim, transcript);

    // Execute transcript consistency univariate opening round
    // TODO(#768): Find a better way to do this. See issue for details.
    bool univariate_opening_verified = false;
    {
        auto hack_commitment = receive_commitment("Translation:hack_commitment");

        FF evaluation_challenge_x = transcript->get_challenge("Translation:evaluation_challenge_x");

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
        FF ipa_batching_challenge = transcript->get_challenge("Translation:ipa_batching_challenge");

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
        OpeningClaim batched_univariate_claim = { { evaluation_challenge_x, batched_transcript_eval },
                                                  batched_commitment };
        univariate_opening_verified = PCS::verify(pcs_verification_key, batched_univariate_claim, transcript);
    }

    return sumcheck_verified.value() && multivariate_opening_verified && univariate_opening_verified;
}

template class ECCVMVerifier_<honk::flavor::ECCVM>;

} // namespace bb::honk
