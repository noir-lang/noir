#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/instance/instances.hpp"
#include "barretenberg/honk/proof_system/folding_result.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/relations/utils.hpp"
namespace proof_system::honk {
template <class ProverInstances> class ProtoGalaxyProver_ {
  public:
    using Flavor = typename ProverInstances::Flavor;
    using Instance = typename ProverInstances::Instance;
    using Utils = barretenberg::RelationUtils<Flavor>;
    using RowEvaluations = typename Flavor::AllValues;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using FF = typename Flavor::FF;
    using RelationEvaluations = typename Flavor::TupleOfArraysOfValues;

    ProverInstances instances;
    ProverTranscript<FF> transcript;

    ProtoGalaxyProver_(ProverInstances insts)
        : instances(insts){};
    ~ProtoGalaxyProver_() = default;

    /**
     * @brief Prior to folding we need to add all the public inputs to the transcript, labelled by their corresponding
     * instance index, compute all the instance's polynomials and record the relation parameters involved in computing
     * these polynomials in the transcript.
     *
     */
    void prepare_for_folding();

    /**
     * @brief For a new round challenge δ at each iteration of the ProtoGalaxy protocol, compute the vector
     * [δ, δ^2,..., δ^t] where t = logn and n is the size of the instance.
     */
    static std::vector<FF> compute_round_challenge_pows(const size_t log_instance_size, const FF& round_challenge)
    {
        std::vector<FF> pows(log_instance_size);
        pows[0] = round_challenge;
        for (size_t i = 1; i < log_instance_size; i++) {
            pows[i] = pows[i - 1].sqr();
        }
        return pows;
    }

    // Returns the accumulator, which is the first element in ProverInstances. The accumulator is assumed to have the
    // FoldingParameters set and be the result of a previous round of folding.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/740): handle the case when the accumulator is empty
    // (i.e. we are in the first round of folding)/
    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Compute the values of the full Honk relation at each row in the execution trace, f_i(ω) in the
     * ProtoGalaxy paper, given the evaluations of all the prover polynomials and α (the parameter that helps establish
     * each subrelation is independently valid in Honk - from the Plonk paper, DO NOT confuse with α in ProtoGalaxy),
     */
    static std::vector<FF> compute_full_honk_evaluations(const ProverPolynomials& instance_polynomials,
                                                         const FF& alpha,
                                                         const RelationParameters<FF>& relation_parameters)
    {
        auto instance_size = std::get<0>(instance_polynomials._data).size();

        std::vector<FF> full_honk_evaluations(instance_size);
        for (size_t row = 0; row < instance_size; row++) {
            auto row_evaluations = instance_polynomials.get_row(row);
            RelationEvaluations relation_evaluations;
            Utils::zero_elements(relation_evaluations);

            // Note that the evaluations are accumulated with the gate separation challenge being 1 at this stage, as
            // this specific randomness is added later through the power polynomial univariate specific to ProtoGalaxy
            Utils::template accumulate_relation_evaluations<>(
                row_evaluations, relation_evaluations, relation_parameters, FF(1));

            auto running_challenge = FF(1);
            auto output = FF(0);
            Utils::scale_and_batch_elements(relation_evaluations, alpha, running_challenge, output);
            full_honk_evaluations[row] = output;
        }
        return full_honk_evaluations;
    }

    /**
     * @brief  Recursively compute the parent nodes of each level in there, starting from the leaves. Note that at each
     * level, the resulting parent nodes will be polynomials of degree (level + 1) because we multiply by an additional
     * factor of X.
     */
    static std::vector<FF> construct_coefficients_tree(const std::vector<FF>& betas,
                                                       const std::vector<FF>& deltas,
                                                       const std::vector<std::vector<FF>>& prev_level_coeffs,
                                                       size_t level = 1)
    {
        // if we are at level t in the tree, where t = logn and n is the instance size, we have reached the root which
        // contains the coefficients of the perturbator polynomial
        if (level == betas.size()) {
            return prev_level_coeffs[0];
        }

        auto degree = level + 1;
        auto prev_level_width = prev_level_coeffs.size();
        // we need degree + 1 terms to represent the intermediate polynomials
        std::vector<std::vector<FF>> level_coeffs(prev_level_width >> 1, std::vector<FF>(degree + 1, 0));
        for (size_t node = 0; node < prev_level_width; node += 2) {
            auto parent = node >> 1;
            std::copy(prev_level_coeffs[node].begin(), prev_level_coeffs[node].end(), level_coeffs[parent].begin());
            for (size_t d = 0; d < degree; d++) {
                level_coeffs[parent][d] += prev_level_coeffs[node + 1][d] * betas[level];
                level_coeffs[parent][d + 1] += prev_level_coeffs[node + 1][d] * deltas[level];
            }
        }
        return construct_coefficients_tree(betas, deltas, level_coeffs, level + 1);
    }

    /**
     * @brief We construct the coefficients of the perturbator polynomial in O(n) time following the technique in
     * Claim 4.4. Consider a binary tree whose leaves are the evaluations of the full Honk relation at each row in the
     * execution trace. The subsequent levels in the tree are constructed using the following technique: At level i in
     * the tree, label the branch connecting the left node n_l to its parent by 1 and for the right node n_r by β_i +
     * δ_i X. The value of the parent node n will be constructed as n = n_l + n_r * (β_i + δ_i X). Recurse over each
     * layer until the root is reached which will correspond to the perturbator polynomial F(X).
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/745): make computation of perturbator more memory
     * efficient, operate in-place and use std::resize; add multithreading
     */
    static std::vector<FF> construct_perturbator_coefficients(const std::vector<FF>& betas,
                                                              const std::vector<FF>& deltas,
                                                              const std::vector<FF>& full_honk_evaluations)
    {
        auto width = full_honk_evaluations.size();
        std::vector<std::vector<FF>> first_level_coeffs(width >> 1, std::vector<FF>(2, 0));
        for (size_t node = 0; node < width; node += 2) {
            auto parent = node >> 1;
            first_level_coeffs[parent][0] = full_honk_evaluations[node] + full_honk_evaluations[node + 1] * betas[0];
            first_level_coeffs[parent][1] = full_honk_evaluations[node + 1] * deltas[0];
        }
        return construct_coefficients_tree(betas, deltas, first_level_coeffs);
    }

    /**
     * @brief Construct the power perturbator polynomial F(X) in coefficient form from the accumulator, representing the
     * relaxed instance.
     *
     *
     */
    static Polynomial<FF> compute_perturbator(const std::shared_ptr<Instance> accumulator,
                                              const std::vector<FF>& deltas,
                                              const FF& alpha)
    {
        auto full_honk_evaluations =
            compute_full_honk_evaluations(accumulator->prover_polynomials, alpha, accumulator->relation_parameters);
        const auto betas = accumulator->folding_parameters.gate_separation_challenges;
        assert(betas.size() == deltas.size());
        auto coeffs = construct_perturbator_coefficients(betas, deltas, full_honk_evaluations);
        return Polynomial<FF>(coeffs);
    }

    ProverFoldingResult<Flavor> fold_instances();
};

extern template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::Ultra, 2>>;
extern template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk