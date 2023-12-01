#include "protogalaxy_verifier.hpp"
#include "barretenberg/proof_system/library/grand_product_delta.hpp"
namespace proof_system::honk {

template <class VerifierInstances>
void ProtoGalaxyVerifier_<VerifierInstances>::prepare_for_folding(std::vector<uint8_t> fold_data)
{
    transcript = std::make_shared<Transcript>(fold_data);
    auto index = 0;
    for (auto it = verifier_instances.begin(); it != verifier_instances.end(); it++, index++) {
        auto inst = *it;
        auto domain_separator = std::to_string(index);
        inst->instance_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "_circuit_size");
        inst->public_input_size =
            transcript->template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");
        inst->pub_inputs_offset =
            transcript->template receive_from_prover<uint32_t>(domain_separator + "_pub_inputs_offset");

        for (size_t i = 0; i < inst->public_input_size; ++i) {
            auto public_input_i =
                transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
            inst->public_inputs.emplace_back(public_input_i);
        }
        auto [eta, beta, gamma] = challenges_to_field_elements<FF>(transcript->get_challenges(
            domain_separator + "_eta", domain_separator + "_beta", domain_separator + "_gamma"));

        const FF public_input_delta = compute_public_input_delta<Flavor>(
            inst->public_inputs, beta, gamma, inst->instance_size, inst->pub_inputs_offset);
        const FF lookup_grand_product_delta = compute_lookup_grand_product_delta<FF>(beta, gamma, inst->instance_size);
        inst->relation_parameters =
            RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };
        inst->alpha = transcript->get_challenge(domain_separator + "_alpha");
    }
}

template <class VerifierInstances>
VerifierFoldingResult<typename VerifierInstances::Flavor> ProtoGalaxyVerifier_<
    VerifierInstances>::fold_public_parameters(std::vector<uint8_t> fold_data)
{
    using Flavor = typename VerifierInstances::Flavor;

    prepare_for_folding(fold_data);
    FF delta = transcript->get_challenge("delta");
    auto accumulator = get_accumulator();
    auto log_instance_size = static_cast<size_t>(numeric::get_msb(accumulator->instance_size));
    auto deltas = compute_round_challenge_pows(log_instance_size, delta);
    std::vector<FF> perturbator_coeffs(log_instance_size + 1);
    for (size_t idx = 0; idx <= log_instance_size; idx++) {
        perturbator_coeffs[idx] = transcript->template receive_from_prover<FF>("perturbator_" + std::to_string(idx));
    }
    auto perturbator = Polynomial<FF>(perturbator_coeffs);
    FF perturbator_challenge = transcript->get_challenge("perturbator_challenge");
    auto perturbator_at_challenge = perturbator.evaluate(perturbator_challenge);

    // Thed degree of K(X) is dk - k - 1 = k(d - 1) - 1. Hence we need  k(d - 1) evaluations to represent it.
    std::array<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM> combiner_quotient_evals = {};
    for (size_t idx = 0; idx < VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM; idx++) {
        combiner_quotient_evals[idx] = transcript->template receive_from_prover<FF>(
            "combiner_quotient_" + std::to_string(idx + VerifierInstances::NUM));
    }
    Univariate<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH, VerifierInstances::NUM> combiner_quotient(
        combiner_quotient_evals);
    FF combiner_challenge = transcript->get_challenge("combiner_quotient_challenge");
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(combiner_challenge);

    auto vanishing_polynomial_at_challenge = combiner_challenge * (combiner_challenge - FF(1));
    auto lagrange_0_at_challenge = FF(1) - combiner_challenge;

    auto new_target_sum = perturbator_at_challenge * lagrange_0_at_challenge +
                          vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;

    VerifierFoldingResult<Flavor> res;
    res.parameters.target_sum = new_target_sum;
    return res;
}

template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk