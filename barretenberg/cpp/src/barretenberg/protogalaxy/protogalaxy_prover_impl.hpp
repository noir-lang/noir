#pragma once
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover_internal.hpp"
#include "barretenberg/protogalaxy/prover_verifier_shared.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/ultra_honk/oink_prover.hpp"
#include "protogalaxy_prover.hpp"

namespace bb {
template <class ProverInstances>
void ProtoGalaxyProver_<ProverInstances>::finalise_and_send_instance(std::shared_ptr<Instance> instance,
                                                                     const std::string& domain_separator)
{
    ZoneScopedN("ProtoGalaxyProver::finalise_and_send_instance");
    OinkProver<Flavor> oink_prover(instance, transcript, domain_separator + '_');

    oink_prover.prove();
}

template <class ProverInstances>
FoldingResult<typename ProverInstances::Flavor> ProtoGalaxyProver_<ProverInstances>::update_target_sum_and_fold(
    const ProverInstances& instances,
    const CombinerQuotient& combiner_quotient,
    const UnivariateRelationSeparator& alphas,
    const UnivariateRelationParameters& univariate_relation_parameters,
    const FF& perturbator_evaluation)
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::update_target_sum_and_fold");
    using Fun = ProtogalaxyProverInternal<ProverInstances>;

    const FF combiner_challenge = transcript->template get_challenge<FF>("combiner_quotient_challenge");

    FoldingResult<Flavor> result{ .accumulator = instances[0], .proof = std::move(transcript->proof_data) };

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/881): bad pattern
    result.accumulator->is_accumulator = true;

    // Compute the next target sum
    auto [vanishing_polynomial_at_challenge, lagranges] =
        Fun::compute_vanishing_polynomial_and_lagranges(combiner_challenge);
    result.accumulator->target_sum = perturbator_evaluation * lagranges[0] +
                                     vanishing_polynomial_at_challenge * combiner_quotient.evaluate(combiner_challenge);

    // Fold the proving key polynomials
    for (auto& poly : result.accumulator->proving_key.polynomials.get_unshifted()) {
        poly *= lagranges[0];
    }
    for (size_t inst_idx = 1; inst_idx < ProverInstances::NUM; inst_idx++) {
        for (auto [acc_poly, inst_poly] : zip_view(result.accumulator->proving_key.polynomials.get_unshifted(),
                                                   instances[inst_idx]->proving_key.polynomials.get_unshifted())) {
            acc_poly.add_scaled(inst_poly, lagranges[inst_idx]);
        }
    }

    // Evaluate the combined batching  α_i univariate at challenge to obtain next α_i and send it to the
    // verifier, where i ∈ {0,...,NUM_SUBRELATIONS - 1}
    for (auto [folded_alpha, inst_alpha] : zip_view(result.accumulator->alphas, alphas)) {
        folded_alpha = inst_alpha.evaluate(combiner_challenge);
    }

    // Evaluate each relation parameter univariate at challenge to obtain the folded relation parameters.
    for (auto [univariate, value] : zip_view(univariate_relation_parameters.get_to_fold(),
                                             result.accumulator->relation_parameters.get_to_fold())) {
        value = univariate.evaluate(combiner_challenge);
    }

    return result;
}

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::run_oink_prover_on_each_instance()
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::run_oink_prover_on_each_instance");
    auto idx = 0;
    auto& instance = instances[0];
    auto domain_separator = std::to_string(idx);

    if (!instance->is_accumulator) {
        finalise_and_send_instance(instance, domain_separator);
        instance->target_sum = 0;
        instance->gate_challenges = std::vector<FF>(instance->proving_key.log_circuit_size, 0);
    }

    idx++;

    for (auto it = instances.begin() + 1; it != instances.end(); it++, idx++) {
        auto instance = *it;
        auto domain_separator = std::to_string(idx);
        finalise_and_send_instance(instance, domain_separator);
    }

    state.accumulator = instances[0];
};

