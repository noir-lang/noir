#include "./eccvm_verifier.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

using namespace barretenberg;
using namespace proof_system::honk::sumcheck;

namespace proof_system::honk {
template <typename Flavor>
ECCVMVerifier_<Flavor>::ECCVMVerifier_(std::shared_ptr<typename Flavor::VerificationKey> verifier_key)
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

    RelationParameters<FF> relation_parameters;

    transcript = VerifierTranscript<FF>{ proof.proof_data };

    auto commitments = VerifierCommitments(key, transcript);
    auto commitment_labels = CommitmentLabels();

    const auto circuit_size = transcript.template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.transcript_add = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_add);
    commitments.transcript_mul = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_mul);
    commitments.transcript_eq = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_eq);
    commitments.transcript_collision_check =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_collision_check);
    commitments.transcript_msm_transition =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_msm_transition);
    commitments.transcript_pc = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_pc);
    commitments.transcript_msm_count =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_msm_count);
    commitments.transcript_x = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_x);
    commitments.transcript_y = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_y);
    commitments.transcript_z1 = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_z1);
    commitments.transcript_z2 = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_z2);
    commitments.transcript_z1zero =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_z1zero);
    commitments.transcript_z2zero =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_z2zero);
    commitments.transcript_op = transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_op);
    commitments.transcript_accumulator_x =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_accumulator_x);
    commitments.transcript_accumulator_y =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_accumulator_y);
    commitments.transcript_msm_x =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_msm_x);
    commitments.transcript_msm_y =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_msm_y);
    commitments.precompute_pc = transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_pc);
    commitments.precompute_point_transition =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_point_transition);
    commitments.precompute_round =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_round);
    commitments.precompute_scalar_sum =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_scalar_sum);
    commitments.precompute_s1hi =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s1hi);
    commitments.precompute_s1lo =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s1lo);
    commitments.precompute_s2hi =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s2hi);
    commitments.precompute_s2lo =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s2lo);
    commitments.precompute_s3hi =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s3hi);
    commitments.precompute_s3lo =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s3lo);
    commitments.precompute_s4hi =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s4hi);
    commitments.precompute_s4lo =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_s4lo);
    commitments.precompute_skew =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_skew);
    commitments.precompute_dx = transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_dx);
    commitments.precompute_dy = transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_dy);
    commitments.precompute_tx = transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_tx);
    commitments.precompute_ty = transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_ty);
    commitments.msm_transition = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_transition);
    commitments.msm_add = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_add);
    commitments.msm_double = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_double);
    commitments.msm_skew = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_skew);
    commitments.msm_accumulator_x =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_accumulator_x);
    commitments.msm_accumulator_y =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_accumulator_y);
    commitments.msm_pc = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_pc);
    commitments.msm_size_of_msm =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_size_of_msm);
    commitments.msm_count = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_count);
    commitments.msm_round = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_round);
    commitments.msm_add1 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_add1);
    commitments.msm_add2 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_add2);
    commitments.msm_add3 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_add3);
    commitments.msm_add4 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_add4);
    commitments.msm_x1 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_x1);
    commitments.msm_y1 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_y1);
    commitments.msm_x2 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_x2);
    commitments.msm_y2 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_y2);
    commitments.msm_x3 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_x3);
    commitments.msm_y3 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_y3);
    commitments.msm_x4 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_x4);
    commitments.msm_y4 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_y4);
    commitments.msm_collision_x1 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_collision_x1);
    commitments.msm_collision_x2 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_collision_x2);
    commitments.msm_collision_x3 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_collision_x3);
    commitments.msm_collision_x4 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.msm_collision_x4);
    commitments.msm_lambda1 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_lambda1);
    commitments.msm_lambda2 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_lambda2);
    commitments.msm_lambda3 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_lambda3);
    commitments.msm_lambda4 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_lambda4);
    commitments.msm_slice1 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_slice1);
    commitments.msm_slice2 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_slice2);
    commitments.msm_slice3 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_slice3);
    commitments.msm_slice4 = transcript.template receive_from_prover<Commitment>(commitment_labels.msm_slice4);
    commitments.transcript_accumulator_empty =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_accumulator_empty);
    commitments.transcript_reset_accumulator =
        transcript.template receive_from_prover<Commitment>(commitment_labels.transcript_reset_accumulator);
    commitments.precompute_select =
        transcript.template receive_from_prover<Commitment>(commitment_labels.precompute_select);
    commitments.lookup_read_counts_0 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.lookup_read_counts_0);
    commitments.lookup_read_counts_1 =
        transcript.template receive_from_prover<Commitment>(commitment_labels.lookup_read_counts_1);

    // Get challenge for sorted list batching and wire four memory records
    auto [beta, gamma] = transcript.get_challenges("bbeta", "gamma");
    relation_parameters.gamma = gamma;
    auto beta_sqr = beta * beta;
    relation_parameters.beta = beta;
    relation_parameters.beta_sqr = beta_sqr;
    relation_parameters.beta_cube = beta_sqr * beta;
    relation_parameters.eccvm_set_permutation_delta =
        gamma * (gamma + beta_sqr) * (gamma + beta_sqr + beta_sqr) * (gamma + beta_sqr + beta_sqr + beta_sqr);
    relation_parameters.eccvm_set_permutation_delta = relation_parameters.eccvm_set_permutation_delta.invert();

    // Get commitment to permutation and lookup grand products
    commitments.lookup_inverses =
        transcript.template receive_from_prover<Commitment>(commitment_labels.lookup_inverses);
    commitments.z_perm = transcript.template receive_from_prover<Commitment>(commitment_labels.z_perm);

    // Execute Sumcheck Verifier
    auto sumcheck = SumcheckVerifier<Flavor>(circuit_size);

    std::optional sumcheck_output = sumcheck.verify(relation_parameters, transcript);

    // If Sumcheck does not return an output, sumcheck verification has failed
    if (!sumcheck_output.has_value()) {
        return false;
    }

    auto [multivariate_challenge, purported_evaluations] = *sumcheck_output;

    // Execute Gemini/Shplonk verification:

    // Construct inputs for Gemini verifier:
    // - Multivariate opening point u = (u_0, ..., u_{d-1})
    // - batched unshifted and to-be-shifted polynomial commitments
    auto batched_commitment_unshifted = GroupElement::zero();
    auto batched_commitment_to_be_shifted = GroupElement::zero();
    const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    // Compute powers of batching challenge rho
    FF rho = transcript.get_challenge("rho");
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
        // TODO(@zac-williamson) ensure ECCVM polynomial commitments are never points at infinity (#2214)
        if (commitment.y != 0) {
            batched_commitment_unshifted += commitment * rhos[commitment_idx];
        } else {
            info("point at infinity (unshifted)");
        }
        ++commitment_idx;
    }

    // Construct batched commitment for to-be-shifted polynomials
    for (auto& commitment : commitments.get_to_be_shifted()) {
        // TODO(@zac-williamson) ensure ECCVM polynomial commitments are never points at infinity (#2214)
        if (commitment.y != 0) {
            batched_commitment_to_be_shifted += commitment * rhos[commitment_idx];
        } else {
            info("point at infinity (to be shifted)");
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

    // // Verify the Shplonk claim with KZG or IPA
    return PCS::verify(pcs_verification_key, shplonk_claim, transcript);
}

template class ECCVMVerifier_<honk::flavor::ECCVM>;
template class ECCVMVerifier_<honk::flavor::ECCVMGrumpkin>;

} // namespace proof_system::honk
