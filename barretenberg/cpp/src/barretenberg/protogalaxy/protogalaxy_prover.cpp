#include "protogalaxy_prover.hpp"
#include "barretenberg/flavor/flavor.hpp"
namespace bb::honk {
template <class ProverInstances>
void ProtoGalaxyProver_<ProverInstances>::finalise_and_send_instance(std::shared_ptr<Instance> instance,
                                                                     const std::string& domain_separator)
{
    instance->initialize_prover_polynomials();

    const auto instance_size = static_cast<uint32_t>(instance->instance_size);
    const auto num_public_inputs = static_cast<uint32_t>(instance->public_inputs.size());
    transcript->send_to_verifier(domain_separator + "_instance_size", instance_size);
    transcript->send_to_verifier(domain_separator + "_public_input_size", num_public_inputs);

    for (size_t i = 0; i < instance->public_inputs.size(); ++i) {
        auto public_input_i = instance->public_inputs[i];
        transcript->send_to_verifier(domain_separator + "_public_input_" + std::to_string(i), public_input_i);
    }
    transcript->send_to_verifier(domain_separator + "_pub_inputs_offset",
                                 static_cast<uint32_t>(instance->pub_inputs_offset));

    auto& witness_commitments = instance->witness_commitments;

    // Commit to the first three wire polynomials of the instance
    // We only commit to the fourth wire polynomial after adding memory recordss
    witness_commitments.w_l = commitment_key->commit(instance->proving_key->w_l);
    witness_commitments.w_r = commitment_key->commit(instance->proving_key->w_r);
    witness_commitments.w_o = commitment_key->commit(instance->proving_key->w_o);

    auto wire_comms = witness_commitments.get_wires();
    auto commitment_labels = instance->commitment_labels;
    auto wire_labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        transcript->send_to_verifier(domain_separator + "_" + wire_labels[idx], wire_comms[idx]);
    }

    auto eta = transcript->get_challenge(domain_separator + "_eta");
    instance->compute_sorted_accumulator_polynomials(eta);

    // Commit to the sorted withness-table accumulator and the finalized (i.e. with memory records) fourth wire
    // polynomial
    witness_commitments.sorted_accum = commitment_key->commit(instance->prover_polynomials.sorted_accum);
    witness_commitments.w_4 = commitment_key->commit(instance->prover_polynomials.w_4);

    transcript->send_to_verifier(domain_separator + "_" + commitment_labels.sorted_accum,
                                 witness_commitments.sorted_accum);
    transcript->send_to_verifier(domain_separator + "_" + commitment_labels.w_4, witness_commitments.w_4);

    auto [beta, gamma] = transcript->get_challenges(domain_separator + "_beta", domain_separator + "_gamma");
    instance->compute_grand_product_polynomials(beta, gamma);

    witness_commitments.z_perm = commitment_key->commit(instance->prover_polynomials.z_perm);
    witness_commitments.z_lookup = commitment_key->commit(instance->prover_polynomials.z_lookup);

    transcript->send_to_verifier(domain_separator + "_" + commitment_labels.z_perm,
                                 instance->witness_commitments.z_perm);
    transcript->send_to_verifier(domain_separator + "_" + commitment_labels.z_lookup,
                                 instance->witness_commitments.z_lookup);
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        instance->alphas[idx] = transcript->get_challenge(domain_separator + "_alpha_" + std::to_string(idx));
    }
    auto vk_view = instance->verification_key->get_all();
    auto labels = instance->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < labels.size(); idx++) {
        transcript->send_to_verifier(domain_separator + "_" + labels[idx], vk_view[idx]);
    }
}

