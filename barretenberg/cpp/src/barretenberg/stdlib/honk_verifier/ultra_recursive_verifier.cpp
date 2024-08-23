#include "barretenberg/stdlib/honk_verifier/ultra_recursive_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Flavor>
UltraRecursiveVerifier_<Flavor>::UltraRecursiveVerifier_(
    Builder* builder, const std::shared_ptr<NativeVerificationKey>& native_verifier_key)
    : key(std::make_shared<VerificationKey>(builder, native_verifier_key))
    , builder(builder)
{}

template <typename Flavor>
UltraRecursiveVerifier_<Flavor>::UltraRecursiveVerifier_(Builder* builder, const std::shared_ptr<VerificationKey>& vkey)
    : key(vkey)
    , builder(builder)
{}

/**
 * @brief This function constructs a recursive verifier circuit for a native Ultra Honk proof of a given flavor.
 * @return Output aggregation object
 */
template <typename Flavor>
UltraRecursiveVerifier_<Flavor>::AggregationObject UltraRecursiveVerifier_<Flavor>::verify_proof(
    const HonkProof& proof, aggregation_state<typename Flavor::Curve> agg_obj)
{
    StdlibProof<Builder> stdlib_proof = bb::convert_proof_to_witness(builder, proof);
    return verify_proof(stdlib_proof, agg_obj);
}

/**
 * @brief This function constructs a recursive verifier circuit for a native Ultra Honk proof of a given flavor.
 * @return Output aggregation object
 */
template <typename Flavor>
UltraRecursiveVerifier_<Flavor>::AggregationObject UltraRecursiveVerifier_<Flavor>::verify_proof(
    const StdlibProof<Builder>& proof, aggregation_state<typename Flavor::Curve> agg_obj)
{
    using Sumcheck = ::bb::SumcheckVerifier<Flavor>;
    using PCS = typename Flavor::PCS;
    using Curve = typename Flavor::Curve;
    using ZeroMorph = ::bb::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using Transcript = typename Flavor::Transcript;

    transcript = std::make_shared<Transcript>(proof);
    auto instance = std::make_shared<Instance>(builder, key);
    OinkVerifier oink_verifier{ builder, instance, transcript };
    oink_verifier.verify();

    VerifierCommitments commitments{ key, instance->witness_commitments };

    auto gate_challenges = std::vector<FF>(CONST_PROOF_SIZE_LOG_N);
    for (size_t idx = 0; idx < CONST_PROOF_SIZE_LOG_N; idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    // Parse out the aggregation object using the key->recursive_proof_public_inputs_indices
    aggregation_state<typename Flavor::Curve> nested_agg_obj;
    size_t idx = 0;
    std::array<typename Curve::Group, 2> nested_pairing_points;
    for (size_t i = 0; i < 2; i++) {
        std::array<typename Curve::BaseField, 2> base_field_vals;
        for (size_t j = 0; j < 2; j++) {
            std::array<FF, 4> bigfield_limbs;
            for (size_t k = 0; k < 4; k++) {
                bigfield_limbs[k] = instance->public_inputs[key->recursive_proof_public_input_indices[idx]];
                idx++;
            }
            base_field_vals[j] =
                typename Curve::BaseField(bigfield_limbs[0], bigfield_limbs[1], bigfield_limbs[2], bigfield_limbs[3]);
        }
        nested_pairing_points[i] = typename Curve::Group(base_field_vals[0], base_field_vals[1]);
    }

    nested_agg_obj.P0 = nested_pairing_points[0];
    nested_agg_obj.P1 = nested_pairing_points[1];
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/995): generate this challenge properly.
    typename Curve::ScalarField recursion_separator =
        Curve::ScalarField::from_witness_index(builder, builder->add_variable(42));
    agg_obj.aggregate(nested_agg_obj, recursion_separator);

    // Execute Sumcheck Verifier and extract multivariate opening point u = (u_0, ..., u_{d-1}) and purported
    // multivariate evaluations at u
    const size_t log_circuit_size = numeric::get_msb(static_cast<uint32_t>(key->circuit_size));
    auto sumcheck = Sumcheck(log_circuit_size, transcript);

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(instance->relation_parameters, instance->alphas, gate_challenges);

    // Execute ZeroMorph to produce an opening claim subsequently verified by a univariate PCS
    auto opening_claim = ZeroMorph::verify(key->circuit_size,
                                           commitments.get_unshifted(),
                                           commitments.get_to_be_shifted(),
                                           claimed_evaluations.get_unshifted(),
                                           claimed_evaluations.get_shifted(),
                                           multivariate_challenge,
                                           Commitment::one(builder),
                                           transcript);
    auto pairing_points = PCS::reduce_verify(opening_claim, transcript);

    pairing_points[0] = pairing_points[0].normalize();
    pairing_points[1] = pairing_points[1].normalize();
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/995): generate recursion separator challenge properly.
    agg_obj.aggregate(pairing_points, recursion_separator);
    return agg_obj;
}

template class UltraRecursiveVerifier_<bb::UltraRecursiveFlavor_<UltraCircuitBuilder>>;
template class UltraRecursiveVerifier_<bb::UltraRecursiveFlavor_<MegaCircuitBuilder>>;
template class UltraRecursiveVerifier_<bb::MegaRecursiveFlavor_<UltraCircuitBuilder>>;
template class UltraRecursiveVerifier_<bb::MegaRecursiveFlavor_<MegaCircuitBuilder>>;
template class UltraRecursiveVerifier_<bb::UltraRecursiveFlavor_<CircuitSimulatorBN254>>;
template class UltraRecursiveVerifier_<bb::MegaRecursiveFlavor_<CircuitSimulatorBN254>>;
} // namespace bb::stdlib::recursion::honk
