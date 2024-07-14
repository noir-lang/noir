#include "protogalaxy_recursive_verifier.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/recursive_instances.hpp"

namespace bb::stdlib::recursion::honk {

template <class VerifierInstances>
void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::receive_and_finalise_instance(
    const std::shared_ptr<Instance>& inst, const std::string& domain_separator)
{
    // Get circuit parameters and the public inputs
    const auto instance_size = transcript->template receive_from_prover<FF>(domain_separator + "_circuit_size");
    const auto public_input_size =
        transcript->template receive_from_prover<FF>(domain_separator + "_public_input_size");
    inst->verification_key->circuit_size = uint32_t(instance_size.get_value());
    inst->verification_key->log_circuit_size =
        static_cast<size_t>(numeric::get_msb(inst->verification_key->circuit_size));
    inst->verification_key->num_public_inputs = uint32_t(public_input_size.get_value());
    const auto pub_inputs_offset =
        transcript->template receive_from_prover<FF>(domain_separator + "_pub_inputs_offset");
    inst->verification_key->pub_inputs_offset = uint32_t(pub_inputs_offset.get_value());

    for (size_t i = 0; i < inst->verification_key->num_public_inputs; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
        inst->public_inputs.emplace_back(public_input_i);
    }

    // Get commitments to first three wire polynomials
    auto labels = inst->commitment_labels;
    auto& witness_commitments = inst->witness_commitments;
    witness_commitments.w_l = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_l);
    witness_commitments.w_r = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_r);
    witness_commitments.w_o = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_o);

    if constexpr (IsGoblinFlavor<Flavor>) {
        // Receive ECC op wire commitments
        for (auto [commitment, label] : zip_view(witness_commitments.get_ecc_op_wires(), labels.get_ecc_op_wires())) {
            commitment = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + label);
        }

        // Receive DataBus related polynomial commitments
        for (auto [commitment, label] :
             zip_view(witness_commitments.get_databus_entities(), labels.get_databus_entities())) {
            commitment = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + label);
        }
    }

    // Get eta challenges
    auto [eta, eta_two, eta_three] = transcript->template get_challenges<FF>(
        domain_separator + "_eta", domain_separator + "_eta_two", domain_separator + "_eta_three");

    // Receive commitments to lookup argument polynomials
    witness_commitments.lookup_read_counts =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.lookup_read_counts);
    witness_commitments.lookup_read_tags =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.lookup_read_tags);

    // Receive commitments to wire 4
    witness_commitments.w_4 = transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.w_4);

    // Get permutation challenges and commitment to permutation and lookup grand products
    auto [beta, gamma] =
        transcript->template get_challenges<FF>(domain_separator + "_beta", domain_separator + "_gamma");

    witness_commitments.lookup_inverses = transcript->template receive_from_prover<Commitment>(
        domain_separator + "_" + commitment_labels.lookup_inverses);

    // If Goblin (i.e. using DataBus) receive commitments to log-deriv inverses polynomial
    if constexpr (IsGoblinFlavor<Flavor>) {
        witness_commitments.calldata_inverses = transcript->template receive_from_prover<Commitment>(
            domain_separator + "_" + commitment_labels.calldata_inverses);
        witness_commitments.return_data_inverses = transcript->template receive_from_prover<Commitment>(
            domain_separator + "_" + commitment_labels.return_data_inverses);
    }

    witness_commitments.z_perm =
        transcript->template receive_from_prover<Commitment>(domain_separator + "_" + labels.z_perm);

    // Compute correction terms for grand products
    const FF public_input_delta =
        compute_public_input_delta<Flavor>(inst->public_inputs,
                                           beta,
                                           gamma,
                                           inst->verification_key->circuit_size,
                                           static_cast<size_t>(inst->verification_key->pub_inputs_offset));
    inst->relation_parameters = RelationParameters<FF>{ eta, eta_two, eta_three, beta, gamma, public_input_delta };

    // Get the relation separation challenges
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        inst->alphas[idx] = transcript->template get_challenge<FF>(domain_separator + "_alpha_" + std::to_string(idx));
    }
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/795): The rounds prior to actual verifying are common
// between decider and folding verifier and could be somehow shared so we do not duplicate code so much.
template <class VerifierInstances> void ProtoGalaxyRecursiveVerifier_<VerifierInstances>::prepare_for_folding()
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
std::shared_ptr<typename VerifierInstances::Instance> ProtoGalaxyRecursiveVerifier_<
    VerifierInstances>::verify_folding_proof(const HonkProof& proof)
{
    using Transcript = typename Flavor::Transcript;

    StdlibProof<Builder> stdlib_proof = bb::convert_proof_to_witness(builder, proof);
    transcript = std::make_shared<Transcript>(stdlib_proof);
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

template class ProtoGalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<UltraRecursiveFlavor_<UltraCircuitBuilder>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances_<MegaRecursiveFlavor_<MegaCircuitBuilder>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances_<UltraRecursiveFlavor_<MegaCircuitBuilder>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances_<MegaRecursiveFlavor_<UltraCircuitBuilder>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<UltraRecursiveFlavor_<CircuitSimulatorBN254>, 2>>;
template class ProtoGalaxyRecursiveVerifier_<
    RecursiveVerifierInstances_<MegaRecursiveFlavor_<CircuitSimulatorBN254>, 2>>;
} // namespace bb::stdlib::recursion::honk