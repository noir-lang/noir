#pragma once
#include "barretenberg/common/log.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

namespace proof_system::honk::sumcheck {

/*
 Notation: The polynomial P(X0, X1) that is the low-degree extension of its values vij = P(i,j)
 for i,j ∈ H = {0,1} is conveniently recorded as follows:
     (0,1)-----(1,1)   v01 ------ v11
       |          |     |          |  P(X0,X1) =   (v00 * (1-X0) + v10 * X0) * (1-X1)
   X1  |   H^2    |     | P(X0,X1) |             + (v01 * (1-X0) + v11 * X0) *   X1
       |          |     |          |
     (0,0) ---- (1,0)  v00 -------v10
            X0
*/

/*
 Example: There are two low-degree extensions Y1, Y2 over the square H^2 in the Cartesian plane.

    3 -------- 7   4 -------- 8
    |          |   |          | Let F(X0, X1) = G(Y1, Y2) =     G0(Y1(X0, X1), Y2(X0, X1))
    |    Y1    |   |    Y2    |                             + α G1(Y1(X0, X1), Y2(X0, X1)),
    |          |   |          |  where the relations are G0(Y1, Y2) = Y1 * Y2
    1 -------- 5   2 -------- 6                      and G1(Y1, Y2) = Y1 + Y2.

 G1, G2 together comprise the Relations.

 In the first round, the computations will relate elements along horizontal lines. As a mnemonic, we
 use the term "edge" for the linear, univariate polynomials corresponding to the four lines
  1 - 5
  2 - 6
  3 - 7
  4 - 8

 The polynomials Y1, Y2 are stored in an array in Multivariates. In the first round, these are arrays
 of spans living outside of the Multivariates object, and in sebsequent rounds these are arrays of field
 elements that are stored in the Multivariates. The rationale for adopting this model is to
 avoid copying the full-length polynomials; this way, the largest polynomial array stores in a
 Multivariates class is multivariates_n/2.

 Note: This class uses recursive function calls with template parameters. This is a common trick that is used to force
 the compiler to unroll loops. The idea is that a function that is only called once will always be inlined, and since
 template functions always create different functions, this is guaranteed.

 @todo TODO(#390): Template only on Flavor? Is it useful to have these decoupled?
 */

template <typename Flavor> class SumcheckProverRound {

    using Relations = typename Flavor::Relations;
    using RelationUnivariates = typename Flavor::RelationUnivariates;

  public:
    using FF = typename Flavor::FF;
    template <size_t univariate_length>
    using ExtendedEdges = typename Flavor::template ExtendedEdges<univariate_length>;

    size_t round_size; // a power of 2

    Relations relations;
    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t MAX_RELATION_LENGTH = Flavor::MAX_RELATION_LENGTH;
    static constexpr size_t MAX_RANDOM_RELATION_LENGTH = Flavor::MAX_RANDOM_RELATION_LENGTH;

    RelationUnivariates univariate_accumulators;

    // TODO(#224)(Cody): this should go away
    barretenberg::BarycentricData<FF, 2, MAX_RELATION_LENGTH> barycentric_2_to_max;

    // Prover constructor
    SumcheckProverRound(size_t initial_round_size)
        : round_size(initial_round_size)
    {
        // Initialize univariate accumulators to 0
        zero_univariates(univariate_accumulators);
    }

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_RELATIONS-1}) and a challenge α,
     * return t_0 + αt_1 + ... + α^{NUM_RELATIONS-1}t_{NUM_RELATIONS-1}).
     *
     * @tparam T : In practice, this is a Univariate<FF, MAX_NUM_RELATIONS>.
     */
    barretenberg::Univariate<FF, MAX_RANDOM_RELATION_LENGTH> batch_over_relations(
        FF challenge, const barretenberg::PowUnivariate<FF>& pow_univariate)
    {
        FF running_challenge = 1;
        scale_univariates(univariate_accumulators, challenge, running_challenge);

        auto result = barretenberg::Univariate<FF, MAX_RANDOM_RELATION_LENGTH>(0);
        extend_and_batch_univariates(univariate_accumulators, pow_univariate, result);

        // Reset all univariate accumulators to 0 before beginning accumulation in the next round
        zero_univariates(univariate_accumulators);
        return result;
    }

    /**
     * @brief Extend each edge in the edge group at to max-relation-length-many values.
     *
     * @details Should only be called externally with relation_idx equal to 0.
     * In practice, multivariates is one of ProverPolynomials or FoldedPolynomials.
     *
     */
    void extend_edges(auto& extended_edges, auto& multivariates, size_t edge_idx)
    {
        size_t univariate_idx = 0; // TODO(#391) zip
        for (auto& poly : multivariates) {
            auto edge = barretenberg::Univariate<FF, 2>({ poly[edge_idx], poly[edge_idx + 1] });
            extended_edges[univariate_idx] = barycentric_2_to_max.extend(edge);
            ++univariate_idx;
        }
    }

    /**
     * @brief Return the evaluations of the univariate restriction (S_l(X_l) in the thesis) at num_multivariates-many
     * values. Most likely this will end up being S_l(0), ... , S_l(t-1) where t is around 12. At the end, reset all
     * univariate accumulators to be zero.
     */
    barretenberg::Univariate<FF, MAX_RANDOM_RELATION_LENGTH> compute_univariate(
        auto& polynomials,
        const proof_system::RelationParameters<FF>& relation_parameters,
        const barretenberg::PowUnivariate<FF>& pow_univariate,
        const FF alpha)
    {
        // Precompute the vector of required powers of zeta
        // TODO(luke): Parallelize this
        std::vector<FF> pow_challenges(round_size >> 1);
        pow_challenges[0] = pow_univariate.partial_evaluation_constant;
        for (size_t i = 1; i < (round_size >> 1); ++i) {
            pow_challenges[i] = pow_challenges[i - 1] * pow_univariate.zeta_pow_sqr;
        }

        // Determine number of threads for multithreading.
        // Note: Multithreading is "on" for every round but we reduce the number of threads from the max available based
        // on a specified minimum number of iterations per thread. This eventually leads to the use of a single thread.
        // For now we use a power of 2 number of threads simply to ensure the round size is evenly divided.
        size_t max_num_threads = get_num_cpus_pow2(); // number of available threads (power of 2)
        size_t min_iterations_per_thread = 1 << 6; // min number of iterations for which we'll spin up a unique thread
        size_t desired_num_threads = round_size / min_iterations_per_thread;
        size_t num_threads = std::min(desired_num_threads, max_num_threads); // fewer than max if justified
        num_threads = num_threads > 0 ? num_threads : 1;                     // ensure num threads is >= 1
        size_t iterations_per_thread = round_size / num_threads;             // actual iterations per thread

        // Constuct univariate accumulator containers; one per thread
        std::vector<RelationUnivariates> thread_univariate_accumulators(num_threads);
        for (auto& accum : thread_univariate_accumulators) {
            zero_univariates(accum);
        }

        // Constuct extended edge containers; one per thread
        std::vector<ExtendedEdges<MAX_RELATION_LENGTH>> extended_edges;
        extended_edges.resize(num_threads);

        // Accumulate the contribution from each sub-relation accross each edge of the hyper-cube
        parallel_for(num_threads, [&](size_t thread_idx) {
            size_t start = thread_idx * iterations_per_thread;
            size_t end = (thread_idx + 1) * iterations_per_thread;

            // For each edge_idx = 2i, we need to multiply the whole contribution by zeta^{2^{2i}}
            // This means that each univariate for each relation needs an extra multiplication.
            for (size_t edge_idx = start; edge_idx < end; edge_idx += 2) {
                extend_edges(extended_edges[thread_idx], polynomials, edge_idx);

                // Update the pow polynomial's contribution c_l ⋅ ζ_{l+1}ⁱ for the next edge.
                FF pow_challenge = pow_challenges[edge_idx >> 1];

                // Compute the i-th edge's univariate contribution,
                // scale it by the pow polynomial's constant and zeta power "c_l ⋅ ζ_{l+1}ⁱ"
                // and add it to the accumulators for Sˡ(Xₗ)
                accumulate_relation_univariates<>(thread_univariate_accumulators[thread_idx],
                                                  extended_edges[thread_idx],
                                                  relation_parameters,
                                                  pow_challenge);
            }
        });

        // Accumulate the per-thread univariate accumulators into a single set of accumulators
        for (auto& accumulators : thread_univariate_accumulators) {
            add_nested_tuples(univariate_accumulators, accumulators);
        }
        // Batch the univariate contributions from each sub-relation to obtain the round univariate
        return batch_over_relations(alpha, pow_univariate);
    }

  private:
    /**
     * @brief For a given edge, calculate the contribution of each relation to the prover round univariate (S_l in the
     * thesis).
     *
     * @details In Round l, the univariate S_l computed by the prover is computed as follows:
     *   - Outer loop: iterate through the points on the boolean hypercube of dimension = log(round_size), skipping
     *                 every other point. On each iteration, create a Univariate<FF, 2> (an 'edge') for each
     *                 multivariate.
     *   - Inner loop: iterate through the relations, feeding each relation the present collection of edges. Each
     *                 relation adds a contribution
     *
     * Result: for each relation, a univariate of some degree is computed by accumulating the contributions of each
     * group of edges. These are stored in `univariate_accumulators`. Adding these univariates together, with
     * appropriate scaling factors, produces S_l.
     */
    template <size_t relation_idx = 0>
    void accumulate_relation_univariates(RelationUnivariates& univariate_accumulators,
                                         const auto& extended_edges,
                                         const proof_system::RelationParameters<FF>& relation_parameters,
                                         const FF& scaling_factor)
    {
        std::get<relation_idx>(relations).add_edge_contribution(
            std::get<relation_idx>(univariate_accumulators), extended_edges, relation_parameters, scaling_factor);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_univariates<relation_idx + 1>(
                univariate_accumulators, extended_edges, relation_parameters, scaling_factor);
        }
    }

  public:
    // TODO(luke): Potentially make RelationUnivarites (tuple of tuples of Univariates) a class and make these utility
    // functions class methods. Alternatively, move all of these tuple utilities (and the ones living elsewhere) to
    // their own module.
    /**
     * Utility methods for tuple of tuples of Univariates
     */

    /**
     * @brief Extend Univariates to specified size then sum them
     *
     * @tparam extended_size Size after extension
     * @param tuple A tuple of tuples of Univariates
     * @param result A Univariate of length extended_size
     */
    template <size_t extended_size>
    static void extend_and_batch_univariates(auto& tuple,
                                             const barretenberg::PowUnivariate<FF>& pow_univariate,
                                             barretenberg::Univariate<FF, extended_size>& result)
    {
        // Random poly R(X) = (1-X) + X.zeta_pow
        auto random_poly_edge = barretenberg::Univariate<FF, 2>({ 1, pow_univariate.zeta_pow });
        barretenberg::BarycentricData<FF, 2, extended_size> pow_zeta_univariate_extender =
            barretenberg::BarycentricData<FF, 2, extended_size>();
        barretenberg::Univariate<FF, extended_size> extended_random_polynomial_edge =
            pow_zeta_univariate_extender.extend(random_poly_edge);

        auto extend_and_sum = [&]<size_t relation_idx, size_t subrelation_idx, typename Element>(Element& element) {
            using Relation = typename std::tuple_element<relation_idx, Relations>::type;

            // TODO(#224)(Cody): this barycentric stuff should be more built-in?
            barretenberg::BarycentricData<FF, Element::LENGTH, extended_size> barycentric_utils;
            auto extended = barycentric_utils.extend(element);

            const bool is_subrelation_linearly_independent =
                Relation::template is_subrelation_linearly_independent<subrelation_idx>();
            if (is_subrelation_linearly_independent) {
                // if subrelation is linearly independent, multiply by random polynomial
                result += extended * extended_random_polynomial_edge;
            } else {
                // if subrelation is pure sum over hypercube, don't multiply by random polynomial
                result += extended;
            }
        };
        apply_to_tuple_of_tuples(tuple, extend_and_sum);
    }

    /**
     * @brief Set all coefficients of Univariates to zero
     *
     * @details After computing the round univariate, it is necessary to zero-out the accumulators used to compute it.
     */
    static void zero_univariates(auto& tuple)
    {
        auto set_to_zero = []<size_t, size_t>(auto& element) {
            std::fill(element.evaluations.begin(), element.evaluations.end(), FF(0));
        };
        apply_to_tuple_of_tuples(tuple, set_to_zero);
    }

    /**
     * @brief Scale Univaraites by consecutive powers of the provided challenge
     *
     * @param tuple Tuple of tuples of Univariates
     * @param challenge
     * @param current_scalar power of the challenge
     */
    static void scale_univariates(auto& tuple, const FF& challenge, FF current_scalar)
    {
        auto scale_by_consecutive_powers_of_challenge = [&]<size_t, size_t>(auto& element) {
            element *= current_scalar;
            current_scalar *= challenge;
        };
        apply_to_tuple_of_tuples(tuple, scale_by_consecutive_powers_of_challenge);
    }

    /**
     * @brief General purpose method for applying an operation to a tuple of tuples of Univariates
     *
     * @tparam Operation Any operation valid on Univariates
     * @tparam outer_idx Index into the outer tuple
     * @tparam inner_idx Index into the inner tuple
     * @param tuple A Tuple of tuples of Univariates
     * @param operation Operation to apply to Univariates
     */
    template <class Operation, size_t outer_idx = 0, size_t inner_idx = 0>
    static void apply_to_tuple_of_tuples(auto& tuple, Operation&& operation)
    {
        auto& inner_tuple = std::get<outer_idx>(tuple);
        auto& univariate = std::get<inner_idx>(inner_tuple);

        // Apply the specified operation to each Univariate
        operation.template operator()<outer_idx, inner_idx>(univariate);

        const size_t inner_size = std::tuple_size_v<std::decay_t<decltype(std::get<outer_idx>(tuple))>>;
        const size_t outer_size = std::tuple_size_v<std::decay_t<decltype(tuple)>>;

        // Recurse over inner and outer tuples
        if constexpr (inner_idx + 1 < inner_size) {
            apply_to_tuple_of_tuples<Operation, outer_idx, inner_idx + 1>(tuple, std::forward<Operation>(operation));
        } else if constexpr (outer_idx + 1 < outer_size) {
            apply_to_tuple_of_tuples<Operation, outer_idx + 1, 0>(tuple, std::forward<Operation>(operation));
        }
    }

    /**
     * @brief Componentwise addition of two tuples
     * @details Used for adding tuples of Univariates but in general works for any object for which += is
     * defined. The result is stored in the first tuple.
     *
     * @tparam T Type of the elements contained in the tuples
     * @param tuple_1 First summand. Result stored in this tuple
     * @param tuple_2 Second summand
     */
    template <typename... T>
    static constexpr void add_tuples(std::tuple<T...>& tuple_1, const std::tuple<T...>& tuple_2)
    {
        auto add_tuples_helper = [&]<std::size_t... I>(std::index_sequence<I...>)
        {
            ((std::get<I>(tuple_1) += std::get<I>(tuple_2)), ...);
        };

        add_tuples_helper(std::make_index_sequence<sizeof...(T)>{});
    }

    /**
     * @brief Componentwise addition of nested tuples (tuples of tuples)
     * @details Used for summing tuples of tuples of Univariates. Needed for Sumcheck multithreading. Each thread
     * accumulates realtion contributions across a portion of the hypecube and then the results are accumulated into a
     * single nested tuple.
     *
     * @tparam Tuple
     * @tparam Index Index into outer tuple
     * @param tuple_1 First nested tuple summand. Result stored here
     * @param tuple_2 Second summand
     */
    template <typename Tuple, std::size_t Index = 0>
    static constexpr void add_nested_tuples(Tuple& tuple_1, const Tuple& tuple_2)
    {
        if constexpr (Index < std::tuple_size<Tuple>::value) {
            add_tuples(std::get<Index>(tuple_1), std::get<Index>(tuple_2));
            add_nested_tuples<Tuple, Index + 1>(tuple_1, tuple_2);
        }
    }
};