template <class ProverInstances>
void ProtoGalaxyProver_<ProverInstances>::send_accumulator(std::shared_ptr<Instance> instance,
                                                           const std::string& domain_separator)
{
    const auto instance_size = static_cast<uint32_t>(instance->instance_size);
    const auto num_public_inputs = static_cast<uint32_t>(instance->public_inputs.size());
    transcript->send_to_verifier(domain_separator + "_instance_size", instance_size);
    transcript->send_to_verifier(domain_separator + "_public_input_size", num_public_inputs);

    for (size_t i = 0; i < instance->public_inputs.size(); ++i) {
        auto public_input_i = instance->public_inputs[i];
        transcript->send_to_verifier(domain_separator + "_public_input_" + std::to_string(i), public_input_i);
    }

    transcript->send_to_verifier(domain_separator + "_eta", instance->relation_parameters.eta);
    transcript->send_to_verifier(domain_separator + "_beta", instance->relation_parameters.beta);
    transcript->send_to_verifier(domain_separator + "_gamma", instance->relation_parameters.gamma);
    transcript->send_to_verifier(domain_separator + "_public_input_delta",
                                 instance->relation_parameters.public_input_delta);
    transcript->send_to_verifier(domain_separator + "_lookup_grand_product_delta",
                                 instance->relation_parameters.lookup_grand_product_delta);

    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        transcript->send_to_verifier(domain_separator + "_alpha_" + std::to_string(idx), instance->alphas[idx]);
    }

    transcript->send_to_verifier(domain_separator + "_target_sum", instance->target_sum);
    for (size_t idx = 0; idx < instance->gate_challenges.size(); idx++) {
        transcript->send_to_verifier(domain_separator + "_gate_challenge_" + std::to_string(idx),
                                     instance->gate_challenges[idx]);
    }

    auto comm_view = instance->witness_commitments.get_all();
    auto witness_labels = instance->commitment_labels.get_witness();
    for (size_t idx = 0; idx < witness_labels.size(); idx++) {
        transcript->send_to_verifier(domain_separator + "_" + witness_labels[idx], comm_view[idx]);
    }

    auto vk_view = instance->verification_key->get_all();
    auto vk_labels = instance->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        transcript->send_to_verifier(domain_separator + "_" + vk_labels[idx], vk_view[idx]);
    }
}

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::prepare_for_folding()
{
    auto idx = 0;
    auto instance = instances[0];
    auto domain_separator = std::to_string(idx);
    transcript->send_to_verifier(domain_separator + "is_accumulator", instance->is_accumulator);
    if (instance->is_accumulator) {
        send_accumulator(instance, domain_separator);
    } else {
        // This is the first round of folding and we need to generate some gate challenges.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/740): implement option 2 to make this more
        // efficient by avoiding the computation of the perturbator
        finalise_and_send_instance(instance, domain_separator);
        instance->target_sum = 0;
        auto beta = transcript->get_challenge(domain_separator + "_initial_gate_challenge");
        std::vector<FF> gate_challenges(instance->log_instance_size);
        gate_challenges[0] = beta;
        for (size_t i = 1; i < instance->log_instance_size; i++) {
            gate_challenges[i] = gate_challenges[i - 1].sqr();
        }
        instance->gate_challenges = gate_challenges;
    }

    idx++;

    for (auto it = instances.begin() + 1; it != instances.end(); it++, idx++) {
        auto instance = *it;
        auto domain_separator = std::to_string(idx);
        finalise_and_send_instance(instance, domain_separator);
    }
}

template <class ProverInstances>
std::shared_ptr<typename ProverInstances::Instance> ProtoGalaxyProver_<ProverInstances>::compute_next_accumulator(
    ProverInstances& instances,
    Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM>& combiner_quotient,
    FF& challenge,
    const FF& compressed_perturbator)
{
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(challenge);

    // Given the challenge \gamma, compute Z(\gamma) and {L_0(\gamma),L_1(\gamma)}
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/764): Generalize the vanishing polynomial formula
    // and the computation of Lagrange basis for k instances
    auto vanishing_polynomial_at_challenge = challenge * (challenge - FF(1));
    std::vector<FF> lagranges{ FF(1) - challenge, challenge };

    auto next_accumulator = std::make_shared<Instance>();
    next_accumulator->is_accumulator = true;
    next_accumulator->instance_size = instances[0]->instance_size;
    next_accumulator->log_instance_size = instances[0]->log_instance_size;

    // Compute the next target sum and send the next folding parameters to the verifier
    FF next_target_sum =
        compressed_perturbator * lagranges[0] + vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;

    transcript->send_to_verifier("next_target_sum", next_target_sum);
    for (size_t idx = 0; idx < instances.next_gate_challenges.size(); idx++) {
        transcript->send_to_verifier("next_gate_challenge_" + std::to_string(idx), instances.next_gate_challenges[idx]);
    }
    next_accumulator->target_sum = next_target_sum;
    next_accumulator->gate_challenges = instances.next_gate_challenges;

    // Initialize prover polynomials
    ProverPolynomials acc_prover_polynomials;
    for (auto& polynomial : acc_prover_polynomials.get_all()) {
        polynomial = typename Flavor::Polynomial(instances[0]->instance_size);
    }

    // Fold the prover polynomials
    auto acc_poly_views = acc_prover_polynomials.get_all();
    for (size_t inst_idx = 0; inst_idx < ProverInstances::NUM; inst_idx++) {
        for (auto [acc_poly, inst_poly] :
             zip_view(acc_prover_polynomials.get_all(), instances[inst_idx]->prover_polynomials.get_all())) {
            for (auto [acc_el, inst_el] : zip_view(acc_poly, inst_poly)) {
                acc_el += inst_el * lagranges[inst_idx];
            }
        }
    }
    next_accumulator->prover_polynomials = std::move(acc_prover_polynomials);

    // Fold the witness commtiments and send them to the verifier
    auto witness_labels = next_accumulator->commitment_labels.get_witness();
    size_t comm_idx = 0;
    for (auto& acc_comm : next_accumulator->witness_commitments.get_all()) {
        acc_comm = Commitment::infinity();
        size_t inst_idx = 0;
        for (auto& instance : instances) {
            acc_comm = acc_comm + instance->witness_commitments.get_all()[comm_idx] * lagranges[inst_idx];
            inst_idx++;
        }
        transcript->send_to_verifier("next_" + witness_labels[comm_idx], acc_comm);
        comm_idx++;
    }

    // Fold public data ϕ from all instances to produce ϕ* and add it to the transcript. As part of the folding
    // verification, the verifier will produce ϕ* as well and check it against what was sent by the prover.

    // Fold the public inputs and send to the verifier
    next_accumulator->public_inputs = std::vector<FF>(instances[0]->public_inputs.size(), 0);
    size_t el_idx = 0;
    for (auto& el : next_accumulator->public_inputs) {
        size_t inst = 0;
        for (auto& instance : instances) {
            el += instance->public_inputs[el_idx] * lagranges[inst];
            inst++;
        }
        transcript->send_to_verifier("next_public_input_" + std::to_string(el_idx), el);
        el_idx++;
    }

    // Evaluate the combined batching  α_i univariate at challenge to obtain next α_i and send it to the
    // verifier, where i ∈ {0,...,NUM_SUBRELATIONS - 1}
    auto& folded_alphas = next_accumulator->alphas;
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        folded_alphas[idx] = instances.alphas[idx].evaluate(challenge);
        transcript->send_to_verifier("next_alpha_" + std::to_string(idx), folded_alphas[idx]);
    }

    // Evaluate each relation parameter univariate at challenge to obtain the folded relation parameters and send to
    // the verifier
    auto& combined_relation_parameters = instances.relation_parameters;
    auto folded_relation_parameters = bb::RelationParameters<FF>{
        combined_relation_parameters.eta.evaluate(challenge),
        combined_relation_parameters.beta.evaluate(challenge),
        combined_relation_parameters.gamma.evaluate(challenge),
        combined_relation_parameters.public_input_delta.evaluate(challenge),
        combined_relation_parameters.lookup_grand_product_delta.evaluate(challenge),
    };
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/805): Add the relation parameters to the transcript
    // together.
    transcript->send_to_verifier("next_eta", folded_relation_parameters.eta);
    transcript->send_to_verifier("next_beta", folded_relation_parameters.beta);
    transcript->send_to_verifier("next_gamma", folded_relation_parameters.gamma);
    transcript->send_to_verifier("next_public_input_delta", folded_relation_parameters.public_input_delta);
    transcript->send_to_verifier("next_lookup_grand_product_delta",
                                 folded_relation_parameters.lookup_grand_product_delta);
    next_accumulator->relation_parameters = folded_relation_parameters;

    // Fold the verification key and send it to the verifier as this is part of ϕ as well
    auto acc_vk = std::make_shared<VerificationKey>(instances[0]->prover_polynomials.get_polynomial_size(),
                                                    instances[0]->public_inputs.size());
    auto labels = next_accumulator->commitment_labels.get_precomputed();
    size_t vk_idx = 0;
    for (auto& vk : acc_vk->get_all()) {
        size_t inst = 0;
        vk = Commitment::infinity();
        for (auto& instance : instances) {
            vk = vk + (instance->verification_key->get_all()[vk_idx]) * lagranges[inst];
            inst++;
        }
        transcript->send_to_verifier("next_" + labels[vk_idx], vk);
        vk_idx++;
    }
    next_accumulator->verification_key = acc_vk;
    return next_accumulator;
}

