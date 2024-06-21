#include "./translator_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

TranslatorVerifier::TranslatorVerifier(const std::shared_ptr<TranslatorVerifier::VerificationKey>& verifier_key,
                                       const std::shared_ptr<Transcript>& transcript)
    : key(verifier_key)
    , transcript(transcript)
{}

TranslatorVerifier::TranslatorVerifier(const std::shared_ptr<TranslatorVerifier::ProvingKey>& proving_key,
                                       const std::shared_ptr<Transcript>& transcript)
    : TranslatorVerifier(std::make_shared<TranslatorFlavor::VerificationKey>(proving_key), transcript){};

void TranslatorVerifier::put_translation_data_in_relation_parameters(const uint256_t& evaluation_input_x,
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
 * @brief This function verifies an TranslatorFlavor Honk proof for given program settings.
 */
bool TranslatorVerifier::verify_proof(const HonkProof& proof)
{
    using Curve = typename Flavor::Curve;
    using PCS = typename Flavor::PCS;
    using ZeroMorph = ::bb::ZeroMorphVerifier_<Curve>;

    batching_challenge_v = transcript->template get_challenge<BF>("Translation:batching_challenge");

    // Load the proof produced by the translator prover
    transcript->load_proof(proof);

    Flavor::VerifierCommitments commitments{ key };
    Flavor::CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");
    evaluation_input_x = transcript->template receive_from_prover<BF>("evaluation_input_x");

    const BF accumulated_result = transcript->template receive_from_prover<BF>("accumulated_result");

    put_translation_data_in_relation_parameters(evaluation_input_x, batching_challenge_v, accumulated_result);

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to wires and the ordered range constraints that do not require additional challenges
    for (auto [comm, label] : zip_view(commitments.get_wires_and_ordered_range_constraints(),
                                       commitment_labels.get_wires_and_ordered_range_constraints())) {
        comm = transcript->template receive_from_prover<Commitment>(label);
    }

    // Get permutation challenges
    FF gamma = transcript->template get_challenge<FF>("gamma");

    relation_parameters.beta = 0;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = 0;
    relation_parameters.lookup_grand_product_delta = 0;

    // Get commitment to permutation and lookup grand products
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

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description ofthe
    // unrolled protocol.

    auto opening_claim = ZeroMorph::verify(commitments.get_unshifted_without_concatenated(),
                                           commitments.get_to_be_shifted(),
                                           claimed_evaluations.get_unshifted_without_concatenated(),
                                           claimed_evaluations.get_shifted(),
                                           multivariate_challenge,
                                           Commitment::one(),
                                           transcript,
                                           commitments.get_concatenation_groups(),
                                           claimed_evaluations.get_concatenated_constraints());
    auto pairing_points = PCS::reduce_verify(opening_claim, transcript);

    auto verified = key->pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);

    return verified;
}

bool TranslatorVerifier::verify_translation(const TranslationEvaluations& translation_evaluations)
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

} // namespace bb
