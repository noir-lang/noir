

#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

/**
 * @brief This class contains an example of how to set LookupSettings classes used by the
 * GenericLookupRelationImpl class to specify a scaled lookup
 *
 * @details To create your own lookup:
 * 1) Create a copy of this class and rename it
 * 2) Update all the values with the ones needed for your lookup
 * 3) Update "DECLARE_LOOKUP_IMPLEMENTATIONS_FOR_ALL_SETTINGS" and "DEFINE_LOOKUP_IMPLEMENTATIONS_FOR_ALL_SETTINGS" to
 * include the new settings
 * 4) Add the relation with the chosen settings to Relations in the flavor (for example,"`
 *   using Relations = std::tuple<GenericLookupRelation<ExampleXorLookupSettings,
 * FF>>;)`
 *
 */
class lookup_byte_operations_lookup_settings {
  public:
    /**
     * @brief The number of read terms (how many lookups we perform) in each row
     *
     */
    static constexpr size_t READ_TERMS = 1;
    /**
     * @brief The number of write terms (how many additions to the lookup table we make) in each row
     *
     */
    static constexpr size_t WRITE_TERMS = 1;

    /**
     * @brief The type of READ_TERM used for each read index (basic and scaled)
     *
     */
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };

    /**
     * @brief They type of WRITE_TERM used for each write index
     *
     */
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };

    /**
     * @brief How many values represent a single lookup object. This value is used by the automatic read term
     * implementation in the relation in case the lookup is a basic or scaled tuple and in the write term if it's a
     * basic tuple
     *
     */
    static constexpr size_t LOOKUP_TUPLE_SIZE = 4;

    /**
     * @brief The polynomial degree of the relation telling us if the inverse polynomial value needs to be computed
     *
     */
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 2;

    /**
     * @brief The degree of the read term if implemented arbitrarily. This value is not used by basic and scaled read
     * terms, but will cause compilation error if not defined
     *
     */
    static constexpr size_t READ_TERM_DEGREE = 0;

    /**
     * @brief The degree of the write term if implemented arbitrarily. This value is not used by the basic write
     * term, but will cause compilation error if not defined
     *
     */

    static constexpr size_t WRITE_TERM_DEGREE = 0;

    /**
     * @brief If this method returns true on a row of values, then the inverse polynomial exists at this index.
     * Otherwise the value needs to be set to zero.
     *
     * @details If this is true then the lookup takes place in this row
     *
     */

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.avm_binary_bin_sel == 1 || in.avm_byte_lookup_bin_sel == 1);
    }

    /**
     * @brief Subprocedure for computing the value deciding if the inverse polynomial value needs to be checked in this
     * row
     *
     * @tparam Accumulator Type specified by the lookup relation
     * @tparam AllEntities Values/Univariates of all entities row
     * @param in Value/Univariate of all entities at row/edge
     * @return Accumulator
     */

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.avm_binary_bin_sel);
        const auto is_table_entry = View(in.avm_byte_lookup_bin_sel);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    /**
     * @brief Get all the entities for the lookup when need to update them
     *
     * @details The generic structure of this tuple is described in ./generic_lookup_relation.hpp . The following is
     description for the current case:
     The entities are returned as a tuple of references in the following order (this is for ):
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that specifies how many times the lookup table entry at this row has been looked up
     * - READ_TERMS entities/polynomials that enable individual lookup operations
     * - The entity/polynomial that enables adding an entry to the lookup table in this row
     * - LOOKUP_TUPLE_SIZE entities/polynomials representing the basic tuple being looked up as the first read term
     * - LOOKUP_TUPLE_SIZE entities/polynomials representing the previous accumulators in the second read term
     (scaled tuple)
     * - LOOKUP_TUPLE_SIZE entities/polynomials representing the shifts in the second read term (scaled tuple)
     * - LOOKUP_TUPLE_SIZE entities/polynomials representing the current accumulators in the second read term
     (scaled tuple)
     * - LOOKUP_TUPLE_SIZE entities/polynomials representing basic tuples added to the table
     *
     * @return All the entities needed for the lookup
     */

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {

        return std::forward_as_tuple(in.lookup_byte_operations,
                                     in.lookup_byte_operations_counts,
                                     in.avm_binary_bin_sel,
                                     in.avm_byte_lookup_bin_sel,
                                     in.avm_binary_op_id,
                                     in.avm_binary_ia_bytes,
                                     in.avm_binary_ib_bytes,
                                     in.avm_binary_ic_bytes,
                                     in.avm_byte_lookup_table_op_id,
                                     in.avm_byte_lookup_table_input_a,
                                     in.avm_byte_lookup_table_input_b,
                                     in.avm_byte_lookup_table_output);
    }

    /**
     * @brief Get all the entities for the lookup when we only need to read them
     * @details Same as in get_const_entities, but nonconst
     *
     * @return All the entities needed for the lookup
     */

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {

        return std::forward_as_tuple(in.lookup_byte_operations,
                                     in.lookup_byte_operations_counts,
                                     in.avm_binary_bin_sel,
                                     in.avm_byte_lookup_bin_sel,
                                     in.avm_binary_op_id,
                                     in.avm_binary_ia_bytes,
                                     in.avm_binary_ib_bytes,
                                     in.avm_binary_ic_bytes,
                                     in.avm_byte_lookup_table_op_id,
                                     in.avm_byte_lookup_table_input_a,
                                     in.avm_byte_lookup_table_input_b,
                                     in.avm_byte_lookup_table_output);
    }
};

template <typename FF_>
using lookup_byte_operations_relation = GenericLookupRelation<lookup_byte_operations_lookup_settings, FF_>;
template <typename FF_> using lookup_byte_operations = GenericLookup<lookup_byte_operations_lookup_settings, FF_>;

} // namespace bb