// TODO(#https://github.com/AztecProtocol/barretenberg/issues/689): finalise implementation this function
template <class ProverInstances>
FoldingResult<typename ProverInstances::Flavor> ProtoGalaxyProver_<ProverInstances>::fold_instances()
{
    prepare_for_folding();

    // TODO(#https://github.com/AztecProtocol/barretenberg/issues/740): Handle the case where we are folding for the
    // first time and accumulator is 0
    FF delta = transcript->get_challenge("delta");

    auto accumulator = get_accumulator();
    auto deltas = compute_round_challenge_pows(accumulator->log_instance_size, delta);

    auto perturbator = compute_perturbator(accumulator, deltas);
    for (size_t idx = 0; idx <= accumulator->log_instance_size; idx++) {
        transcript->send_to_verifier("perturbator_" + std::to_string(idx), perturbator[idx]);
    }
    auto perturbator_challenge = transcript->get_challenge("perturbator_challenge");
    instances.next_gate_challenges =
        update_gate_challenges(perturbator_challenge, accumulator->gate_challenges, deltas);
    combine_relation_parameters(instances);
    combine_alpha(instances);
    auto pow_polynomial = PowPolynomial<FF>(instances.next_gate_challenges);
    auto combiner = compute_combiner(instances, pow_polynomial);

    auto compressed_perturbator = perturbator.evaluate(perturbator_challenge);
    auto combiner_quotient = compute_combiner_quotient(compressed_perturbator, combiner);

    for (size_t idx = ProverInstances::NUM; idx < ProverInstances::BATCHED_EXTENDED_LENGTH; idx++) {
        transcript->send_to_verifier("combiner_quotient_" + std::to_string(idx), combiner_quotient.value_at(idx));
    }
    FF combiner_challenge = transcript->get_challenge("combiner_quotient_challenge");

    FoldingResult<Flavor> res;
    auto next_accumulator =
        compute_next_accumulator(instances, combiner_quotient, combiner_challenge, compressed_perturbator);
    res.folding_data = transcript->proof_data;
    res.accumulator = next_accumulator;

    return res;
}

template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::Ultra, 2>>;
template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace bb::honk