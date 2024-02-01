/**
 * @file generic_permutation_relation.hpp
 * @author Rumata888
 * @brief This file contains the template for the generic permutation that can be specialized to enforce various
 * permutations (for explanation on how to define them, see "relation_definer.hpp")
 *
 */
#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {
/**
 * @brief Specifies positions of elements in the tuple of entities received from methods in the Settings class
 *
 */
enum GenericPermutationSettingIndices {
    INVERSE_POLYNOMIAL_INDEX,                          /* The index of the inverse polynomial*/
    ENABLE_INVERSE_CORRECTNESS_CHECK_POLYNOMIAL_INDEX, /* The index of the polynomial enabling first subrelation*/
    FIRST_PERMUTATION_SET_ENABLE_POLYNOMIAL_INDEX,  /* The index of the polynomial that adds an element from the first
                                                       set to the sum*/
    SECOND_PERMUTATION_SET_ENABLE_POLYNOMIAL_INDEX, /* The index of the polynomial that adds an element from the second
                                                       set to the sum*/

    PERMUTATION_SETS_START_POLYNOMIAL_INDEX, /* The starting index of the polynomials that are used in the permutation
                                                sets*/
};

template <typename Settings, typename FF_> class GenericPermutationRelationImpl {
  public:
    using FF = FF_;
    // Read and write terms counts should stay set to 1 unless we want to permute several columns at once as accumulated
    // sets (not as tuples).
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    // 1 + polynomial degree of this relation
    static constexpr size_t LENGTH = READ_TERMS + WRITE_TERMS + 3; // 5

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        LENGTH, // inverse polynomial correctness sub-relation
        LENGTH  // log-derived terms subrelation
    };

    /**
     * @brief We apply the power polynomial only to the first subrelation
     *
     *@details The first subrelation establishes correspondence between the inverse polynomial elements and the terms.
     *The second relation computes the inverses of individual terms, which are then summed up with sumcheck
     *
     */
    static constexpr std::array<bool, 2> SUBRELATION_LINEARLY_INDEPENDENT = { true, false };

    /**
     * @brief Check if we need to compute the inverse polynomial element value for this row
     * @details This proxies to a method in the Settings class
     *
     * @param row All values at row
     */
    template <typename AllValues> static bool operation_exists_at_row(const AllValues& row)

    {
        return Settings::inverse_polynomial_is_computed_at_row(row);
    }

    /**
     * @brief Get the inverse permutation polynomial (needed to compute its value)
     *
     */
    template <typename AllEntities> static auto& get_inverse_polynomial(AllEntities& in)
    {
        // WIRE containing the inverse of the product of terms at this row. Used to reconstruct individual inversed
        // terms
        return std::get<INVERSE_POLYNOMIAL_INDEX>(Settings::get_nonconst_entities(in));
    }

    /**
     * @brief Get selector/wire switching on(1) or off(0) inverse computation
     *
     */
    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        // WIRE/SELECTOR enabling the permutation used in the sumcheck computation. This affects the first subrelation
        return Accumulator(
            View(std::get<ENABLE_INVERSE_CORRECTNESS_CHECK_POLYNOMIAL_INDEX>(Settings::get_const_entities(in))));
    }

    /**
     * @brief Compute if the value from the first set exists in this row
     *
     * @tparam read_index Kept for compatibility with lookups, behavior doesn't change
     */
    template <typename Accumulator, size_t read_index, typename AllEntities>
    static Accumulator compute_read_term_predicate(const AllEntities& in)

    {
        static_assert(read_index < READ_TERMS);
        using View = typename Accumulator::View;

        // The selector/wire value that determines that an element from the first set needs to be included. Can be
        // different from the wire used in the write part.
        return Accumulator(
            View(std::get<FIRST_PERMUTATION_SET_ENABLE_POLYNOMIAL_INDEX>(Settings::get_const_entities(in))));
    }

    /**
     * @brief Compute if the value from the second set exists in this row
     *
     * @tparam write_index Kept for compatibility with lookups, behavior doesn't change
     */
    template <typename Accumulator, size_t write_index, typename AllEntities>
    static Accumulator compute_write_term_predicate(const AllEntities& in)
    {
        static_assert(write_index < WRITE_TERMS);
        using View = typename Accumulator::View;

        // The selector/wire value that determines that an element from the second set needs to be included. Can be
        // different from the wire used in the read part.
        return Accumulator(
            View(std::get<SECOND_PERMUTATION_SET_ENABLE_POLYNOMIAL_INDEX>(Settings::get_const_entities(in))));
    }

    /**
     * @brief Compute the value of a single item in the set
     *
     * @details Computes the polynomial \gamma + \sum_{i=0}^{num_columns}(column_i*\beta^i), so the tuple of columns is
     * in the first set
     *
     * @tparam read_index Kept for compatibility with lookups, behavior doesn't change
     *
     * @param params Used for beta and gamma
     */
    template <typename Accumulator, size_t read_index, typename AllEntities, typename Parameters>
    static Accumulator compute_read_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        static_assert(read_index < READ_TERMS);

        // Retrieve all polynomials used
        const auto all_polynomials = Settings::get_const_entities(in);

        auto result = Accumulator(0);

        // Iterate over tuple and sum as a polynomial over beta
        bb::constexpr_for<PERMUTATION_SETS_START_POLYNOMIAL_INDEX,
                          PERMUTATION_SETS_START_POLYNOMIAL_INDEX + Settings::COLUMNS_PER_SET,
                          1>([&]<size_t i>() { result = result * params.beta + View(std::get<i>(all_polynomials)); });

        const auto& gamma = params.gamma;
        return result + gamma;
    }

    /**
     * @brief Compute the value of a single item in the set
     *
     * @details Computes the polynomial \gamma + \sum_{i=0}^{num_columns}(column_i*\beta^i), so the tuple of columns is
     * in the second set
     *
     * @tparam write_index Kept for compatibility with lookups, behavior doesn't change
     *
     * @param params Used for beta and gamma
     */
    template <typename Accumulator, size_t write_index, typename AllEntities, typename Parameters>
    static Accumulator compute_write_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        static_assert(write_index < WRITE_TERMS);

        // Get all used entities
        const auto& used_entities = Settings::get_const_entities(in);

        auto result = Accumulator(0);
        // Iterate over tuple and sum as a polynomial over beta
        bb::constexpr_for<PERMUTATION_SETS_START_POLYNOMIAL_INDEX + Settings::COLUMNS_PER_SET,
                          PERMUTATION_SETS_START_POLYNOMIAL_INDEX + 2 * Settings::COLUMNS_PER_SET,
                          1>([&]<size_t i>() { result = result * params.beta + View(std::get<i>(used_entities)); });

        const auto& gamma = params.gamma;
        return result + gamma;
    }

    /**
     * @brief Expression for generic log-derivative-based set permutation.
     * @param accumulator transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Accumulator edges.
     * @param relation_params contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor)
    {
        accumulate_logderivative_permutation_subrelation_contributions<FF,
                                                                       GenericPermutationRelationImpl<Settings, FF>>(
            accumulator, in, params, scaling_factor);
    }
};

template <typename Settings, typename FF>
using GenericPermutationRelation = Relation<GenericPermutationRelationImpl<Settings, FF>>;

template <typename Settings, typename FF> using GenericPermutation = GenericPermutationRelationImpl<Settings, FF>;

} // namespace bb
