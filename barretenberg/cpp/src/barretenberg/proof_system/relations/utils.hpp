#pragma once

#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"
#include <cstddef>
#include <functional>
#include <tuple>
namespace barretenberg {
template <typename Flavor> class RelationUtils {
    using FF = typename Flavor::FF;
    using PolynomialEvaluations = typename Flavor::AllValues;
    using Relations = typename Flavor::Relations;
    using RelationEvaluations = typename Flavor::TupleOfArraysOfValues;

    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;

  public:
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
    static void accumulate_relation_evaluations(PolynomialEvaluations evaluations,
                                                RelationEvaluations& relation_evaluations,
                                                const proof_system::RelationParameters<FF>& relation_parameters,
                                                const FF& partial_evaluation_constant)
    {
        using Relation = std::tuple_element_t<relation_idx, Relations>;
        Relation::accumulate(std::get<relation_idx>(relation_evaluations),
                             evaluations,
                             relation_parameters,
                             partial_evaluation_constant);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_evaluations<relation_idx + 1>(
                evaluations, relation_evaluations, relation_parameters, partial_evaluation_constant);
        }
    }

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
} // namespace barretenberg