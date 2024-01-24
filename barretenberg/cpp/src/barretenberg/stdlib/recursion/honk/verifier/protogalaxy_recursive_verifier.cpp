#include "protogalaxy_recursive_verifier.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/library/grand_product_delta.hpp"
namespace bb::stdlib::recursion::honk {

template <class VerifierInstances>
void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::receive_accumulator(const std::shared_ptr<Instance>& inst,
                                                                           const std::string& domain_separator)
{
    // Get circuit parameters
    const auto instance_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "_instance_size");
    const auto public_input_size =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");
    inst->instance_size = uint32_t(instance_size.get_value());
    inst->log_instance_size = uint32_t(numeric::get_msb(inst->instance_size));
    inst->public_input_size = uint32_t(public_input_size.get_value());

    // Get folded public inputs
    for (size_t i = 0; i < inst->public_input_size; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    // Get folded relation parameters
    auto eta = transcript->template receive_from_prover<FF>(domain_separator + "_eta");
    auto beta = transcript->template receive_from_prover<FF>(domain_separator + "_beta");
    auto gamma = transcript->template receive_from_prover<FF>(domain_separator + "_gamma");
    auto public_input_delta = transcript->template receive_from_prover<FF>(domain_separator + "_public_input_delta");
    auto lookup_grand_product_delta =
        transcript->template receive_from_prover<FF>(domain_separator + "_lookup_grand_product_delta");
    inst->relation_parameters =
        RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };

    // Get the folded relation separator challenges \vec{α}
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] =
            transcript->template receive_from_prover<FF>(domain_separator + "_alpha_" + std::to_string(idx));
    }

    inst->target_sum = transcript->template receive_from_prover<FF>(domain_separator + "_target_sum");

    // Get the folded gate challenges, \vec{β} in the paper
    inst->gate_challenges = std::vector<FF>(inst->log_instance_size);
    for (size_t idx = 0; idx < inst->log_instance_size; idx++) {
        inst->gate_challenges[idx] =
            transcript->template receive_from_prover<FF>(domain_separator + "_gate_challenge_" + std::to_string(idx));
    }

    // Get the folded commitments to all witness polynomials
    auto comm_view = inst->witness_commitments.get_all();
    auto witness_labels = inst->commitment_labels.get_witness();
    for (size_t idx = 0; idx < witness_labels.size(); idx++) {
        comm_view[idx] =
            transcript->template receive_from_prover<Commitment>(domain_separator + "_" + witness_labels[idx]);
    }

    // Get the folded commitments to selector polynomials
    inst->verification_key = std::make_shared<VerificationKey>(inst->instance_size, inst->public_input_size);
    auto vk_view = inst->verification_key->get_all();
    auto vk_labels = inst->commitment_labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        vk_view[idx] = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + vk_labels[idx]);
    }
}