template <class ProverInstances>
std::tuple<std::vector<typename ProverInstances::Flavor::FF>, Polynomial<typename ProverInstances::Flavor::FF>>
ProtoGalaxyProver_<ProverInstances>::perturbator_round(
    const std::shared_ptr<const typename ProverInstances::Instance>& accumulator)
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::perturbator_round");

    using Fun = ProtogalaxyProverInternal<ProverInstances>;

    const FF delta = transcript->template get_challenge<FF>("delta");
    const std::vector<FF> deltas = compute_round_challenge_pows(accumulator->proving_key.log_circuit_size, delta);
    // An honest prover with valid initial instances computes that the perturbator is 0 in the first round
    const Polynomial<FF> perturbator = accumulator->is_accumulator
                                           ? Fun::compute_perturbator(accumulator, deltas)
                                           : Polynomial<FF>(accumulator->proving_key.log_circuit_size + 1);
    // Prover doesn't send the constant coefficient of F because this is supposed to be equal to the target sum of
    // the accumulator which the folding verifier has from the previous iteration.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1087): Verifier circuit for first IVC step is different
    if (accumulator->is_accumulator) {
        for (size_t idx = 1; idx <= accumulator->proving_key.log_circuit_size; idx++) {
            transcript->send_to_verifier("perturbator_" + std::to_string(idx), perturbator[idx]);
        }
    }

    return std::make_tuple(deltas, perturbator);
};

template <class ProverInstances>
std::tuple<std::vector<typename ProverInstances::Flavor::FF>,
           typename ProtoGalaxyProver_<ProverInstances>::UnivariateRelationSeparator,
           typename ProtoGalaxyProver_<ProverInstances>::UnivariateRelationParameters,
           typename ProverInstances::Flavor::FF,
           typename ProtoGalaxyProver_<ProverInstances>::CombinerQuotient>
ProtoGalaxyProver_<ProverInstances>::combiner_quotient_round(const std::vector<FF>& gate_challenges,
                                                             const std::vector<FF>& deltas,
                                                             const ProverInstances& instances)
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::combiner_quotient_round");

    using Fun = ProtogalaxyProverInternal<ProverInstances>;

    const FF perturbator_challenge = transcript->template get_challenge<FF>("perturbator_challenge");

    const std::vector<FF> updated_gate_challenges =
        update_gate_challenges(perturbator_challenge, gate_challenges, deltas);
    const UnivariateRelationSeparator alphas = Fun::compute_and_extend_alphas(instances);
    const PowPolynomial<FF> pow_polynomial{ updated_gate_challenges, instances[0]->proving_key.log_circuit_size };
    const UnivariateRelationParameters relation_parameters =
        Fun::template compute_extended_relation_parameters<UnivariateRelationParameters>(instances);

    OptimisedTupleOfTuplesOfUnivariates accumulators;
    auto combiner = Fun::compute_combiner(instances, pow_polynomial, relation_parameters, alphas, accumulators);

    const FF perturbator_evaluation = state.perturbator.evaluate(perturbator_challenge);
    const CombinerQuotient combiner_quotient = Fun::compute_combiner_quotient(perturbator_evaluation, combiner);

    for (size_t idx = ProverInstances::NUM; idx < ProverInstances::BATCHED_EXTENDED_LENGTH; idx++) {
        transcript->send_to_verifier("combiner_quotient_" + std::to_string(idx), combiner_quotient.value_at(idx));
    }

    return std::make_tuple(
        updated_gate_challenges, alphas, relation_parameters, perturbator_evaluation, combiner_quotient);
}

template <class ProverInstances>
FoldingResult<typename ProverInstances::Flavor> ProtoGalaxyProver_<ProverInstances>::prove()
{
    ZoneScopedN("ProtogalaxyProver::prove");
    BB_OP_COUNT_TIME_NAME("ProtogalaxyProver::prove");
    // Ensure instances are all of the same size
    for (size_t idx = 0; idx < ProverInstances::NUM - 1; ++idx) {
        if (instances[idx]->proving_key.circuit_size != instances[idx + 1]->proving_key.circuit_size) {
            info("ProtogalaxyProver: circuit size mismatch!");
            info("Instance ", idx, " size = ", instances[idx]->proving_key.circuit_size);
            info("Instance ", idx + 1, " size = ", instances[idx + 1]->proving_key.circuit_size);
            ASSERT(false);
        }
    }
    run_oink_prover_on_each_instance();

    std::tie(state.deltas, state.perturbator) = perturbator_round(state.accumulator);

    std::tie(state.accumulator->gate_challenges,
             state.alphas,
             state.relation_parameters,
             state.perturbator_evaluation,
             state.combiner_quotient) =
        combiner_quotient_round(state.accumulator->gate_challenges, state.deltas, instances);

    const FoldingResult<Flavor> result = update_target_sum_and_fold(
        instances, state.combiner_quotient, state.alphas, state.relation_parameters, state.perturbator_evaluation);

    return result;
}
} // namespace bb