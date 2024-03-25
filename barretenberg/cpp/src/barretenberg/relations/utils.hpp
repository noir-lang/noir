#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

namespace bb {

template <typename Flavor> class RelationUtils {
  public:
    using FF = typename Flavor::FF;
    using Relations = typename Flavor::Relations;
    using PolynomialEvaluations = typename Flavor::AllValues;
    using RelationEvaluations = typename Flavor::TupleOfArraysOfValues;
    using RelationSeparator = typename Flavor::RelationSeparator;

    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;

    /**
     * Utility methods for tuple of tuples of Univariates
     */
    /**
     * @brief General purpose method for applying an operation to a tuple of tuples of Univariates
     *
     * @tparam Operation Any operation valid on Univariates
     * @tparam outer_idx Index into the outer tuple
     * @tparam inner_idx Index into the inner tuple
     * @param tuple A Tuple of tuples of Univariates
     * @param operation Operation to apply to Univariates
     */
    template <size_t outer_idx = 0, size_t inner_idx = 0, class Operation>
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
            apply_to_tuple_of_tuples<outer_idx, inner_idx + 1, Operation>(tuple, std::forward<Operation>(operation));
        } else if constexpr (outer_idx + 1 < outer_size) {
            apply_to_tuple_of_tuples<outer_idx + 1, 0, Operation>(tuple, std::forward<Operation>(operation));
        }
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
     * @brief Scale Univaraites, each representing a subrelation, by different challenges
     *
     * @param tuple Tuple of tuples of Univariates
     * @param challenge Array of NUM_SUBRELATIONS - 1 challenges (because the first subrelation doesn't need to be
     * scaled)
     * @param current_scalar power of the challenge
     */
    static void scale_univariates(auto& tuple, const RelationSeparator& challenges, FF& current_scalar)
        requires bb::IsFoldingFlavor<Flavor>
    {
        size_t idx = 0;
        std::array<FF, NUM_SUBRELATIONS> tmp{ current_scalar };
        std::copy(challenges.begin(), challenges.end(), tmp.begin() + 1);
        auto scale_by_challenges = [&]<size_t, size_t>(auto& element) {
            element *= tmp[idx];
            idx++;
        };
        apply_to_tuple_of_tuples(tuple, scale_by_challenges);
    }

    /**
     * @brief Scale Univaraites by consecutive powers of the provided challenge
     *
     * @param tuple Tuple of tuples of Univariates
     * @param challenge
     * @param current_scalar power of the challenge
     */
    static void scale_univariates(auto& tuple, const RelationSeparator& challenge, FF& current_scalar)
        requires(!bb::IsFoldingFlavor<Flavor>)
    {
        auto scale_by_consecutive_powers_of_challenge = [&]<size_t, size_t>(auto& element) {
            element *= current_scalar;
            current_scalar *= challenge;
        };
        apply_to_tuple_of_tuples(tuple, scale_by_consecutive_powers_of_challenge);
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
     * accumulates relation contributions across a portion of the hypecube and then the results are accumulated into
     * a single nested tuple.
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

    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to
     * calculate a contribution to the purported value of the full Honk relation. These are stored in `evaluations`.
     * Adding these together, with appropriate scaling factors, produces the expected value of the full Honk
     * relation. This value is checked against the final value of the target total sum (called sigma_0 in the
     * thesis).
     */
    template <typename Parameters, size_t relation_idx = 0>
    // TODO(#224)(Cody): Input should be an array?
    inline static void accumulate_relation_evaluations(PolynomialEvaluations evaluations,
                                                       RelationEvaluations& relation_evaluations,
                                                       const Parameters& relation_parameters,
                                                       const FF& partial_evaluation_result)
    {
        using Relation = std::tuple_element_t<relation_idx, Relations>;
        Relation::accumulate(
            std::get<relation_idx>(relation_evaluations), evaluations, relation_parameters, partial_evaluation_result);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_evaluations<Parameters, relation_idx + 1>(
                evaluations, relation_evaluations, relation_parameters, partial_evaluation_result);
        }
    }

    /**
     * Utility methods for tuple of arrays
     */

    /**
     * @brief Set each element in a tuple of arrays to zero.
     * @details FF's default constructor may not initialize to zero (e.g., bb::fr), hence we can't rely on
     * aggregate initialization of the evaluations array.
     */
    template <size_t idx = 0> static void zero_elements(auto& tuple)
    {
        auto set_to_zero = [](auto& element) { std::fill(element.begin(), element.end(), FF(0)); };
        apply_to_tuple_of_arrays(set_to_zero, tuple);
    };

