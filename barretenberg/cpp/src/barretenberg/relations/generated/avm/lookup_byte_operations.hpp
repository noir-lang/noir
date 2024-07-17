#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class lookup_byte_operations_lookup_settings {
  public:
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };
    static constexpr size_t LOOKUP_TUPLE_SIZE = 4;
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 4;
    static constexpr size_t READ_TERM_DEGREE = 0;
    static constexpr size_t WRITE_TERM_DEGREE = 0;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.binary_sel_bin == 1 || in.byte_lookup_sel_bin == 1);
    }

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.binary_sel_bin);
        const auto is_table_entry = View(in.byte_lookup_sel_bin);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_byte_operations,
                                     in.lookup_byte_operations_counts,
                                     in.binary_sel_bin,
                                     in.byte_lookup_sel_bin,
                                     in.binary_op_id,
                                     in.binary_ia_bytes,
                                     in.binary_ib_bytes,
                                     in.binary_ic_bytes,
                                     in.byte_lookup_table_op_id,
                                     in.byte_lookup_table_input_a,
                                     in.byte_lookup_table_input_b,
                                     in.byte_lookup_table_output);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_byte_operations,
                                     in.lookup_byte_operations_counts,
                                     in.binary_sel_bin,
                                     in.byte_lookup_sel_bin,
                                     in.binary_op_id,
                                     in.binary_ia_bytes,
                                     in.binary_ib_bytes,
                                     in.binary_ic_bytes,
                                     in.byte_lookup_table_op_id,
                                     in.byte_lookup_table_input_a,
                                     in.byte_lookup_table_input_b,
                                     in.byte_lookup_table_output);
    }
};

template <typename FF_>
class lookup_byte_operations_relation : public GenericLookupRelation<lookup_byte_operations_lookup_settings, FF_> {
  public:
    static constexpr const char* NAME = "LOOKUP_BYTE_OPERATIONS";
};
template <typename FF_> using lookup_byte_operations = GenericLookup<lookup_byte_operations_lookup_settings, FF_>;

} // namespace bb