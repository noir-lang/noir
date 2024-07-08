#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class lookup_into_kernel_lookup_settings {
  public:
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };
    static constexpr size_t LOOKUP_TUPLE_SIZE = 2;
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 4;
    static constexpr size_t READ_TERM_DEGREE = 0;
    static constexpr size_t WRITE_TERM_DEGREE = 0;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_q_kernel_lookup == 1 || in.kernel_q_public_input_kernel_add_to_table == 1);
    }

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.main_sel_q_kernel_lookup);
        const auto is_table_entry = View(in.kernel_q_public_input_kernel_add_to_table);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_into_kernel,
                                     in.lookup_into_kernel_counts,
                                     in.main_sel_q_kernel_lookup,
                                     in.kernel_q_public_input_kernel_add_to_table,
                                     in.main_ia,
                                     in.kernel_kernel_in_offset,
                                     in.kernel_kernel_inputs,
                                     in.main_clk);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_into_kernel,
                                     in.lookup_into_kernel_counts,
                                     in.main_sel_q_kernel_lookup,
                                     in.kernel_q_public_input_kernel_add_to_table,
                                     in.main_ia,
                                     in.kernel_kernel_in_offset,
                                     in.kernel_kernel_inputs,
                                     in.main_clk);
    }
};

template <typename FF_>
class lookup_into_kernel_relation : public GenericLookupRelation<lookup_into_kernel_lookup_settings, FF_> {
  public:
    static constexpr const char* NAME = "lookup_into_kernel";
};
template <typename FF_> using lookup_into_kernel = GenericLookup<lookup_into_kernel_lookup_settings, FF_>;

} // namespace bb