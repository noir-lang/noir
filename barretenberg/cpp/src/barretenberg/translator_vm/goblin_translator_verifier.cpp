#include "./goblin_translator_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {

GoblinTranslatorVerifier::GoblinTranslatorVerifier(
    const std::shared_ptr<typename Flavor::VerificationKey>& verifier_key,
    const std::shared_ptr<Transcript>& transcript)
    : key(verifier_key)
    , transcript(transcript)
{}

GoblinTranslatorVerifier::GoblinTranslatorVerifier(GoblinTranslatorVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

GoblinTranslatorVerifier& GoblinTranslatorVerifier::operator=(GoblinTranslatorVerifier&& other) noexcept
{
    key = std::move(other.key);
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    pcs_fr_elements.clear();
    return *this;
}

void GoblinTranslatorVerifier::put_translation_data_in_relation_parameters(const uint256_t& evaluation_input_x,
                                                                           const BF& batching_challenge_v,
                                                                           const uint256_t& accumulated_result)
{

    const auto compute_four_limbs = [](const auto& in) {
        constexpr size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
        return std::array<FF, 4>{ in.slice(0, NUM_LIMB_BITS),
                                  in.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                  in.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                  in.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4) };
    };

    const auto compute_five_limbs = [](const auto& in) {
        constexpr size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
        return std::array<FF, 5>{ in.slice(0, NUM_LIMB_BITS),
                                  in.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                  in.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                  in.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                  in };
    };

    relation_parameters.evaluation_input_x = compute_five_limbs(evaluation_input_x);

    uint256_t batching_challenge_v_power{ batching_challenge_v };
    for (size_t i = 0; i < 4; i++) {
        relation_parameters.batching_challenge_v[i] = compute_five_limbs(batching_challenge_v_power);
        batching_challenge_v_power = BF(batching_challenge_v_power) * batching_challenge_v;
    }

    relation_parameters.accumulated_result = compute_four_limbs(accumulated_result);
};

/**
 * @brief This function verifies an GoblinTranslator Honk proof for given program settings.
 */
