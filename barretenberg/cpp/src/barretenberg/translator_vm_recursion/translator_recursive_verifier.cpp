#include "./translator_recursive_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/relations/translator_vm/translator_decomposition_relation_impl.hpp"
#include "barretenberg/relations/translator_vm/translator_delta_range_constraint_relation_impl.hpp"
#include "barretenberg/relations/translator_vm/translator_extra_relations_impl.hpp"
#include "barretenberg/relations/translator_vm/translator_non_native_field_relation_impl.hpp"
#include "barretenberg/relations/translator_vm/translator_permutation_relation_impl.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

template <typename Flavor>
TranslatorRecursiveVerifier_<Flavor>::TranslatorRecursiveVerifier_(
    Builder* builder,
    const std::shared_ptr<NativeVerificationKey>& native_verifier_key,
    const std::shared_ptr<Transcript>& transcript)
    : key(std::make_shared<VerificationKey>(builder, native_verifier_key))
    , transcript(transcript)
    , builder(builder)
{}

// Relation params used in sumcheck which is done over FF but the received data is from BF
template <typename Flavor>
void TranslatorRecursiveVerifier_<Flavor>::put_translation_data_in_relation_parameters(const BF& evaluation_input_x,
                                                                                       const BF& batching_challenge_v,
                                                                                       const BF& accumulated_result)
{

    const auto compute_four_limbs = [](const BF& in) {
        return std::array<FF, 4>{ FF(in.binary_basis_limbs[0].element),
                                  FF(in.binary_basis_limbs[1].element),
                                  FF(in.binary_basis_limbs[2].element),
                                  FF(in.binary_basis_limbs[3].element) };
    };

    const auto compute_five_limbs = [](const BF& in) {
        return std::array<FF, 5>{ FF(in.binary_basis_limbs[0].element),
                                  FF(in.binary_basis_limbs[1].element),
                                  FF(in.binary_basis_limbs[2].element),
                                  FF(in.binary_basis_limbs[3].element),
                                  FF(in.prime_basis_limb) };
    };

    relation_parameters.evaluation_input_x = compute_five_limbs(evaluation_input_x);

    BF batching_challenge_v_power = batching_challenge_v;
    for (size_t i = 0; i < 4; i++) {
        relation_parameters.batching_challenge_v[i] = compute_five_limbs(batching_challenge_v_power);
        batching_challenge_v_power = batching_challenge_v_power * batching_challenge_v;
    }

    relation_parameters.accumulated_result = compute_four_limbs(accumulated_result);
};

/**
 * @brief This function verifies an TranslatorFlavor Honk proof for given program settings.
 */
template <typename Flavor>
std::array<typename Flavor::GroupElement, 2> TranslatorRecursiveVerifier_<Flavor>::verify_proof(const HonkProof& proof)
{
    using Sumcheck = ::bb::SumcheckVerifier<Flavor>;
    using PCS = typename Flavor::PCS;
    using ZeroMorph = ::bb::ZeroMorphVerifier_<PCS>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;

    StdlibProof<Builder> stdlib_proof = bb::convert_proof_to_witness(builder, proof);
    transcript->load_proof(stdlib_proof);

    batching_challenge_v = transcript->template get_challenge<BF>("Translation:batching_challenge");

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<FF>("circuit_size");
    ASSERT(static_cast<uint32_t>(circuit_size.get_value()) == key->circuit_size);
    evaluation_input_x = transcript->template receive_from_prover<BF>("evaluation_input_x");

    const BF accumulated_result = transcript->template receive_from_prover<BF>("accumulated_result");

    put_translation_data_in_relation_parameters(evaluation_input_x, batching_challenge_v, accumulated_result);

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
    const size_t log_circuit_size = numeric::get_msb(static_cast<uint32_t>(circuit_size.get_value()));
    auto sumcheck = Sumcheck(log_circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(log_circuit_size);
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description ofthe
    // unrolled protocol.
    auto pairing_points = ZeroMorph::verify(commitments.get_unshifted_without_concatenated(),
                                            commitments.get_to_be_shifted(),
                                            claimed_evaluations.get_unshifted_without_concatenated(),
                                            claimed_evaluations.get_shifted(),
                                            multivariate_challenge,
                                            transcript,
                                            commitments.get_concatenation_groups(),
                                            claimed_evaluations.get_concatenated_constraints());

    return pairing_points;
}

template <typename Flavor>
bool TranslatorRecursiveVerifier_<Flavor>::verify_translation(
    const TranslationEvaluations_<typename Flavor::BF, typename Flavor::FF>& translation_evaluations)
{
    const auto reconstruct_from_array = [&](const auto& arr) {
        const BF reconstructed = BF(arr[0], arr[1], arr[2], arr[3]);
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
        eccvm_opening.assert_equal(x * accumulated_result);
        return (eccvm_opening.get_value() == (x * accumulated_result).get_value());
    };

    bool is_value_reconstructed =
        reconstruct_value_from_eccvm_evaluations(translation_evaluations, relation_parameters);
    return is_value_reconstructed;
}
template class TranslatorRecursiveVerifier_<bb::TranslatorRecursiveFlavor_<UltraCircuitBuilder>>;
template class TranslatorRecursiveVerifier_<bb::TranslatorRecursiveFlavor_<MegaCircuitBuilder>>;
template class TranslatorRecursiveVerifier_<bb::TranslatorRecursiveFlavor_<CircuitSimulatorBN254>>;

} // namespace bb
