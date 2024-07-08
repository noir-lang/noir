#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class lookup_opcode_gas_lookup_settings {
  public:
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };
    static constexpr size_t LOOKUP_TUPLE_SIZE = 3;
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 4;
    static constexpr size_t READ_TERM_DEGREE = 0;
    static constexpr size_t WRITE_TERM_DEGREE = 0;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_gas_accounting_active == 1 || in.gas_sel_gas_cost == 1);
    }

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.main_sel_gas_accounting_active);
        const auto is_table_entry = View(in.gas_sel_gas_cost);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_opcode_gas,
                                     in.lookup_opcode_gas_counts,
                                     in.main_sel_gas_accounting_active,
                                     in.gas_sel_gas_cost,
                                     in.main_opcode_val,
                                     in.main_l2_gas_op_cost,
                                     in.main_da_gas_op_cost,
                                     in.main_clk,
                                     in.gas_l2_gas_fixed_table,
                                     in.gas_da_gas_fixed_table);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_opcode_gas,
                                     in.lookup_opcode_gas_counts,
                                     in.main_sel_gas_accounting_active,
                                     in.gas_sel_gas_cost,
                                     in.main_opcode_val,
                                     in.main_l2_gas_op_cost,
                                     in.main_da_gas_op_cost,
                                     in.main_clk,
                                     in.gas_l2_gas_fixed_table,
                                     in.gas_da_gas_fixed_table);
    }
};

template <typename FF_>
class lookup_opcode_gas_relation : public GenericLookupRelation<lookup_opcode_gas_lookup_settings, FF_> {
  public:
    static constexpr const char* NAME = "lookup_opcode_gas";
};
template <typename FF_> using lookup_opcode_gas = GenericLookup<lookup_opcode_gas_lookup_settings, FF_>;

} // namespace bb