bool GoblinTranslatorVerifier::verify_proof(const plonk::proof& proof)
{
    batching_challenge_v = transcript->get_challenge("Translation:batching_challenge");
    transcript->load_proof(proof.proof_data);

    Flavor::VerifierCommitments commitments{ key };
    Flavor::CommitmentLabels commitment_labels;

    // TODO(Adrian): Change the initialization of the transcript to take the VK hash?
    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");
    evaluation_input_x = transcript->template receive_from_prover<BF>("evaluation_input_x");

    const BF accumulated_result = transcript->template receive_from_prover<BF>("accumulated_result");

    put_translation_data_in_relation_parameters(evaluation_input_x, batching_challenge_v, accumulated_result);

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get all the values of wires
    const auto receive_commitment = [&](const std::string& label) {
        return transcript->template receive_from_prover<Commitment>(label);
    };

    commitments.op = receive_commitment(commitment_labels.op);
    commitments.x_lo_y_hi = receive_commitment(commitment_labels.x_lo_y_hi);
    commitments.x_hi_z_1 = receive_commitment(commitment_labels.x_hi_z_1);
    commitments.y_lo_z_2 = receive_commitment(commitment_labels.y_lo_z_2);
    commitments.p_x_low_limbs = receive_commitment(commitment_labels.p_x_low_limbs);
    commitments.p_x_low_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_0);
    commitments.p_x_low_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_1);
    commitments.p_x_low_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_2);
    commitments.p_x_low_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_3);
    commitments.p_x_low_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_4);
    commitments.p_x_low_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.p_x_low_limbs_range_constraint_tail);
    commitments.p_x_high_limbs = receive_commitment(commitment_labels.p_x_high_limbs);
    commitments.p_x_high_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_0);
    commitments.p_x_high_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_1);
    commitments.p_x_high_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_2);
    commitments.p_x_high_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_3);
    commitments.p_x_high_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_4);
    commitments.p_x_high_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.p_x_high_limbs_range_constraint_tail);
    commitments.p_y_low_limbs = receive_commitment(commitment_labels.p_y_low_limbs);
    commitments.p_y_low_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_0);
    commitments.p_y_low_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_1);
    commitments.p_y_low_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_2);
    commitments.p_y_low_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_3);
    commitments.p_y_low_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_4);
    commitments.p_y_low_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.p_y_low_limbs_range_constraint_tail);
    commitments.p_y_high_limbs = receive_commitment(commitment_labels.p_y_high_limbs);
    commitments.p_y_high_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_0);
    commitments.p_y_high_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_1);
    commitments.p_y_high_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_2);
    commitments.p_y_high_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_3);
    commitments.p_y_high_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_4);
    commitments.p_y_high_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.p_y_high_limbs_range_constraint_tail);
    commitments.z_low_limbs = receive_commitment(commitment_labels.z_low_limbs);
    commitments.z_low_limbs_range_constraint_0 = receive_commitment(commitment_labels.z_low_limbs_range_constraint_0);
    commitments.z_low_limbs_range_constraint_1 = receive_commitment(commitment_labels.z_low_limbs_range_constraint_1);
    commitments.z_low_limbs_range_constraint_2 = receive_commitment(commitment_labels.z_low_limbs_range_constraint_2);
    commitments.z_low_limbs_range_constraint_3 = receive_commitment(commitment_labels.z_low_limbs_range_constraint_3);
    commitments.z_low_limbs_range_constraint_4 = receive_commitment(commitment_labels.z_low_limbs_range_constraint_4);
    commitments.z_low_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.z_low_limbs_range_constraint_tail);
    commitments.z_high_limbs = receive_commitment(commitment_labels.z_high_limbs);
    commitments.z_high_limbs_range_constraint_0 = receive_commitment(commitment_labels.z_high_limbs_range_constraint_0);
    commitments.z_high_limbs_range_constraint_1 = receive_commitment(commitment_labels.z_high_limbs_range_constraint_1);
    commitments.z_high_limbs_range_constraint_2 = receive_commitment(commitment_labels.z_high_limbs_range_constraint_2);
    commitments.z_high_limbs_range_constraint_3 = receive_commitment(commitment_labels.z_high_limbs_range_constraint_3);
    commitments.z_high_limbs_range_constraint_4 = receive_commitment(commitment_labels.z_high_limbs_range_constraint_4);
    commitments.z_high_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.z_high_limbs_range_constraint_tail);
    commitments.accumulators_binary_limbs_0 = receive_commitment(commitment_labels.accumulators_binary_limbs_0);
    commitments.accumulators_binary_limbs_1 = receive_commitment(commitment_labels.accumulators_binary_limbs_1);
    commitments.accumulators_binary_limbs_2 = receive_commitment(commitment_labels.accumulators_binary_limbs_2);
    commitments.accumulators_binary_limbs_3 = receive_commitment(commitment_labels.accumulators_binary_limbs_3);
    commitments.accumulator_low_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_0);
    commitments.accumulator_low_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_1);
    commitments.accumulator_low_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_2);
    commitments.accumulator_low_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_3);
    commitments.accumulator_low_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_4);
    commitments.accumulator_low_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.accumulator_low_limbs_range_constraint_tail);
    commitments.accumulator_high_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_0);
    commitments.accumulator_high_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_1);
    commitments.accumulator_high_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_2);
    commitments.accumulator_high_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_3);
    commitments.accumulator_high_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_4);
    commitments.accumulator_high_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.accumulator_high_limbs_range_constraint_tail);
    commitments.quotient_low_binary_limbs = receive_commitment(commitment_labels.quotient_low_binary_limbs);
    commitments.quotient_high_binary_limbs = receive_commitment(commitment_labels.quotient_high_binary_limbs);
    commitments.quotient_low_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_0);
    commitments.quotient_low_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_1);
    commitments.quotient_low_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_2);
    commitments.quotient_low_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_3);
    commitments.quotient_low_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_4);
    commitments.quotient_low_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.quotient_low_limbs_range_constraint_tail);
    commitments.quotient_high_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_0);
    commitments.quotient_high_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_1);
    commitments.quotient_high_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_2);
    commitments.quotient_high_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_3);
    commitments.quotient_high_limbs_range_constraint_4 =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_4);
    commitments.quotient_high_limbs_range_constraint_tail =
        receive_commitment(commitment_labels.quotient_high_limbs_range_constraint_tail);
    commitments.relation_wide_limbs = receive_commitment(commitment_labels.relation_wide_limbs);
    commitments.relation_wide_limbs_range_constraint_0 =
        receive_commitment(commitment_labels.relation_wide_limbs_range_constraint_0);
    commitments.relation_wide_limbs_range_constraint_1 =
        receive_commitment(commitment_labels.relation_wide_limbs_range_constraint_1);
    commitments.relation_wide_limbs_range_constraint_2 =
        receive_commitment(commitment_labels.relation_wide_limbs_range_constraint_2);
    commitments.relation_wide_limbs_range_constraint_3 =
        receive_commitment(commitment_labels.relation_wide_limbs_range_constraint_3);
    commitments.ordered_range_constraints_0 = receive_commitment(commitment_labels.ordered_range_constraints_0);
    commitments.ordered_range_constraints_1 = receive_commitment(commitment_labels.ordered_range_constraints_1);
    commitments.ordered_range_constraints_2 = receive_commitment(commitment_labels.ordered_range_constraints_2);
    commitments.ordered_range_constraints_3 = receive_commitment(commitment_labels.ordered_range_constraints_3);
    commitments.ordered_range_constraints_4 = receive_commitment(commitment_labels.ordered_range_constraints_4);

    // Get permutation challenges
    FF gamma = transcript->get_challenge("gamma");

    relation_parameters.beta = 0;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = 0;
    relation_parameters.lookup_grand_product_delta = 0;

    // Get commitment to permutation and lookup grand products
    commitments.z_perm = receive_commitment(commitment_labels.z_perm);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);
    FF alpha = transcript->get_challenge("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        info("sumcheck failed");
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description ofthe
    // unrolled protocol.
    auto pairing_points =
        pcs::zeromorph::ZeroMorphVerifier_<Flavor::Curve>::verify(commitments.get_unshifted(),
                                                                  commitments.get_to_be_shifted(),
                                                                  claimed_evaluations.get_unshifted(),
                                                                  claimed_evaluations.get_shifted(),
                                                                  multivariate_challenge,
                                                                  transcript,
                                                                  commitments.get_concatenation_groups(),
                                                                  claimed_evaluations.get_concatenated_constraints());

    auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);

    return verified;
}