template <typename Flavor> class SumcheckVerifierRound {

    using Relations = typename Flavor::Relations;
    using RelationEvaluations = typename Flavor::RelationValues;

  public:
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;

    bool round_failed = false;

    Relations relations;
    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t MAX_RANDOM_RELATION_LENGTH = Flavor::MAX_RANDOM_RELATION_LENGTH;

    FF target_total_sum = 0;

    RelationEvaluations relation_evaluations;

    // Verifier constructor
    explicit SumcheckVerifierRound() { zero_elements(relation_evaluations); };

    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to calculate
     * a contribution to the purported value of the full Honk relation. These are stored in `evaluations`. Adding these
     * together, with appropriate scaling factors, produces the expected value of the full Honk relation. This value is
     * checked against the final value of the target total sum, defined as sigma_d.
     */
    FF compute_full_honk_relation_purported_value(ClaimedEvaluations purported_evaluations,
                                                  const proof_system::RelationParameters<FF>& relation_parameters,
                                                  const barretenberg::PowUnivariate<FF>& pow_univariate,
                                                  const FF alpha)
    {
        accumulate_relation_evaluations<>(
            purported_evaluations, relation_parameters, pow_univariate.partial_evaluation_constant);

        auto running_challenge = FF(1);
        auto output = FF(0);
        scale_and_batch_elements(relation_evaluations, alpha, running_challenge, output);
        return output;
    }

    /**
     * @brief check if S^{l}(0) + S^{l}(1) = S^{l-1}(u_{l-1}) = sigma_{l} (or 0 if l=0)
     *
     * @param univariate T^{l}(X), the round univariate that is equal to S^{l}(X)/( (1−X) + X⋅ζ^{ 2^l } )
     */
    bool check_sum(barretenberg::Univariate<FF, MAX_RANDOM_RELATION_LENGTH>& univariate)
    {
        // S^{l}(0) = ( (1−0) + 0⋅ζ^{ 2^l } ) ⋅ T^{l}(0) = T^{l}(0)
        // S^{l}(1) = ( (1−1) + 1⋅ζ^{ 2^l } ) ⋅ T^{l}(1) = ζ^{ 2^l } ⋅ T^{l}(1)
        FF total_sum = univariate.value_at(0) + univariate.value_at(1);
        // target_total_sum = sigma_{l} =
        // TODO(#673): Conditionals like this can go away once native verification is is just recursive verification
        // with a simulated builder.
        bool sumcheck_round_failed(false);
        if constexpr (IsRecursiveFlavor<Flavor>) {
            sumcheck_round_failed = (target_total_sum != total_sum).get_value();
        } else {
            sumcheck_round_failed = (target_total_sum != total_sum);
        }

        round_failed = round_failed || sumcheck_round_failed;
        return !sumcheck_round_failed;
    };

    /**
     * @brief After checking that the univariate is good for this round, compute the next target sum.
     *
     * @param univariate T^l(X), given by its evaluations over {0,1,2,...},
     * equal to S^{l}(X)/( (1−X) + X⋅ζ^{ 2^l } )
     * @param round_challenge u_l
     * @return FF sigma_{l+1} = S^l(u_l)
     */
    FF compute_next_target_sum(barretenberg::Univariate<FF, MAX_RANDOM_RELATION_LENGTH>& univariate,
                               FF& round_challenge)
    {
        // IMPROVEMENT(Cody): Use barycentric static method, maybe implement evaluation as member
        // function on Univariate.
        auto barycentric = barretenberg::BarycentricData<FF, MAX_RANDOM_RELATION_LENGTH, MAX_RANDOM_RELATION_LENGTH>();
        // Evaluate T^{l}(u_{l})
        target_total_sum = barycentric.evaluate(univariate, round_challenge);

        return target_total_sum;
    }

  private:
    // TODO(#224)(Cody): make uniform with accumulate_relation_univariates
    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to calculate
     * a contribution to the purported value of the full Honk relation. These are stored in `evaluations`. Adding these
     * together, with appropriate scaling factors, produces the expected value of the full Honk relation. This value is
     * checked against the final value of the target total sum (called sigma_0 in the thesis).
     */
    template <size_t relation_idx = 0>
    // TODO(#224)(Cody): Input should be an array?
    void accumulate_relation_evaluations(ClaimedEvaluations purported_evaluations,
                                         const proof_system::RelationParameters<FF>& relation_parameters,
                                         const FF& partial_evaluation_constant)
    {
        std::get<relation_idx>(relations).add_full_relation_value_contribution(
            std::get<relation_idx>(relation_evaluations),
            purported_evaluations,
            relation_parameters,
            partial_evaluation_constant);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_evaluations<relation_idx + 1>(
                purported_evaluations, relation_parameters, partial_evaluation_constant);
        }
    }

  public:
    /**
     * Utility methods for tuple of arrays
     */

    /**
     * @brief Set each element in a tuple of arrays to zero.
     * @details FF's default constructor may not initialize to zero (e.g., barretenberg::fr), hence we can't rely on
     * aggregate initialization of the evaluations array.
     */
    template <size_t idx = 0> static void zero_elements(auto& tuple)
    {
        auto set_to_zero = [](auto& element) { std::fill(element.begin(), element.end(), FF(0)); };
        apply_to_tuple_of_arrays(set_to_zero, tuple);
    };

    /**
     * @brief Scale elements by consecutive powers of the challenge then sum
     * @param result Batched result
     */
    static void scale_and_batch_elements(auto& tuple, const FF& challenge, FF current_scalar, FF& result)
    {
        auto scale_by_challenge_and_accumulate = [&](auto& element) {
            for (auto& entry : element) {
                result += entry * current_scalar;
                current_scalar *= challenge;
            }
        };
        apply_to_tuple_of_arrays(scale_by_challenge_and_accumulate, tuple);
    }

    /**
     * @brief General purpose method for applying a tuple of arrays (of FFs)
     *
     * @tparam Operation Any operation valid on elements of the inner arrays (FFs)
     * @param tuple Tuple of arrays (of FFs)
     */
    template <typename Operation, size_t idx = 0, typename... Ts>
    static void apply_to_tuple_of_arrays(Operation&& operation, std::tuple<Ts...>& tuple)
    {
        auto& element = std::get<idx>(tuple);

        std::invoke(std::forward<Operation>(operation), element);

        if constexpr (idx + 1 < sizeof...(Ts)) {
            apply_to_tuple_of_arrays<Operation, idx + 1>(operation, tuple);
        }
    }
};
} // namespace proof_system::honk::sumcheck
