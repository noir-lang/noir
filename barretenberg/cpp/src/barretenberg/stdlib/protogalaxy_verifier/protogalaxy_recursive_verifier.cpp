#include "protogalaxy_recursive_verifier.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/stdlib/protogalaxy_verifier/recursive_instances.hpp"

namespace bb::stdlib::recursion::honk {

template <class VerifierInstances>
void ProtogalaxyRecursiveVerifier_<VerifierInstances>::receive_and_finalise_instance(
    const std::shared_ptr<Instance>& inst, std::string& domain_separator)
{
    domain_separator = domain_separator + "_";
    OinkVerifier oink_verifier{ builder, inst, transcript, domain_separator };
    oink_verifier.verify();
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/795): The rounds prior to actual verifying are common
// between decider and folding verifier and could be somehow shared so we do not duplicate code so much.
template <class VerifierInstances> void ProtogalaxyRecursiveVerifier_<VerifierInstances>::prepare_for_folding()
{
    auto index = 0;
    auto inst = instances[0];
    auto domain_separator = std::to_string(index);

    if (!inst->is_accumulator) {
        receive_and_finalise_instance(inst, domain_separator);
        inst->target_sum = 0;
        inst->gate_challenges = std::vector<FF>(static_cast<size_t>(inst->verification_key->log_circuit_size), 0);
    }
    index++;

    for (auto it = instances.begin() + 1; it != instances.end(); it++, index++) {
        auto inst = *it;
        auto domain_separator = std::to_string(index);
        receive_and_finalise_instance(inst, domain_separator);
    }
}

template <class VerifierInstances>
std::shared_ptr<typename VerifierInstances::Instance> ProtogalaxyRecursiveVerifier_<
    VerifierInstances>::verify_folding_proof(const StdlibProof<Builder>& proof)
{
    using Transcript = typename Flavor::Transcript;

    transcript = std::make_shared<Transcript>(proof);
    prepare_for_folding();

    auto delta = transcript->template get_challenge<FF>("delta");
    auto accumulator = get_accumulator();
    auto deltas =
        compute_round_challenge_pows(static_cast<size_t>(accumulator->verification_key->log_circuit_size), delta);

    std::vector<FF> perturbator_coeffs(static_cast<size_t>(accumulator->verification_key->log_circuit_size) + 1, 0);
    if (accumulator->is_accumulator) {
        for (size_t idx = 1; idx <= static_cast<size_t>(accumulator->verification_key->log_circuit_size); idx++) {
            perturbator_coeffs[idx] =
                transcript->template receive_from_prover<FF>("perturbator_" + std::to_string(idx));
        }
    }

    perturbator_coeffs[0] = accumulator->target_sum;

    FF perturbator_challenge = transcript->template get_challenge<FF>("perturbator_challenge");

    auto perturbator_at_challenge = evaluate_perturbator(perturbator_coeffs, perturbator_challenge);
    // The degree of K(X) is dk - k - 1 = k(d - 1) - 1. Hence we need  k(d - 1) evaluations to represent it.
    std::array<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM> combiner_quotient_evals;
    for (size_t idx = 0; idx < VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM; idx++) {
        combiner_quotient_evals[idx] = transcript->template receive_from_prover<FF>(
            "combiner_quotient_" + std::to_string(idx + VerifierInstances::NUM));
    }
    Univariate<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH, VerifierInstances::NUM> combiner_quotient(
        combiner_quotient_evals);
    FF combiner_challenge = transcript->template get_challenge<FF>("combiner_quotient_challenge");
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(combiner_challenge); // fine recursive i think

    auto vanishing_polynomial_at_challenge = combiner_challenge * (combiner_challenge - FF(1));
    auto lagranges = std::vector<FF>{ FF(1) - combiner_challenge, combiner_challenge };

    auto next_accumulator = std::make_shared<Instance>(builder);

    next_accumulator->verification_key = std::make_shared<VerificationKey>(
        accumulator->verification_key->circuit_size, accumulator->verification_key->num_public_inputs);
    next_accumulator->verification_key->pcs_verification_key = accumulator->verification_key->pcs_verification_key;
    next_accumulator->verification_key->pub_inputs_offset = accumulator->verification_key->pub_inputs_offset;
    next_accumulator->verification_key->contains_recursive_proof =
        accumulator->verification_key->contains_recursive_proof;
    next_accumulator->verification_key->recursive_proof_public_input_indices =
        accumulator->verification_key->recursive_proof_public_input_indices;
    if constexpr (IsGoblinFlavor<Flavor>) { // Databus commitment propagation data
        next_accumulator->verification_key->databus_propagation_data =
            accumulator->verification_key->databus_propagation_data;
    }

    next_accumulator->public_inputs = accumulator->public_inputs;

    next_accumulator->is_accumulator = true;

    // Compute next folding parameters
    next_accumulator->target_sum =
        perturbator_at_challenge * lagranges[0] + vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;
    next_accumulator->gate_challenges =
        update_gate_challenges(perturbator_challenge, accumulator->gate_challenges, deltas);

    // Compute ϕ
    fold_commitments(lagranges, instances, next_accumulator);

    size_t alpha_idx = 0;
    for (auto& alpha : next_accumulator->alphas) {
        alpha = FF(0);
        size_t instance_idx = 0;
        for (auto& instance : instances) {
            alpha += instance->alphas[alpha_idx] * lagranges[instance_idx];
            instance_idx++;
        }
        alpha_idx++;
    }

    auto& expected_parameters = next_accumulator->relation_parameters;
    for (size_t inst_idx = 0; inst_idx < VerifierInstances::NUM; inst_idx++) {
        auto instance = instances[inst_idx];
        expected_parameters.eta += instance->relation_parameters.eta * lagranges[inst_idx];
        expected_parameters.eta_two += instance->relation_parameters.eta_two * lagranges[inst_idx];
        expected_parameters.eta_three += instance->relation_parameters.eta_three * lagranges[inst_idx];
        expected_parameters.beta += instance->relation_parameters.beta * lagranges[inst_idx];
        expected_parameters.gamma += instance->relation_parameters.gamma * lagranges[inst_idx];
        expected_parameters.public_input_delta +=
            instance->relation_parameters.public_input_delta * lagranges[inst_idx];
        expected_parameters.lookup_grand_product_delta +=
            instance->relation_parameters.lookup_grand_product_delta * lagranges[inst_idx];
    }
    return next_accumulator;
}

template class ProtogalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<UltraRecursiveFlavor_<UltraCircuitBuilder>, 2>>;
template class ProtogalaxyRecursiveVerifier_<RecursiveVerifierInstances_<MegaRecursiveFlavor_<MegaCircuitBuilder>, 2>>;
template class ProtogalaxyRecursiveVerifier_<RecursiveVerifierInstances_<UltraRecursiveFlavor_<MegaCircuitBuilder>, 2>>;
template class ProtogalaxyRecursiveVerifier_<RecursiveVerifierInstances_<MegaRecursiveFlavor_<UltraCircuitBuilder>, 2>>;
template class ProtogalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<UltraRecursiveFlavor_<CircuitSimulatorBN254>, 2>>;
template class ProtogalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<MegaRecursiveFlavor_<CircuitSimulatorBN254>, 2>>;
} // namespace bb::stdlib::recursion::honk