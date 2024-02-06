#pragma once
#include "barretenberg/common/thread.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/utils.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"

namespace bb {
template <class ProverInstances_> class ProtoGalaxyProver_ {
  public:
    using ProverInstances = ProverInstances_;
    using Flavor = typename ProverInstances::Flavor;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;
    using Instance = typename ProverInstances::Instance;
    using Utils = bb::RelationUtils<Flavor>;
    using RowEvaluations = typename Flavor::AllValues;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Relations = typename Flavor::Relations;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using CombinedRelationSeparator = typename ProverInstances::RelationSeparator;
    using VerificationKey = typename Flavor::VerificationKey;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using Commitment = typename Flavor::Commitment;

    using BaseUnivariate = Univariate<FF, ProverInstances::NUM>;
    // The length of ExtendedUnivariate is the largest length (==max_relation_degree + 1) of a univariate polynomial
    // obtained by composing a relation with folded instance + relation parameters .
    using ExtendedUnivariate = Univariate<FF, (Flavor::MAX_TOTAL_RELATION_LENGTH - 1) * (ProverInstances::NUM - 1) + 1>;
    // Represents the total length of the combiner univariate, obtained by combining the already folded relations with
    // the folded relation batching challenge.
    using ExtendedUnivariateWithRandomization =
        Univariate<FF,
                   (Flavor::MAX_TOTAL_RELATION_LENGTH - 1 + ProverInstances::NUM - 1) * (ProverInstances::NUM - 1) + 1>;
    using ExtendedUnivariates = typename Flavor::template ProverUnivariates<ExtendedUnivariate::LENGTH>;

    using TupleOfTuplesOfUnivariates =
        typename Flavor::template ProtogalaxyTupleOfTuplesOfUnivariates<ProverInstances::NUM>;
    using RelationEvaluations = typename Flavor::TupleOfArraysOfValues;

    static constexpr size_t NUM_SUBRELATIONS = ProverInstances::NUM_SUBRELATIONS;

    ProverInstances instances;
    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();

    std::shared_ptr<CommitmentKey> commitment_key;

    ProtoGalaxyProver_() = default;
    ProtoGalaxyProver_(const std::vector<std::shared_ptr<Instance>>& insts,
                       const std::shared_ptr<CommitmentKey>& commitment_key)
        : instances(ProverInstances(insts))
        , commitment_key(std::move(commitment_key)){};
    ~ProtoGalaxyProver_() = default;

    /**
     * @brief Prior to folding, we need to finalize the given instances and add all their public data ϕ to the
     * transcript, labelled by their corresponding instance index for domain separation.
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/795):The rounds prior to actual proving/folding are
     * common between decider and folding verifier and could be somehow shared so we do not duplicate code so much.
     */
    void prepare_for_folding();

    /**
     * @brief Send the public data of an accumulator, i.e. a relaxed instance, to the verifier (ϕ in the paper).
     *
     *  @param domain_separator separates the same type of data coming from difference instances by instance
     * index
     */
    void send_accumulator(std::shared_ptr<Instance>, const std::string& domain_separator);

    /**
     * @brief For each instance produced by a circuit, prior to folding, we need to complete the computation of its
     * prover polynomials, commit to witnesses and generate the relation parameters as well as send the public data ϕ of
     * an instance to the verifier.
     *
     * @param domain_separator  separates the same type of data coming from difference instances by instance
     * index
     */
    void finalise_and_send_instance(std::shared_ptr<Instance>, const std::string& domain_separator);

    /**
     * @brief Run the folding prover protocol to produce a new accumulator and a folding proof to be verified by the
     * folding verifier.
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/753): fold goblin polynomials
     */
    BB_PROFILE FoldingResult<Flavor> fold_instances();

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

    static std::vector<FF> update_gate_challenges(const FF perturbator_challenge,
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

    // Returns the accumulator, which is the first element in ProverInstances. The accumulator is assumed to have the
    // FoldingParameters set and be the result of a previous round of folding.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/740): handle the case when the accumulator is empty
    // (i.e. we are in the first round of folding)/
    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Compute the values of the full Honk relation at each row in the execution trace, representing f_i(ω) in
     * the ProtoGalaxy paper, given the evaluations of all the prover polynomials and \vec{α} (the batching challenges
     * that help establishing each subrelation is independently valid in Honk - from the Plonk paper, DO NOT confuse
     * with α in ProtoGalaxy).
     *
     * @details When folding GoblinUltra instances, one of the relations is linearly dependent. We define such relations
     * as acting on the entire execution trace and hence requiring to be accumulated separately as we iterate over each
     * row. At the end of the function, the linearly dependent contribution is accumulated at index 0 representing the
     * sum f_0(ω) + α_j*g(ω) where f_0 represents the full honk evaluation at row 0, g(ω) is the linearly dependent
     * subrelation and α_j is its corresponding batching challenge.
     */
    static std::vector<FF> compute_full_honk_evaluations(const ProverPolynomials& instance_polynomials,
                                                         const RelationSeparator& alpha,
                                                         const RelationParameters<FF>& relation_parameters)
    {
        auto instance_size = instance_polynomials.get_polynomial_size();
        FF linearly_dependent_contribution = FF(0);
        std::vector<FF> full_honk_evaluations(instance_size);
        for (size_t row = 0; row < instance_size; row++) {
            auto row_evaluations = instance_polynomials.get_row(row);
            RelationEvaluations relation_evaluations;
            Utils::zero_elements(relation_evaluations);

            // Note that the evaluations are accumulated with the gate separation challenge being 1 at this stage, as
            // this specific randomness is added later through the power polynomial univariate specific to ProtoGalaxy
            Utils::template accumulate_relation_evaluations<>(
                row_evaluations, relation_evaluations, relation_parameters, FF(1));

            auto output = FF(0);
            auto running_challenge = FF(1);

            // Sum relation evaluations, batched by their corresponding relation separator challenge, to get the value
            // of the full honk relation at a specific row
            Utils::scale_and_batch_elements(
                relation_evaluations, alpha, running_challenge, output, linearly_dependent_contribution);

            full_honk_evaluations[row] = output;
        }
        full_honk_evaluations[0] += linearly_dependent_contribution;
        return full_honk_evaluations;
    }

    /**
     * @brief  Recursively compute the parent nodes of each level in the tree, starting from the leaves. Note that at
     * each level, the resulting parent nodes will be polynomials of degree (level+1) because we multiply by an
     * additional factor of X.
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
                                              const std::vector<FF>& deltas)
    {
        auto full_honk_evaluations = compute_full_honk_evaluations(
            accumulator->prover_polynomials, accumulator->alphas, accumulator->relation_parameters);
        const auto betas = accumulator->gate_challenges;
        assert(betas.size() == deltas.size());
        auto coeffs = construct_perturbator_coefficients(betas, deltas, full_honk_evaluations);
        return Polynomial<FF>(coeffs);
    }

    TupleOfTuplesOfUnivariates univariate_accumulators;

    /**
     * @brief Prepare a univariate polynomial for relation execution in one step of the main loop in folded instance
     * construction.
     * @details For a fixed prover polynomial index, extract that polynomial from each instance in Instances. From each
     * polynomial, extract the value at row_idx. Use these values to create a univariate polynomial, and then extend
     * (i.e., compute additional evaluations at adjacent domain values) as needed.
     * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/751) Optimize memory
     */
    void extend_univariates(ExtendedUnivariates& extended_univariates,
                            const ProverInstances& instances,
                            const size_t row_idx)
    {
        auto base_univariates = instances.row_to_univariates(row_idx);
        for (auto [extended_univariate, base_univariate] : zip_view(extended_univariates.get_all(), base_univariates)) {
            extended_univariate = base_univariate.template extend_to<ExtendedUnivariate::LENGTH>();
        }
    }

    template <typename Parameters, size_t relation_idx = 0>
    void accumulate_relation_univariates(TupleOfTuplesOfUnivariates& univariate_accumulators,
                                         const ExtendedUnivariates& extended_univariates,
                                         const Parameters& relation_parameters,
                                         const FF& scaling_factor)
    {
        using Relation = std::tuple_element_t<relation_idx, Relations>;
        Relation::accumulate(
            std::get<relation_idx>(univariate_accumulators), extended_univariates, relation_parameters, scaling_factor);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < Flavor::NUM_RELATIONS) {
            accumulate_relation_univariates<Parameters, relation_idx + 1>(
                univariate_accumulators, extended_univariates, relation_parameters, scaling_factor);
        }
    }

    /**
     * @brief Compute the combiner polynomial $G$ in the Protogalaxy paper.
     *
     */
    ExtendedUnivariateWithRandomization compute_combiner(const ProverInstances& instances, PowPolynomial<FF>& pow_betas)
    {
        size_t common_instance_size = instances[0]->instance_size;
        pow_betas.compute_values();
        // Determine number of threads for multithreading.
        // Note: Multithreading is "on" for every round but we reduce the number of threads from the max available based
        // on a specified minimum number of iterations per thread. This eventually leads to the use of a single thread.
        // For now we use a power of 2 number of threads simply to ensure the round size is evenly divided.
        size_t max_num_threads = get_num_cpus_pow2(); // number of available threads (power of 2)
        size_t min_iterations_per_thread = 1 << 6; // min number of iterations for which we'll spin up a unique thread
        size_t desired_num_threads = common_instance_size / min_iterations_per_thread;
        size_t num_threads = std::min(desired_num_threads, max_num_threads); // fewer than max if justified
        num_threads = num_threads > 0 ? num_threads : 1;                     // ensure num threads is >= 1
        size_t iterations_per_thread = common_instance_size / num_threads;   // actual iterations per thread
        // Construct univariate accumulator containers; one per thread
        std::vector<TupleOfTuplesOfUnivariates> thread_univariate_accumulators(num_threads);
        for (auto& accum : thread_univariate_accumulators) {
            // just normal relation lengths
            Utils::zero_univariates(accum);
        }

        // Construct extended univariates containers; one per thread
        std::vector<ExtendedUnivariates> extended_univariates;
        extended_univariates.resize(num_threads);

        // Accumulate the contribution from each sub-relation
        parallel_for(num_threads, [&](size_t thread_idx) {
            size_t start = thread_idx * iterations_per_thread;
            size_t end = (thread_idx + 1) * iterations_per_thread;

            for (size_t idx = start; idx < end; idx++) {
                // No need to initialise extended_univariates to 0, it's assigned to
                extend_univariates(extended_univariates[thread_idx], instances, idx);

                FF pow_challenge = pow_betas[idx];

                // Accumulate the i-th row's univariate contribution. Note that the relation parameters passed to this
                // function have already been folded. Moreover, linear-dependent relations that act over the entire
                // execution trace rather than on rows, will not be multiplied by the pow challenge.
                accumulate_relation_univariates(
                    thread_univariate_accumulators[thread_idx],
                    extended_univariates[thread_idx],
                    instances.relation_parameters, // these parameters have already been folded
                    pow_challenge);
            }
        });

        // Accumulate the per-thread univariate accumulators into a single set of accumulators
        for (auto& accumulators : thread_univariate_accumulators) {
            Utils::add_nested_tuples(univariate_accumulators, accumulators);
        }
        // Batch the univariate contributions from each sub-relation to obtain the round univariate
        return batch_over_relations(univariate_accumulators, instances.alphas);
    }

    static ExtendedUnivariateWithRandomization batch_over_relations(TupleOfTuplesOfUnivariates& univariate_accumulators,
                                                                    const CombinedRelationSeparator& alpha)
    {

        // First relation does not get multiplied by a batching challenge
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

    /**
     * @brief Compute the combiner quotient defined as $K$ polynomial in the paper.
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/764): generalize the computation of vanishing
     * polynomials and Lagrange basis and use batch_invert.
     *
     */
    static Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM> compute_combiner_quotient(
        const FF compressed_perturbator, ExtendedUnivariateWithRandomization combiner)
    {
        std::array<FF, ProverInstances::BATCHED_EXTENDED_LENGTH - ProverInstances::NUM> combiner_quotient_evals = {};

        // Compute the combiner quotient polynomial as evaluations on points that are not in the vanishing set.
        //
        for (size_t point = ProverInstances::NUM; point < combiner.size(); point++) {
            auto idx = point - ProverInstances::NUM;
            auto lagrange_0 = FF(1) - FF(point);
            auto vanishing_polynomial = FF(point) * (FF(point) - 1);

            combiner_quotient_evals[idx] =
                (combiner.value_at(point) - compressed_perturbator * lagrange_0) * vanishing_polynomial.invert();
        }

        Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM> combiner_quotient(
            combiner_quotient_evals);
        return combiner_quotient;
    }

    /**
     * @brief Combine each relation parameter, in part, from all the instances into univariates, used in the computation
     * of combiner.
     * @details For a given relation parameter type, extract that parameter from each instance, place the values in a
     * univariate (i.e., sum them against an appropriate univariate Lagrange basis) and then extended as needed during
     * the constuction of the combiner.
     */
    static void combine_relation_parameters(ProverInstances& instances)
    {
        size_t param_idx = 0;
        auto to_fold = instances.relation_parameters.get_to_fold();
        for (auto& folded_parameter : to_fold) {
            Univariate<FF, ProverInstances::NUM> tmp(0);
            size_t instance_idx = 0;
            for (auto& instance : instances) {
                tmp.value_at(instance_idx) = instance->relation_parameters.get_to_fold()[param_idx];
                instance_idx++;
            }
            folded_parameter = tmp.template extend_to<ProverInstances::EXTENDED_LENGTH>();
            param_idx++;
        }
    }

    /**
     * @brief Combine the relation batching parameters (alphas) from each instance into a univariate, used in the
     * computation of combiner.
     *
     */
    static void combine_alpha(ProverInstances& instances)
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

    /**
     * @brief Compute the next accumulator (ϕ*, ω*, \vec{\beta*}, e*), send the public data ϕ*  and the folding
     * parameters
     * (\vec{\beta*}, e*) to the verifier and return the complete accumulator
     *
     * @details At this stage, we assume that the instances have the same size and the same number of public parameter.s
     * @param instances
     * @param combiner_quotient polynomial K in the paper
     * @param challenge
     * @param compressed_perturbator
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/796): optimise the construction of the new accumulator
     */
    std::shared_ptr<Instance> compute_next_accumulator(
        ProverInstances& instances,
        Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM>& combiner_quotient,
        FF& challenge,
        const FF& compressed_perturbator);
};

} // namespace bb