bool GoblinTranslatorVerifier::verify_translation(const TranslationEvaluations& translation_evaluations)
{
    const auto reconstruct_from_array = [&](const auto& arr) {
        const BF elt_0 = (static_cast<uint256_t>(arr[0]));
        const BF elt_1 = (static_cast<uint256_t>(arr[1]) << 68);
        const BF elt_2 = (static_cast<uint256_t>(arr[2]) << 136);
        const BF elt_3 = (static_cast<uint256_t>(arr[3]) << 204);
        const BF reconstructed = elt_0 + elt_1 + elt_2 + elt_3;
        return reconstructed;
    };

    const auto& reconstruct_value_from_eccvm_evaluations = [&](const TranslationEvaluations& translation_evaluations,
                                                               auto& relation_parameters) {
        const BF accumulated_result = reconstruct_from_array(relation_parameters.accumulated_result);
        const BF x = reconstruct_from_array(relation_parameters.evaluation_input_x);
        const BF v1 = reconstruct_from_array(relation_parameters.batching_challenge_v[0]);
        const BF v2 = reconstruct_from_array(relation_parameters.batching_challenge_v[1]);
        const BF v3 = reconstruct_from_array(relation_parameters.batching_challenge_v[2]);
        const BF v4 = reconstruct_from_array(relation_parameters.batching_challenge_v[3]);
        const BF& op = translation_evaluations.op;
        const BF& Px = translation_evaluations.Px;
        const BF& Py = translation_evaluations.Py;
        const BF& z1 = translation_evaluations.z1;
        const BF& z2 = translation_evaluations.z2;

        const BF eccvm_opening = (op + (v1 * Px) + (v2 * Py) + (v3 * z1) + (v4 * z2));
        // multiply by x here to deal with shift
        return x * accumulated_result == eccvm_opening;
    };

    bool is_value_reconstructed =
        reconstruct_value_from_eccvm_evaluations(translation_evaluations, relation_parameters);
    return is_value_reconstructed;
}

} // namespace bb::honk
