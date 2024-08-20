#pragma once
#include "barretenberg/common/container.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/ultra_honk/oink_prover.hpp"
#include "protogalaxy_prover.hpp"
namespace bb {
// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
std::vector<typename ProtoGalaxyProver_<ProverInstances_>::FF> ProtoGalaxyProver_<
    ProverInstances_>::compute_full_honk_evaluations(const ProverPolynomials& instance_polynomials,
                                                     const RelationSeparator& alpha,
                                                     const RelationParameters<FF>& relation_parameters)
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::compute_full_honk_evaluations");
    auto instance_size = instance_polynomials.get_polynomial_size();
    std::vector<FF> full_honk_evaluations(instance_size);
    std::vector<FF> linearly_dependent_contribution_accumulators = parallel_for_heuristic(
        instance_size,
        /*accumulator default*/ FF(0),
        [&](size_t row, FF& linearly_dependent_contribution_accumulator) {
            auto row_evaluations = instance_polynomials.get_row(row);
            RelationEvaluations relation_evaluations;
            Utils::zero_elements(relation_evaluations);

            Utils::template accumulate_relation_evaluations<>(
                row_evaluations, relation_evaluations, relation_parameters, FF(1));

            auto output = FF(0);
            auto running_challenge = FF(1);
            Utils::scale_and_batch_elements(
                relation_evaluations, alpha, running_challenge, output, linearly_dependent_contribution_accumulator);

            full_honk_evaluations[row] = output;
        },
        thread_heuristics::ALWAYS_MULTITHREAD);
    full_honk_evaluations[0] += sum(linearly_dependent_contribution_accumulators);
    return full_honk_evaluations;
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
std::vector<typename ProtoGalaxyProver_<ProverInstances_>::FF> ProtoGalaxyProver_<
    ProverInstances_>::construct_coefficients_tree(const std::vector<FF>& betas,
                                                   const std::vector<FF>& deltas,
                                                   const std::vector<std::vector<FF>>& prev_level_coeffs,
                                                   size_t level)
{
    if (level == betas.size()) {
        return prev_level_coeffs[0];
    }

    auto degree = level + 1;
    auto prev_level_width = prev_level_coeffs.size();
    std::vector<std::vector<FF>> level_coeffs(prev_level_width / 2, std::vector<FF>(degree + 1, 0));
    parallel_for_heuristic(
        prev_level_width / 2,
        [&](size_t parent) {
            size_t node = parent * 2;
            std::copy(prev_level_coeffs[node].begin(), prev_level_coeffs[node].end(), level_coeffs[parent].begin());
            for (size_t d = 0; d < degree; d++) {
                level_coeffs[parent][d] += prev_level_coeffs[node + 1][d] * betas[level];
                level_coeffs[parent][d + 1] += prev_level_coeffs[node + 1][d] * deltas[level];
            }
        },
        /* overestimate */ thread_heuristics::FF_MULTIPLICATION_COST * degree * 3);
    return construct_coefficients_tree(betas, deltas, level_coeffs, level + 1);
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
std::vector<typename ProtoGalaxyProver_<ProverInstances_>::FF> ProtoGalaxyProver_<
    ProverInstances_>::construct_perturbator_coefficients(const std::vector<FF>& betas,
                                                          const std::vector<FF>& deltas,
                                                          const std::vector<FF>& full_honk_evaluations)
{
    auto width = full_honk_evaluations.size();
    std::vector<std::vector<FF>> first_level_coeffs(width / 2, std::vector<FF>(2, 0));
    parallel_for_heuristic(
        width / 2,
        [&](size_t parent) {
            size_t node = parent * 2;
            first_level_coeffs[parent][0] = full_honk_evaluations[node] + full_honk_evaluations[node + 1] * betas[0];
            first_level_coeffs[parent][1] = full_honk_evaluations[node + 1] * deltas[0];
        },
        /* overestimate */ thread_heuristics::FF_MULTIPLICATION_COST * 3);
    return construct_coefficients_tree(betas, deltas, first_level_coeffs);
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
LegacyPolynomial<typename ProtoGalaxyProver_<ProverInstances_>::FF> ProtoGalaxyProver_<
    ProverInstances_>::compute_perturbator(const std::shared_ptr<Instance> accumulator, const std::vector<FF>& deltas)
{
    BB_OP_COUNT_TIME();
    auto full_honk_evaluations = compute_full_honk_evaluations(
        accumulator->proving_key.polynomials, accumulator->alphas, accumulator->relation_parameters);
    const auto betas = accumulator->gate_challenges;
    assert(betas.size() == deltas.size());
    auto coeffs = construct_perturbator_coefficients(betas, deltas, full_honk_evaluations);
    return LegacyPolynomial<FF>(coeffs);
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
void ProtoGalaxyProver_<ProverInstances_>::deoptimise_univariates(
    const OptimisedTupleOfTuplesOfUnivariates& optimised_univariate_accumulators,
    TupleOfTuplesOfUnivariates& new_univariate_accumulators)
{
    auto deoptimise = [&]<size_t outer_idx, size_t inner_idx>(auto& element) {
        auto& optimised_element = std::get<inner_idx>(std::get<outer_idx>(optimised_univariate_accumulators));
        element = optimised_element.convert();
    };

    Utils::template apply_to_tuple_of_tuples<0, 0>(new_univariate_accumulators, deoptimise);
}

template <class ProverInstances_>
ProtoGalaxyProver_<ProverInstances_>::ExtendedUnivariateWithRandomization ProtoGalaxyProver_<
    ProverInstances_>::batch_over_relations(TupleOfTuplesOfUnivariates& univariate_accumulators,
                                            const CombinedRelationSeparator& alpha)
{
    auto result = std::get<0>(std::get<0>(univariate_accumulators))
                      .template extend_to<ProverInstances::BATCHED_EXTENDED_LENGTH>();
    size_t idx = 0;
    auto scale_and_sum = [&]<size_t outer_idx, size_t inner_idx>(auto& element) {
        auto extended = element.template extend_to<ProverInstances::BATCHED_EXTENDED_LENGTH>();
        extended *= alpha[idx];
        result += extended;
        idx++;
    };

    Utils::template apply_to_tuple_of_tuples<0, 1>(univariate_accumulators, scale_and_sum);
    Utils::zero_univariates(univariate_accumulators);

    return result;
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
Univariate<typename ProtoGalaxyProver_<ProverInstances_>::FF,
           ProverInstances_::BATCHED_EXTENDED_LENGTH,
           ProverInstances_::NUM>
ProtoGalaxyProver_<ProverInstances_>::compute_combiner_quotient(const FF compressed_perturbator,
                                                                ExtendedUnivariateWithRandomization combiner)
{
    std::array<FF, ProverInstances::BATCHED_EXTENDED_LENGTH - ProverInstances::NUM> combiner_quotient_evals = {};

    constexpr FF inverse_two = FF(2).invert();
    constexpr FF inverse_six = FF(6).invert();
    for (size_t point = ProverInstances::NUM; point < combiner.size(); point++) {
        auto idx = point - ProverInstances::NUM;
        FF lagrange_0;
        FF vanishing_polynomial;
        if constexpr (ProverInstances::NUM == 2) {
            lagrange_0 = FF(1) - FF(point);
            vanishing_polynomial = FF(point) * (FF(point) - 1);
        } else if constexpr (ProverInstances::NUM == 3) {
            lagrange_0 = (FF(1) - FF(point)) * (FF(2) - FF(point)) * inverse_two;
            vanishing_polynomial = FF(point) * (FF(point) - 1) * (FF(point) - 2);
        } else if constexpr (ProverInstances::NUM == 4) {
            lagrange_0 = (FF(1) - FF(point)) * (FF(2) - FF(point)) * (FF(3) - FF(point)) * inverse_six;
            vanishing_polynomial = FF(point) * (FF(point) - 1) * (FF(point) - 2) * (FF(point) - 3);
        }
        static_assert(ProverInstances::NUM < 5);

        combiner_quotient_evals[idx] =
            (combiner.value_at(point) - compressed_perturbator * lagrange_0) * vanishing_polynomial.invert();
    }

    Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM> combiner_quotient(
        combiner_quotient_evals);
    return combiner_quotient;
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_>
void ProtoGalaxyProver_<ProverInstances_>::combine_relation_parameters(ProverInstances& instances)
{
    size_t param_idx = 0;
    auto to_fold = instances.relation_parameters.get_to_fold();
    auto to_fold_optimised = instances.optimised_relation_parameters.get_to_fold();
    for (auto [folded_parameter, optimised_folded_parameter] : zip_view(to_fold, to_fold_optimised)) {
        Univariate<FF, ProverInstances::NUM> tmp(0);
        size_t instance_idx = 0;
        for (auto& instance : instances) {
            tmp.value_at(instance_idx) = instance->relation_parameters.get_to_fold()[param_idx];
            instance_idx++;
        }
        folded_parameter = tmp.template extend_to<ProverInstances::EXTENDED_LENGTH>();
        optimised_folded_parameter =
            tmp.template extend_to<ProverInstances::EXTENDED_LENGTH, ProverInstances::NUM - 1>();
        param_idx++;
    }
}

// See protogalaxy_prover.hpp for details
template <class ProverInstances_> void ProtoGalaxyProver_<ProverInstances_>::combine_alpha(ProverInstances& instances)
{
    size_t alpha_idx = 0;
    for (auto& alpha : instances.alphas) {
        Univariate<FF, ProverInstances::NUM> tmp;
        size_t instance_idx = 0;
        for (auto& instance : instances) {
            tmp.value_at(instance_idx) = instance->alphas[alpha_idx];
            instance_idx++;
        }
        alpha = tmp.template extend_to<ProverInstances::BATCHED_EXTENDED_LENGTH>();
        alpha_idx++;
    }
}

template <class ProverInstances>
void ProtoGalaxyProver_<ProverInstances>::finalise_and_send_instance(std::shared_ptr<Instance> instance,
                                                                     const std::string& domain_separator)
{
    OinkProver<Flavor> oink_prover(instance->proving_key, transcript, domain_separator + '_');

    auto [proving_key, relation_params, alphas] = oink_prover.prove();
    instance->proving_key = std::move(proving_key);
    instance->relation_parameters = std::move(relation_params);
    instance->alphas = std::move(alphas);
}

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::prepare_for_folding()
{
    auto idx = 0;
    auto instance = instances[0];
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
}

/**
 * @brief Given the challenge \gamma, compute Z(\gamma) and {L_0(\gamma),L_1(\gamma)}
 * TODO(https://github.com/AztecProtocol/barretenberg/issues/764): Generalize the vanishing polynomial formula
 * and the computation of Lagrange basis for k instances
 */
template <class ProverInstances>
std::pair<typename ProverInstances::FF, std::array<typename ProverInstances::FF, ProverInstances::NUM>>
ProtoGalaxyProver_<ProverInstances>::_compute_vanishing_polynomial_and_lagranges(const FF& challenge)
{
    FF vanishing_polynomial_at_challenge;
    std::array<FF, ProverInstances::NUM> lagranges;
    constexpr FF inverse_two = FF(2).invert();

    if constexpr (ProverInstances::NUM == 2) {
        vanishing_polynomial_at_challenge = challenge * (challenge - FF(1));
        lagranges = { FF(1) - challenge, challenge };
    } else if constexpr (ProverInstances::NUM == 3) {
        vanishing_polynomial_at_challenge = challenge * (challenge - FF(1)) * (challenge - FF(2));
        lagranges = { (FF(1) - challenge) * (FF(2) - challenge) * inverse_two,
                      challenge * (FF(2) - challenge),
                      challenge * (challenge - FF(1)) / FF(2) };
    } else if constexpr (ProverInstances::NUM == 4) {
        constexpr FF inverse_six = FF(6).invert();
        vanishing_polynomial_at_challenge = challenge * (challenge - FF(1)) * (challenge - FF(2)) * (challenge - FF(3));
        lagranges = { (FF(1) - challenge) * (FF(2) - challenge) * (FF(3) - challenge) * inverse_six,
                      challenge * (FF(2) - challenge) * (FF(3) - challenge) * inverse_two,
                      challenge * (challenge - FF(1)) * (FF(3) - challenge) * inverse_two,
                      challenge * (challenge - FF(1)) * (challenge - FF(2)) * inverse_six };
    }
    static_assert(ProverInstances::NUM < 5);

    return { vanishing_polynomial_at_challenge, lagranges };
}

template <class ProverInstances>
std::shared_ptr<typename ProverInstances::Instance> ProtoGalaxyProver_<ProverInstances>::compute_next_accumulator(
    ProverInstances& instances,
    Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM>& combiner_quotient,
    FF& challenge,
    const FF& compressed_perturbator)
{
    auto combiner_quotient_at_challenge = combiner_quotient.evaluate(challenge);
    auto [vanishing_polynomial_at_challenge, lagranges] = _compute_vanishing_polynomial_and_lagranges(challenge);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/881): bad pattern
    auto next_accumulator = std::move(instances[0]);
    next_accumulator->is_accumulator = true;

    // Compute the next target sum and send the next folding parameters to the verifier
    FF next_target_sum =
        compressed_perturbator * lagranges[0] + vanishing_polynomial_at_challenge * combiner_quotient_at_challenge;

    next_accumulator->target_sum = next_target_sum;
    next_accumulator->gate_challenges = instances.next_gate_challenges;

    // Initialize accumulator proving key polynomials
    auto accumulator_polys = next_accumulator->proving_key.polynomials.get_all();
    for (size_t poly_idx = 0; poly_idx < Flavor::NUM_FOLDED_ENTITIES; poly_idx++) {
        accumulator_polys[poly_idx] *= lagranges[0];
    }

    // Fold the proving key polynomials
    for (size_t inst_idx = 1; inst_idx < ProverInstances::NUM; inst_idx++) {
        auto input_polys = instances[inst_idx]->proving_key.polynomials.get_all();
        for (size_t poly_idx = 0; poly_idx < Flavor::NUM_FOLDED_ENTITIES; poly_idx++) {
            accumulator_polys[poly_idx].add_scaled(input_polys[poly_idx], lagranges[inst_idx]);
        }
    }

    // Evaluate the combined batching  α_i univariate at challenge to obtain next α_i and send it to the
    // verifier, where i ∈ {0,...,NUM_SUBRELATIONS - 1}
    auto& folded_alphas = next_accumulator->alphas;
    for (size_t idx = 0; idx < NUM_SUBRELATIONS - 1; idx++) {
        folded_alphas[idx] = instances.alphas[idx].evaluate(challenge);
    }

    // Evaluate each relation parameter univariate at challenge to obtain the folded relation parameters and send to
    // the verifier
    auto& combined_relation_parameters = instances.relation_parameters;
    auto folded_relation_parameters = bb::RelationParameters<FF>{
        combined_relation_parameters.eta.evaluate(challenge),
        combined_relation_parameters.eta_two.evaluate(challenge),
        combined_relation_parameters.eta_three.evaluate(challenge),
        combined_relation_parameters.beta.evaluate(challenge),
        combined_relation_parameters.gamma.evaluate(challenge),
        combined_relation_parameters.public_input_delta.evaluate(challenge),
        combined_relation_parameters.lookup_grand_product_delta.evaluate(challenge),
    };
    next_accumulator->relation_parameters = folded_relation_parameters;
    next_accumulator->proving_key = std::move(instances[0]->proving_key);
    return next_accumulator;
}

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::preparation_round()
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::preparation_round");
    prepare_for_folding();
};

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::perturbator_round()
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::perturbator_round");
    state.accumulator = get_accumulator();
    FF delta = transcript->template get_challenge<FF>("delta");
    state.deltas = compute_round_challenge_pows(state.accumulator->proving_key.log_circuit_size, delta);
    state.perturbator =
        LegacyPolynomial<FF>(state.accumulator->proving_key.log_circuit_size + 1); // initialize to all zeros
    // compute perturbator only if this is not the first round and has an accumulator
    if (state.accumulator->is_accumulator) {
        state.perturbator = compute_perturbator(state.accumulator, state.deltas);
        // Prover doesn't send the constant coefficient of F because this is supposed to be equal to the target sum of
        // the accumulator which the folding verifier has from the previous iteration.
        for (size_t idx = 1; idx <= state.accumulator->proving_key.log_circuit_size; idx++) {
            transcript->send_to_verifier("perturbator_" + std::to_string(idx), state.perturbator[idx]);
        }
    }
};

template <class ProverInstances_>
std::vector<typename bb::ProtoGalaxyProver_<ProverInstances_>::FF> bb::ProtoGalaxyProver_<
    ProverInstances_>::update_gate_challenges(const FF perturbator_challenge,
                                              const std::vector<FF>& gate_challenges,
                                              const std::vector<FF>& round_challenges)
{
    auto log_instance_size = gate_challenges.size();
    std::vector<FF> next_gate_challenges(log_instance_size);

    for (size_t idx = 0; idx < log_instance_size; idx++) {
        next_gate_challenges[idx] = gate_challenges[idx] + perturbator_challenge * round_challenges[idx];
    }
    return next_gate_challenges;
}

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::combiner_quotient_round()
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::combiner_quotient_round");
    auto perturbator_challenge = transcript->template get_challenge<FF>("perturbator_challenge");
    instances.next_gate_challenges =
        update_gate_challenges(perturbator_challenge, state.accumulator->gate_challenges, state.deltas);
    combine_relation_parameters(instances);
    combine_alpha(instances);
    auto pow_polynomial = PowPolynomial<FF>(instances.next_gate_challenges);
    auto combiner = compute_combiner(instances, pow_polynomial);

    state.compressed_perturbator = state.perturbator.evaluate(perturbator_challenge);
    state.combiner_quotient = compute_combiner_quotient(state.compressed_perturbator, combiner);

    for (size_t idx = ProverInstances::NUM; idx < ProverInstances::BATCHED_EXTENDED_LENGTH; idx++) {
        transcript->send_to_verifier("combiner_quotient_" + std::to_string(idx), state.combiner_quotient.value_at(idx));
    }
};

template <class ProverInstances> void ProtoGalaxyProver_<ProverInstances>::accumulator_update_round()
{
    BB_OP_COUNT_TIME_NAME("ProtoGalaxyProver_::accumulator_update_round");
    FF combiner_challenge = transcript->template get_challenge<FF>("combiner_quotient_challenge");
    std::shared_ptr<Instance> next_accumulator =
        compute_next_accumulator(instances, state.combiner_quotient, combiner_challenge, state.compressed_perturbator);
    state.result.proof = transcript->proof_data;
    state.result.accumulator = next_accumulator;
};

template <class ProverInstances>
FoldingResult<typename ProverInstances::Flavor> ProtoGalaxyProver_<ProverInstances>::prove()
{
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
    preparation_round();
    perturbator_round();
    combiner_quotient_round();
    accumulator_update_round();

    return state.result;
}
} // namespace bb