    /**
     * @brief Scale elements, representing evaluations of subrelations, by separate challenges then sum them
     * @param challenges Array of NUM_SUBRELATIONS - 1 challenges (because the first subrelation does not need to be
     * scaled)
     * @param result Batched result
     */
    static void scale_and_batch_elements(auto& tuple,
                                         const RelationSeparator& challenges,
                                         FF current_scalar,
                                         FF& result)
        requires bb::IsFoldingFlavor<Flavor>
    {
        size_t idx = 0;
        std::array<FF, NUM_SUBRELATIONS> tmp{ current_scalar };
        std::copy(challenges.begin(), challenges.end(), tmp.begin() + 1);
        auto scale_by_challenges_and_accumulate = [&](auto& element) {
            for (auto& entry : element) {
                result += entry * tmp[idx];
                idx++;
            }
        };
        apply_to_tuple_of_arrays(scale_by_challenges_and_accumulate, tuple);
    }

    /**
     * @brief Scales elements, representing evaluations of polynomials in subrelations, by separate challenges and then
     * sum them together. This function has identical functionality with the one above with the caveat that one such
     * evaluation is part of a linearly dependent subrelation and hence needs to be accumulated separately.
     *
     * @details Such functionality is needed when computing the evaluation of the full relation at a specific row in
     * the execution trace because a linearly dependent subrelation does not act on a specific row but rather on the
     * entire execution trace.
     *
     * @param tuple
     * @param challenges
     * @param current_scalar
     * @param result
     * @param linearly_dependent_contribution
     */
    static void scale_and_batch_elements(auto& tuple,
                                         const RelationSeparator& challenges,
                                         FF current_scalar,
                                         FF& result,
                                         FF& linearly_dependent_contribution)
        requires bb::IsFoldingFlavor<Flavor>
    {
        size_t idx = 0;
        std::array<FF, NUM_SUBRELATIONS> tmp{ current_scalar };

        std::copy(challenges.begin(), challenges.end(), tmp.begin() + 1);

        auto scale_by_challenge_and_accumulate =
            [&]<size_t relation_idx, size_t subrelation_idx, typename Element>(Element& element) {
                using Relation = typename std::tuple_element_t<relation_idx, Relations>;
                const bool is_subrelation_linearly_independent =
                    bb::subrelation_is_linearly_independent<Relation, subrelation_idx>();
                if (is_subrelation_linearly_independent) {
                    result += element * tmp[idx];
                } else {
                    linearly_dependent_contribution += element * tmp[idx];
                }
                idx++;
            };
        apply_to_tuple_of_arrays_elements(scale_by_challenge_and_accumulate, tuple);
    }

    /**
     * @brief Scale elements by consecutive powers of a given challenge then sum the result
     * @param result Batched result
     */
    static void scale_and_batch_elements(auto& tuple, const RelationSeparator& challenge, FF current_scalar, FF& result)
        requires(!bb::IsFoldingFlavor<Flavor>)
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
    template <size_t idx = 0, typename Operation, typename... Ts>
    static void apply_to_tuple_of_arrays(Operation&& operation, std::tuple<Ts...>& tuple)
    {
        auto& element = std::get<idx>(tuple);

        std::invoke(std::forward<Operation>(operation), element);

        if constexpr (idx + 1 < sizeof...(Ts)) {
            apply_to_tuple_of_arrays<idx + 1, Operation>(operation, tuple);
        }
    }

    /**
     * @brief Recursive template function to apply a specific operation on each element of several arrays in a tuple
     *
     * @details We need this method in addition to the apply_to_tuple_of_arrays when we aim to perform different
     * operations depending on the array element. More explicitly, in our codebase this method is used when the elements
     * of array are values of subrelations and we want to accumulate some of these values separately (the linearly
     * dependent contribution when we compute the evaluation of full rel_U(G)H at particular row.)
     */
    template <size_t outer_idx = 0, size_t inner_idx = 0, typename Operation, typename... Ts>
    static void apply_to_tuple_of_arrays_elements(Operation&& operation, std::tuple<Ts...>& tuple)
    {
        using Relation = typename std::tuple_element_t<outer_idx, Relations>;
        const auto subrelation_length = Relation::SUBRELATION_PARTIAL_LENGTHS.size();
        auto& element = std::get<outer_idx>(tuple);

        // Invoke the operation with outer_idx (array index) and inner_idx (element index) as template arguments
        operation.template operator()<outer_idx, inner_idx>(element[inner_idx]);

        if constexpr (inner_idx + 1 < subrelation_length) {
            // Recursively call for the next element within the same array
            apply_to_tuple_of_arrays_elements<outer_idx, inner_idx + 1, Operation>(std::forward<Operation>(operation),
                                                                                   tuple);
        } else if constexpr (outer_idx + 1 < sizeof...(Ts)) {
            // Move to the next array in the tuple
            apply_to_tuple_of_arrays_elements<outer_idx + 1, 0, Operation>(std::forward<Operation>(operation), tuple);
        }
    }
};
} // namespace bb
