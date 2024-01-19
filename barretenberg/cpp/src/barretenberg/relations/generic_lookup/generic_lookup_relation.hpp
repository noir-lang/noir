/**
 * @file generic_lookup_relation.hpp
 * @author Rumata888
 * @brief This file contains the template for the generic lookup that can be specialized to enforce various
 * lookups (for explanation on how to define them, see "relation_definer.hpp")
 *
 * @details Lookup is a mechanism to ensure that a particular value or tuple of values (these can be values of
 * witnesses, selectors or a function of these) is contained within a particular set. It is a relative of set
 * permutation, but has a one-to-many relationship beween elements that are being looked up and the table of values they
 * are being looked up from. In this relation template we use the following terminology:
 * + READ - the action of looking up the value in the table
 * + WRITE - the action of adding the value to the lookup table
 *
 * TODO(@Rumata888): Talk to Zac why "lookup_read_count" refers to the count of the looked up element in the multiset.
 * (The value is applied to the write predicate, so it is confusing).
 */
#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::honk::sumcheck {
/**
 * @brief Specifies positions of elements in the tuple of entities received from methods in the Settings class
 *
 */

template <typename Settings, typename FF_> class GenericLookupRelationImpl {
  public:
    using FF = FF_;

    // Read terms specified how many maximum lookups can be performed in 1 row
    static constexpr size_t READ_TERMS = Settings::READ_TERMS;

    // Looked up entries can be a basic tuple, a scaled tuple or completely arbitrary
    enum READ_TERM_TYPES { READ_BASIC_TUPLE = 0, READ_SCALED_TUPLE, READ_ARBITRARY };

    // Write terms specifies how many insertions into the lookup table can be performed in 1 row
    static constexpr size_t WRITE_TERMS = Settings::WRITE_TERMS;

    // Entries put into the table are ever defined as a tuple or constructed arbitrarily
    enum WRITE_TERM_TYPES { WRITE_BASIC_TUPLE = 0, WRITE_ARBITRARY };

    // Lookup tuple size specifies how many values are bundled together to represent a single entry in the lookup table.
    // For example, it would be 1 for a range constraint lookup, or 3 for XOR lookup
    static constexpr size_t LOOKUP_TUPLE_SIZE = Settings::LOOKUP_TUPLE_SIZE;

    /**
     * @brief Compute the maximum degree of read terms
     *
     * @details We need this to evaluate the length of the subrelations correctly
     * @return constexpr size_t
     */
    static constexpr size_t compute_maximum_read_term_degree()
    {
        size_t maximum_degree = 0;
        for (size_t i = 0; i < READ_TERMS; i++) {
            size_t current_degree = 0;
            if (Settings::READ_TERM_TYPES[i] == READ_BASIC_TUPLE) {
                current_degree = 1;
            } else if (Settings::READ_TERM_TYPES[i] == READ_SCALED_TUPLE) {
                current_degree = 2;
            } else {
                current_degree = Settings::READ_TERM_DEGREE;
            }
            maximum_degree = std::max(current_degree, maximum_degree);
        }
        return maximum_degree;
    }

    /**
     * @brief Compute the maximum degree of write terms
     *
     * @details We need this to evaluate the length of the subrelations correctly
     * @return constexpr size_t
     */
    static constexpr size_t compute_maximum_write_term_degree()
    {
        size_t maximum_degree = 0;
        for (size_t i = 0; i < WRITE_TERMS; i++) {
            size_t current_degree = 0;
            if (Settings::WRITE_TERM_TYPES[i] == WRITE_BASIC_TUPLE) {
                current_degree = 1;
            } else {
                current_degree = Settings::WRITE_TERM_DEGREE;
            }
            maximum_degree = std::max(current_degree, maximum_degree);
        }
        return maximum_degree;
    }

    /**
     * @brief Compute the degree of of the product of read terms
     *
     * @details The degree of the inverse polynomial check subrelation is dependent on this value
     *
     * @return constexpr size_t
     */
    static constexpr size_t compute_read_term_product_degree()
    {
        size_t accumulated_degree = 0;
        for (size_t i = 0; i < READ_TERMS; i++) {
            size_t current_degree = 0;
            if (Settings::READ_TERM_TYPES[i] == READ_BASIC_TUPLE) {
                current_degree = 1;
            } else if (Settings::READ_TERM_TYPES[i] == READ_SCALED_TUPLE) {
                current_degree = 2;
            } else {
                current_degree = Settings::READ_TERM_DEGREE;
            }
            accumulated_degree += current_degree;
        }
        return accumulated_degree;
    }

    /**
     * @brief Compute the degree of of the product of write terms
     *
     * @details The degree of the inverse polynomial check subrelation is dependent on this value
     *
     * @return constexpr size_t
     */
    static constexpr size_t compute_write_term_product_degree()
    {
        size_t accumulated_degree = 0;
        for (size_t i = 0; i < WRITE_TERMS; i++) {
            size_t current_degree = 0;
            if (Settings::WRITE_TERM_TYPES[i] == WRITE_BASIC_TUPLE) {
                current_degree = 1;
            } else {
                current_degree = Settings::WRITE_TERM_DEGREE;
            }
            accumulated_degree += current_degree;
        }
        return accumulated_degree;
    }

    // Read term degree is dependent on what type of read term we use
    static constexpr size_t READ_TERM_DEGREE = compute_maximum_read_term_degree();
    static_assert(READ_TERM_DEGREE != 0);

    // Write  term degree is dependent on what type of write term we use
    static constexpr size_t WRITE_TERM_DEGREE = compute_maximum_write_term_degree();

    static_assert(WRITE_TERM_DEGREE != 0);

    // Compute the length of the inverse polynomial correctness sub-relation MAX(product of terms * inverse, inverse
    // exists polynomial) + 1;
    static constexpr size_t FIRST_SUBRELATION_LENGTH =
        std::max((compute_read_term_product_degree() + compute_write_term_product_degree() + 1),
                 Settings::INVERSE_EXISTS_POLYNOMIAL_DEGREE) +
        1;

    // Compute the length of the log-derived term subrelation MAX(read term * enable read, write term * write count *
    // enable write)
    static constexpr size_t SECOND_SUBRELATION_LENGTH = std::max(READ_TERM_DEGREE + 1, WRITE_TERM_DEGREE + 2);
    // 1 + polynomial degree of this relation
    static constexpr size_t LENGTH = std::max(FIRST_SUBRELATION_LENGTH, SECOND_SUBRELATION_LENGTH);

    // The structure of polynomial tuple returned from Settings' functions get_const_entities and get_nonconst_entities
    // is the following:
    // 1) 1 Polynomial used to contain the inverse product from which we reconstruct individual inverses
    // used in the sum
    // 2) WRITE_TERMS number of polynomials representing how much each write term has been read
    // 3) READ_TERMS number of polynomials enabling the addition of a particular read term in this row (should we lookup
    // or not)
    // 4) WRITE_TERMS number of polynomials enabling a particular write term in this row (should we add it to
    // the lookup table or not)
    // 5) For each read term depending on its type (READ_BASIC_TUPLE, READ_SCALED_TUPLE or READ_ARBITRARY):
    //  1. In case of basic tuple LOOKUP_TUPLE_SIZE polynomials the combination of whose values in a row is supposed to
    //  represent the looked up entry
    //  2. In case of scaled tuple there are LOOKUP_TUPLE_SIZE previous accumulator polynomials, LOOKUP_TUPLE_SIZE
    //  scaling polynomials and LOOKUP_TUPLE_SIZE current accumulator polynomials. The tuple is comprised of values
    //  (current_accumulator-scale*previous_accumulator)
    //  3. In the arbitrary case the are no additional
    //  polynomials, because the logic is completely decided in the settings
    // 6) For each  write term depending on its type (READ_BASIC_TUPLE or READ_ARBITRARY):
    //  1. In case of basic tuple LOOKUP_TUPLE_SIZE polynomials the combination of whose values in a row is supposed to
    //  represent the entry written into the lookup table
    //  2. In the arbitrary case the are no additional write term polynomials,
    //  because the logic is completely decided in the settings
    static constexpr size_t INVERSE_POLYNOMIAL_INDEX = 0;
    static constexpr size_t LOOKUP_READ_COUNT_START_POLYNOMIAL_INDEX = 1;
    static constexpr size_t LOOKUP_READ_TERM_PREDICATE_START_POLYNOMIAL_INDEX =
        LOOKUP_READ_COUNT_START_POLYNOMIAL_INDEX + WRITE_TERMS;
    static constexpr size_t LOOKUP_WRITE_TERM_PREDICATE_START_POLYNOMIAL_INDEX =
        LOOKUP_READ_TERM_PREDICATE_START_POLYNOMIAL_INDEX + READ_TERMS;
    static constexpr size_t LOOKUP_READ_PREDICATE_START_POLYNOMIAL_INDEX =
        LOOKUP_WRITE_TERM_PREDICATE_START_POLYNOMIAL_INDEX + WRITE_TERMS;

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

        // A lookup could be enabled by one of several selectors or witnesses, so we want to give as much freedom as
        // possible to the implementor
        return Settings::template compute_inverse_exists<Accumulator>(in);
    }

    /**
     * @brief Returns the number of times a particular value is written (how many times it is being looked up)
     *
     * @details Lookup read counts should be independent columns, so there is no need to call a separate function
     *
     * @tparam Accumulator
     * @tparam index The index of the write predicate to which this count belongs
     * @tparam AllEntities
     * @param in
     * @return Accumulator
     */
    template <typename Accumulator, size_t index, typename AllEntities>
    static Accumulator lookup_read_counts(const AllEntities& in)
    {

        static_assert(index < WRITE_TERMS);
        using View = typename Accumulator::View;

        return Accumulator(
            View(std::get<LOOKUP_READ_COUNT_START_POLYNOMIAL_INDEX + index>(Settings::get_const_entities(in))));
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
        return Accumulator(View(std::get<LOOKUP_READ_TERM_PREDICATE_START_POLYNOMIAL_INDEX + read_index>(
            Settings::get_const_entities(in))));
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

        // The selector/wire value that determines that an element from the first set needs to be included. Can be
        // different from the wire used in the write part.
        return Accumulator(View(std::get<LOOKUP_WRITE_TERM_PREDICATE_START_POLYNOMIAL_INDEX + write_index>(
            Settings::get_const_entities(in))));
    }

    /**
     * @brief Compute where the polynomials defining a particular read term are located
     *
     * @details We pass polynomials involved in read an write terms from settings as a tuple of references. However,
     * depending on the type of read term different number of polynomials can be used to compute it. So we need to
     * compute the offset in the tuple iteratively
     *
     * @param read_index Index of the read term
     * @return constexpr size_t
     */
    static constexpr size_t compute_read_term_polynomial_offset(size_t read_index)
    {
        // If it's the starting index, then there is nothing to compute, just get the starting index
        if (read_index == 0) {
            return LOOKUP_READ_PREDICATE_START_POLYNOMIAL_INDEX;
        }

        // If the previous term used basic tuple lookup, add lookup tuple size (it was using just a linear combination
        // of polynomials)
        if (Settings::READ_TERM_TYPES[read_index - 1] == READ_BASIC_TUPLE) {
            return compute_read_term_polynomial_offset(read_index - 1) + LOOKUP_TUPLE_SIZE;
        }

        // If the previous term used scaled tuple lookup, add lookup tuple size x 3 (it was using just a linear
        // combination of differences (current - previous⋅scale))

        if (Settings::READ_TERM_TYPES[read_index - 1] == READ_SCALED_TUPLE) {
            return compute_read_term_polynomial_offset(read_index - 1) + 3 * LOOKUP_TUPLE_SIZE;
        }
        // In case of arbitrary read term, no polynomials from the tuple are being used
        if (Settings::READ_TERM_TYPES[read_index - 1] == READ_ARBITRARY) {
            return compute_read_term_polynomial_offset(read_index - 1);
        }
        return SIZE_MAX;
    }

    /**
     * @brief Compute where the polynomials defining a particular write term are located
     *
     * @details We pass polynomials involved in read an write terms from settings as a tuple of references. However,
     * depending on the type of term different number of polynomials can be used to compute it. So we need to
     * compute the offset in the tuple iteratively
     *
     * @param write_index Index of the write term
     * @return constexpr size_t
     */
    static constexpr size_t compute_write_term_polynomial_offset(size_t write_index)
    {
        // If it's the starting index, then we need to find out how many polynomials were taken by read terms
        if (write_index == 0) {
            return compute_read_term_polynomial_offset(READ_TERMS);
        }

        // If the previous term used basic tuple lookup, add lookup tuple size (it was using just a linear combination
        // of polynomials)
        if (Settings::WRITE_TERM_TYPES[write_index - 1] == WRITE_BASIC_TUPLE) {
            return compute_write_term_polynomial_offset(write_index - 1) + LOOKUP_TUPLE_SIZE;
        }

        // In case of arbitrary write term, no polynomials from the tuple are being used
        if (Settings::WRITE_TERM_TYPES[write_index - 1] == WRITE_ARBITRARY) {
            return compute_write_term_polynomial_offset(write_index - 1);
        }
        return SIZE_MAX;
    }

    /**
     * @brief Compute the value of a single item in the set
     *
     * @details Computes the polynomial \gamma + \sum_{i=0}^{num_columns}(column_i*\beta^i), so the tuple of columns is
     * in the first set
     *
     * @tparam read_index The chosen polynomial relation
     *
     * @param params Used for beta and gamma
     */
    template <typename Accumulator, size_t read_index, typename AllEntities, typename Parameters>
    static Accumulator compute_read_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        static_assert(read_index < READ_TERMS);
        constexpr size_t start_polynomial_index = compute_read_term_polynomial_offset(read_index);
        if constexpr (Settings::READ_TERM_TYPES[read_index] == READ_BASIC_TUPLE) {
            // Retrieve all polynomials used
            const auto all_polynomials = Settings::get_const_entities(in);

            auto result = Accumulator(0);

            // Iterate over tuple and sum as a polynomial over beta
            bb::constexpr_for<start_polynomial_index, start_polynomial_index + LOOKUP_TUPLE_SIZE, 1>(
                [&]<size_t i>() { result = (result * params.beta) + View(std::get<i>(all_polynomials)); });
            const auto& gamma = params.gamma;
            return result + gamma;
        } else if constexpr (Settings::READ_TERM_TYPES[read_index] == READ_SCALED_TUPLE) {
            // Retrieve all polynomials used
            const auto all_polynomials = Settings::get_const_entities(in);

            auto result = Accumulator(0);
            // Iterate over tuple and sum as a polynomial over beta
            bb::constexpr_for<start_polynomial_index, start_polynomial_index + LOOKUP_TUPLE_SIZE, 1>([&]<size_t i>() {
                result = (result * params.beta) + View(std::get<i + 2 * LOOKUP_TUPLE_SIZE>(all_polynomials)) -
                         View(std::get<i + LOOKUP_TUPLE_SIZE>(all_polynomials)) * View(std::get<i>(all_polynomials));
            });
            const auto& gamma = params.gamma;
            return result + gamma;
        } else {

            return Settings::template compute_read_term<Accumulator, read_index>(in, params);
        }
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

        static_assert(write_index < WRITE_TERMS);

        using View = typename Accumulator::View;
        constexpr size_t start_polynomial_index = compute_write_term_polynomial_offset(write_index);

        if constexpr (Settings::WRITE_TERM_TYPES[write_index] == WRITE_BASIC_TUPLE) {
            // Retrieve all polynomials used
            const auto all_polynomials = Settings::get_const_entities(in);

            auto result = Accumulator(0);

            // Iterate over tuple and sum as a polynomial over beta
            bb::constexpr_for<start_polynomial_index, start_polynomial_index + LOOKUP_TUPLE_SIZE, 1>(
                [&]<size_t i>() { result = (result * params.beta) + View(std::get<i>(all_polynomials)); });
            const auto& gamma = params.gamma;
            return result + gamma;
        } else {
            // Sometimes we construct lookup tables on the fly from intermediate

            return Settings::template compute_write_term<Accumulator, write_index>(in, params);
        }
    }

    /**
     * @brief Expression for generic log-derivative-based set permutation.
     * @param accumulator transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Accumulator edges.
     * @param relation_params contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
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
        logderivative_library::
            accumulate_logderivative_lookup_subrelation_contributions<FF, GenericLookupRelationImpl<Settings, FF>>(
                accumulator, in, params, scaling_factor);
    }
};

template <typename Settings, typename FF>
using GenericLookupRelation = Relation<GenericLookupRelationImpl<Settings, FF>>;

template <typename Settings, typename FF> using GenericLookup = GenericLookupRelationImpl<Settings, FF>;

} // namespace bb::honk::sumcheck