template <class VerifierInstances>
void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::receive_and_finalise_instance(
    const std::shared_ptr<Instance>& inst, const std::string& domain_separator)
{
    // Get circuit parameters and the public inputs
    const auto instance_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "_instance_size");
    const auto public_input_size =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");
    inst->instance_size = uint32_t(instance_size.get_value());
    inst->log_instance_size = static_cast<size_t>(numeric::get_msb(inst->instance_size));
    inst->public_input_size = uint32_t(public_input_size.get_value());

    for (size_t i = 0; i < inst->public_input_size; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    const auto pub_inputs_offset =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "_pub_inputs_offset");

    inst->pub_inputs_offset = uint32_t(pub_inputs_offset.get_value());

    // Get commitments to first three wire polynomials
    auto labels = inst->commitment_labels;
    auto& witness_commitments = inst->witness_commitments;
    witness_commitments.w_l = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_l);
    witness_commitments.w_r = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_r);
    witness_commitments.w_o = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_o);

    // Get challenge for sorted list batching and wire four memory records commitment
    auto eta = transcript->get_challenge(domain_separator + "_eta");
    witness_commitments.sorted_accum =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.sorted_accum);
    witness_commitments.w_4 = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_4);

    // Get permutation challenges and commitment to permutation and lookup grand products
    auto [beta, gamma] = transcript->get_challenges(domain_separator + "_beta", domain_separator + "_gamma");
    witness_commitments.z_perm =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.z_perm);
    witness_commitments.z_lookup =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.z_lookup);

    // Compute correction terms for grand products
    const FF public_input_delta = bb::honk::compute_public_input_delta<Flavor>(
        inst->public_inputs, beta, gamma, inst->instance_size, inst->pub_inputs_offset);
    const FF lookup_grand_product_delta =
        bb::honk::compute_lookup_grand_product_delta<FF>(beta, gamma, inst->instance_size);
    inst->relation_parameters =
        RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };

    // Get the relation separation challenges
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] = transcript->get_challenge(domain_separator + "_alpha_" + std::to_string(idx));
    }

    // Get the commitments to the selector polynomials for the given instance
    inst->verification_key = std::make_shared<VerificationKey>(inst->instance_size, inst->public_input_size);
    auto vk_view = inst->verification_key->get_all();
    auto vk_labels = labels.get_precomputed();
    for (size_t idx = 0; idx < vk_labels.size(); idx++) {
        vk_view[idx] = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + vk_labels[idx]);
    }
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/795): The rounds prior to actual verifying are common
// between decider and folding verifier and could be somehow shared so we do not duplicate code so much.
template <class VerifierInstances> void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::prepare_for_folding()
{
    auto index = 0;
    auto inst = instances[0];
    auto domain_separator = std::to_string(index);
    const auto is_accumulator = transcript->template receive_from_prover<bool>(domain_separator + "is_accumulator");
    inst->is_accumulator = static_cast<bool>(is_accumulator.get_value());
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
void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::verify_folding_proof(std::vector<uint8_t> proof)
{
    using Transcript = typename Flavor::Transcript;
    using ElementNative = typename Flavor::Curve::ElementNative;
    using AffineElementNative = typename Flavor::Curve::AffineElementNative;
    using ScalarNative = typename Flavor::Curve::ScalarFieldNative;

    transcript = std::make_shared<Transcript>(builder, proof);
    prepare_for_folding();

    auto delta = transcript->get_challenge("delta");
    auto accumulator = get_accumulator();
    auto deltas = compute_round_challenge_pows(accumulator->log_instance_size, delta);

    std::vector<FF> perturbator_coeffs(accumulator->log_instance_size + 1);
    for (size_t idx = 0; idx <= accumulator->log_instance_size; idx++) {
        perturbator_coeffs[idx] = transcript->template receive_from_prover<FF>("perturbator_" + std::to_string(idx));
    }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/833): As currently the stdlib transcript is not
    // creating proper constraints linked to Fiat-Shamir we add an additonal gate to ensure assert_equal is correct.
    // This comparison to 0 can be removed here and below once we have merged the transcript.
    auto zero = FF::from_witness(builder, ScalarNative(0));
    zero.assert_equal(accumulator->target_sum - perturbator_coeffs[0], "F(0) != e");

    FF perturbator_challenge = transcript->get_challenge("perturbator_challenge");

    auto perturbator_at_challenge = evaluate_perturbator(perturbator_coeffs, perturbator_challenge);
    // The degree of K(X) is dk - k - 1 = k(d - 1) - 1. Hence we need  k(d - 1) evaluations to represent it.
    std::array<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM> combiner_quotient_evals;
    for (size_t idx = 0; idx < VerifierInstances::BATCHED_EXTENDED_LENGTH - VerifierInstances::NUM; idx++) {
        combiner_quotient_evals[idx] = transcript->template receive_from_prover<FF>(
            "combiner_quotient_" + std::to_string(idx + VerifierInstances::NUM));
    }
    Univariate<FF, VerifierInstances::BATCHED_EXTENDED_LENGTH, VerifierInstances::NUM> combiner_quotient(
        combiner_quotient_evals);
    FF combiner_challenge = transcript->get_challenge("combiner_quotient_challenge");
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(combiner_challenge); // fine recursive i think

    auto vanishing_polynomial_at_challenge = combiner_challenge * (combiner_challenge - FF(1));
    auto lagranges = std::vector<FF>{ FF(1) - combiner_challenge, combiner_challenge };

    // Compute next folding parameters and verify against the ones received from the prover
    auto expected_next_target_sum =
        perturbator_at_challenge * lagranges[0] + vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;
    auto next_target_sum = transcript->template receive_from_prover<FF>("next_target_sum");
    zero.assert_equal(expected_next_target_sum - next_target_sum, "next target sum mismatch");

    auto expected_betas_star = update_gate_challenges(perturbator_challenge, accumulator->gate_challenges, deltas);
    for (size_t idx = 0; idx < accumulator->log_instance_size; idx++) {
        auto beta_star = transcript->template receive_from_prover<FF>("next_gate_challenge_" + std::to_string(idx));
        zero.assert_equal(beta_star - expected_betas_star[idx],
                          " next gate challenge mismatch at: " + std::to_string(idx));
    }

    // Compute ϕ and verify against the data received from the prover
    WitnessCommitments acc_witness_commitments;
    auto witness_labels = commitment_labels.get_witness();
    size_t comm_idx = 0;
    auto random_generator = Commitment::from_witness(builder, AffineElementNative(ElementNative::random_element()));
    for (auto& expected_comm : acc_witness_commitments.get_all()) {
        expected_comm = random_generator;
        size_t inst = 0;
        for (auto& instance : instances) {
            expected_comm = expected_comm + instance->witness_commitments.get_all()[comm_idx] * lagranges[inst];
            inst++;
        }
        auto comm = transcript->template receive_from_prover<Commitment>("next_" + witness_labels[comm_idx]);
        auto res = expected_comm - comm;
        random_generator.x.assert_equal(res.x);
        random_generator.y.assert_equal(res.y);
        comm_idx++;
    }

    std::vector<FF> folded_public_inputs(instances[0]->public_inputs.size(), 0);
    size_t public_input_idx = 0;
    for (auto& expected_public_input : folded_public_inputs) {
        size_t inst = 0;
        for (auto& instance : instances) {
            expected_public_input += instance->public_inputs[public_input_idx] * lagranges[inst];
            inst++;
        }
        auto next_public_input =
            transcript->template receive_from_prover<FF>("next_public_input" + std::to_string(public_input_idx));
        zero.assert_equal(expected_public_input - next_public_input,
                          "folded public input mismatch at: " + std::to_string(public_input_idx));
        public_input_idx++;
    }

    for (size_t alpha_idx = 0; alpha_idx < NUM_SUBRELATIONS - 1; alpha_idx++) {
        FF expected_alpha(0);
        size_t instance_idx = 0;
        for (auto& instance : instances) {
            expected_alpha += instance->alphas[alpha_idx] * lagranges[instance_idx];
            instance_idx++;
        }
        auto next_alpha = transcript->template receive_from_prover<FF>("next_alpha_" + std::to_string(alpha_idx));
        zero.assert_equal(expected_alpha - next_alpha,
                          "folded relation separator mismatch at: " + std::to_string(alpha_idx));
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
    zero.assert_equal(expected_parameters.eta - next_eta, "relation parameter eta mismatch");

    auto next_beta = transcript->template receive_from_prover<FF>("next_beta");
    zero.assert_equal(expected_parameters.beta - next_beta, "relation parameter beta mismatch");

    auto next_gamma = transcript->template receive_from_prover<FF>("next_gamma");
    zero.assert_equal(expected_parameters.gamma - next_gamma, "relation parameter gamma mismatch");

    auto next_public_input_delta = transcript->template receive_from_prover<FF>("next_public_input_delta");
    zero.assert_equal(expected_parameters.public_input_delta - next_public_input_delta,
                      "relation parameter public input delta mismatch");

    auto next_lookup_grand_product_delta =
        transcript->template receive_from_prover<FF>("next_lookup_grand_product_delta");
    zero.assert_equal(expected_parameters.lookup_grand_product_delta - next_lookup_grand_product_delta,
                      "relation parameter lookup grand product delta mismatch");

    auto acc_vk = std::make_shared<VerificationKey>(instances[0]->instance_size, instances[0]->public_input_size);
    auto vk_labels = commitment_labels.get_precomputed();
    size_t vk_idx = 0;
    for (auto& expected_vk : acc_vk->get_all()) {
        size_t inst = 0;
        expected_vk = random_generator;
        for (auto& instance : instances) {
            expected_vk = expected_vk + instance->verification_key->get_all()[vk_idx] * lagranges[inst];
            inst++;
        }
        auto vk = transcript->template receive_from_prover<Commitment>("next_" + vk_labels[vk_idx]);
        auto res = expected_vk - vk;
        random_generator.x.assert_equal(res.x);
        random_generator.y.assert_equal(res.y);
        vk_idx++;
    }
}

template class ProtoGalaxyRecursiveVerifier_<
    bb::honk::VerifierInstances_<bb::honk::flavor::UltraRecursive_<GoblinUltraCircuitBuilder>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<
    bb::honk::VerifierInstances_<bb::honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>, 2>>;
} // namespace bb::stdlib::recursion::honk