#include "protogalaxy_verifier.hpp"
#include "barretenberg/proof_system/library/grand_product_delta.hpp"
namespace bb::honk {

template <class VerifierInstances>
void ProtoGalaxyVerifier_<VerifierInstances>::receive_accumulator(const std::shared_ptr<Instance>& inst,
                                                                  const std::string& domain_separator)
{
    inst->instance_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "_instance_size");
    inst->log_instance_size = static_cast<size_t>(numeric::get_msb(inst->instance_size));
    inst->public_input_size =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");

    for (size_t i = 0; i < inst->public_input_size; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    auto eta = transcript->template receive_from_prover<FF>(domain_separator + "_eta");
    auto beta = transcript->template receive_from_prover<FF>(domain_separator + "_beta");
    auto gamma = transcript->template receive_from_prover<FF>(domain_separator + "_gamma");
    auto public_input_delta = transcript->template receive_from_prover<FF>(domain_separator + "_public_input_delta");
    auto lookup_grand_product_delta =
        transcript->template receive_from_prover<FF>(domain_separator + "_lookup_grand_product_delta");
    inst->relation_parameters =
        RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };

    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] =
            transcript->template receive_from_prover<FF>(domain_separator + "_alpha_" + std::to_string(idx));
    }

    inst->target_sum = transcript->template receive_from_prover<FF>(domain_separator + "_target_sum");

    inst->gate_challenges = std::vector<FF>(inst->log_instance_size);
    for (size_t idx = 0; idx < inst->log_instance_size; idx++) {
        inst->gate_challenges[idx] =
            transcript->template receive_from_prover<FF>(domain_separator + "_gate_challenge_" + std::to_string(idx));
    }
    auto comm_view = inst->witness_commitments.get_all();
    auto witness_labels = inst->commitment_labels.get_witness();
    for (size_t idx = 0; idx < witness_labels.size(); idx++) {
        comm_view[idx] =
            transcript->template receive_from_prover<Commitment>(domain_separator + "_" + witness_labels[idx]);
    }

    inst->verification_key = std::make_shared<VerificationKey>(inst->instance_size, inst->public_input_size);
    auto vk_view = inst->verification_key->get_all();
    auto vk_labels = inst->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        vk_view[idx] = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + vk_labels[idx]);
    }
}

template <class VerifierInstances>
void ProtoGalaxyVerifier_<VerifierInstances>::receive_and_finalise_instance(const std::shared_ptr<Instance>& inst,
                                                                            const std::string& domain_separator)
{
    inst->instance_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "_instance_size");
    inst->log_instance_size = static_cast<size_t>(numeric::get_msb(inst->instance_size));
    inst->public_input_size =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");

    for (size_t i = 0; i < inst->public_input_size; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    inst->pub_inputs_offset =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_pub_inputs_offset");

    auto labels = inst->commitment_labels;
    auto& witness_commitments = inst->witness_commitments;
    witness_commitments.w_l = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_l);
    witness_commitments.w_r = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_r);
    witness_commitments.w_o = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_o);

    auto eta = transcript->get_challenge(domain_separator + "_eta");
    witness_commitments.sorted_accum =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.sorted_accum);
    witness_commitments.w_4 = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_4);

    auto [beta, gamma] = transcript->get_challenges(domain_separator + "_beta", domain_separator + "_gamma");
    witness_commitments.z_perm =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.z_perm);
    witness_commitments.z_lookup =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.z_lookup);

    const FF public_input_delta = compute_public_input_delta<Flavor>(
        inst->public_inputs, beta, gamma, inst->instance_size, inst->pub_inputs_offset);
    const FF lookup_grand_product_delta = compute_lookup_grand_product_delta<FF>(beta, gamma, inst->instance_size);
    inst->relation_parameters =
        RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };

    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] = transcript->get_challenge(domain_separator + "_alpha_" + std::to_string(idx));
    }

    inst->verification_key = std::make_shared<VerificationKey>(inst->instance_size, inst->public_input_size);
    auto vk_view = inst->verification_key->get_all();
    auto vk_labels = labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        vk_view[idx] = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + vk_labels[idx]);
    }
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/795): The rounds prior to actual verifying are common
// between decider and folding verifier and could be somehow shared so we do not duplicate code so much.
template <class VerifierInstances>
void ProtoGalaxyVerifier_<VerifierInstances>::prepare_for_folding(const std::vector<uint8_t>& fold_data)
{
    transcript = std::make_shared<Transcript>(fold_data);
    auto index = 0;
    auto inst = instances[0];
    auto domain_separator = std::to_string(index);
    inst->is_accumulator = transcript->template receive_from_prover<bool>(domain_separator + "is_accumulator");
    if (inst->is_accumulator) {
        receive_accumulator(inst, domain_separator);
    } else {
        // This is the first round of folding and we need to generate some gate challenges.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/740): implement option 2 to make this more
        // efficient by avoiding the computation of the perturbator
        receive_and_finalise_instance(inst, domain_separator);
        inst->target_sum = 0;
        auto beta = transcript->get_challenge(domain_separator + "_initial_gate_challenge");
        std::vector<FF> gate_challenges(inst->log_instance_size);
        gate_challenges[0] = beta;
        for (size_t i = 1; i < inst->log_instance_size; i++) {
            gate_challenges[i] = gate_challenges[i - 1].sqr();
        }
        inst->gate_challenges = gate_challenges;
    }
    index++;

    for (auto it = instances.begin() + 1; it != instances.end(); it++, index++) {
        auto inst = *it;
        auto domain_separator = std::to_string(index);
        receive_and_finalise_instance(inst, domain_separator);
    }
}

template <class VerifierInstances>
bool ProtoGalaxyVerifier_<VerifierInstances>::verify_folding_proof(std::vector<uint8_t> fold_data)
{
    prepare_for_folding(fold_data);

    auto delta = transcript->get_challenge("delta");
    auto accumulator = get_accumulator();
    auto deltas = compute_round_challenge_pows(accumulator->log_instance_size, delta);

    std::vector<FF> perturbator_coeffs(accumulator->log_instance_size + 1);
    for (size_t idx = 0; idx <= accumulator->log_instance_size; idx++) {
        perturbator_coeffs[idx] = transcript->template receive_from_prover<FF>("perturbator_" + std::to_string(idx));
    }

    if (perturbator_coeffs[0] != accumulator->target_sum) {
        return false;
    }

    auto perturbator = Polynomial<FF>(perturbator_coeffs);
    FF perturbator_challenge = transcript->get_challenge("perturbator_challenge");
    auto perturbator_at_challenge = perturbator.evaluate(perturbator_challenge);

    // The degree of K(X) is dk - k - 1 = k(d - 1) - 1. Hence we need  k(d - 1) evaluations to represent it.
    std::array<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM> combiner_quotient_evals;
    for (size_t idx = 0; idx < VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM; idx++) {
        combiner_quotient_evals[idx] = transcript->template receive_from_prover<FF>(
            "combiner_quotient_" + std::to_string(idx + VerifierInstances::NUM));
    }
    Univariate<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH, VerifierInstances::NUM> combiner_quotient(
        combiner_quotient_evals);
    FF combiner_challenge = transcript->get_challenge("combiner_quotient_challenge");
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(combiner_challenge);

    auto vanishing_polynomial_at_challenge = combiner_challenge * (combiner_challenge - FF(1));
    auto lagranges = std::vector<FF>{ FF(1) - combiner_challenge, combiner_challenge };

    // Compute next folding parameters and verify against the ones received from the prover
    auto expected_next_target_sum =
        perturbator_at_challenge * lagranges[0] + vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;
    auto next_target_sum = transcript->template receive_from_prover<FF>("next_target_sum");
    bool verified = (expected_next_target_sum == next_target_sum);
    auto expected_betas_star = update_gate_challenges(perturbator_challenge, accumulator->gate_challenges, deltas);
    for (size_t idx = 0; idx < accumulator->log_instance_size; idx++) {
        auto beta_star = transcript->template receive_from_prover<FF>("next_gate_challenge_" + std::to_string(idx));
        verified = verified & (expected_betas_star[idx] == beta_star);
    }

    // Compute ϕ and verify against the data received from the prover
    WitnessCommitments acc_witness_commitments;
    auto witness_labels = commitment_labels.get_witness();
    size_t comm_idx = 0;
    for (auto& expected_comm : acc_witness_commitments.get_all()) {
        expected_comm = Commitment::infinity();
        size_t inst = 0;
        for (auto& instance : instances) {
            expected_comm = expected_comm + instance->witness_commitments.get_all()[comm_idx] * lagranges[inst];
            inst++;
        }
        auto comm = transcript->template receive_from_prover<Commitment>("next_" + witness_labels[comm_idx]);
        verified = verified & (comm == expected_comm);
        comm_idx++;
    }

    std::vector<FF> folded_public_inputs(instances[0]->public_inputs.size(), 0);
    size_t el_idx = 0;
    for (auto& expected_el : folded_public_inputs) {
        size_t inst = 0;
        for (auto& instance : instances) {
            expected_el += instance->public_inputs[el_idx] * lagranges[inst];
            inst++;
        }
        auto el = transcript->template receive_from_prover<FF>("next_public_input" + std::to_string(el_idx));
        verified = verified & (el == expected_el);
        el_idx++;
    }

    for (size_t alpha_idx = 0; alpha_idx < NUM_SUBRELATIONS - 1; alpha_idx++) {
        FF alpha(0);
        size_t instance_idx = 0;
        for (auto& instance : instances) {
            alpha += instance->alphas[alpha_idx] * lagranges[instance_idx];
            instance_idx++;
        }
        auto next_alpha = transcript->template receive_from_prover<FF>("next_alpha_" + std::to_string(alpha_idx));
        verified = verified & (alpha == next_alpha);
    }
    auto expected_parameters = bb::RelationParameters<FF>{};
    for (size_t inst_idx = 0; inst_idx < VerifierInstances::NUM; inst_idx++) {
        auto instance = instances[inst_idx];
        expected_parameters.eta += instance->relation_parameters.eta * lagranges[inst_idx];
        expected_parameters.beta += instance->relation_parameters.beta * lagranges[inst_idx];
        expected_parameters.gamma += instance->relation_parameters.gamma * lagranges[inst_idx];
        expected_parameters.public_input_delta +=
            instance->relation_parameters.public_input_delta * lagranges[inst_idx];
        expected_parameters.lookup_grand_product_delta +=
            instance->relation_parameters.lookup_grand_product_delta * lagranges[inst_idx];
    }

    auto next_eta = transcript->template receive_from_prover<FF>("next_eta");
    verified = verified & (next_eta == expected_parameters.eta);

    auto next_beta = transcript->template receive_from_prover<FF>("next_beta");
    verified = verified & (next_beta == expected_parameters.beta);

    auto next_gamma = transcript->template receive_from_prover<FF>("next_gamma");
    verified = verified & (next_gamma == expected_parameters.gamma);

    auto next_public_input_delta = transcript->template receive_from_prover<FF>("next_public_input_delta");
    verified = verified & (next_public_input_delta == expected_parameters.public_input_delta);

    auto next_lookup_grand_product_delta =
        transcript->template receive_from_prover<FF>("next_lookup_grand_product_delta");
    verified = verified & (next_lookup_grand_product_delta == expected_parameters.lookup_grand_product_delta);

    auto acc_vk = std::make_shared<VerificationKey>(instances[0]->instance_size, instances[0]->public_input_size);
    auto vk_labels = commitment_labels.get_precomputed();
    size_t vk_idx = 0;
    for (auto& expected_vk : acc_vk->get_all()) {
        size_t inst = 0;
        expected_vk = Commitment::infinity();
        for (auto& instance : instances) {
            expected_vk = expected_vk + instance->verification_key->get_all()[vk_idx] * lagranges[inst];
            inst++;
        }
        auto vk = transcript->template receive_from_prover<Commitment>("next_" + vk_labels[vk_idx]);
        verified = verified & (vk == expected_vk);
        vk_idx++;
    }

    return verified;
}

template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace bb::honk