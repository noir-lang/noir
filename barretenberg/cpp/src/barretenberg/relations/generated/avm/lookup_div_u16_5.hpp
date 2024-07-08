#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class lookup_div_u16_5_lookup_settings {
  public:
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };
    static constexpr size_t LOOKUP_TUPLE_SIZE = 1;
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 4;
    static constexpr size_t READ_TERM_DEGREE = 0;
    static constexpr size_t WRITE_TERM_DEGREE = 0;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.alu_sel_div_rng_chk == 1 || in.main_sel_rng_16 == 1);
    }

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.alu_sel_div_rng_chk);
        const auto is_table_entry = View(in.main_sel_rng_16);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_div_u16_5,
                                     in.lookup_div_u16_5_counts,
                                     in.alu_sel_div_rng_chk,
                                     in.main_sel_rng_16,
                                     in.alu_div_u16_r5,
                                     in.main_clk);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_div_u16_5,
                                     in.lookup_div_u16_5_counts,
                                     in.alu_sel_div_rng_chk,
                                     in.main_sel_rng_16,
                                     in.alu_div_u16_r5,
                                     in.main_clk);
    }
};

template <typename FF_>
class lookup_div_u16_5_relation : public GenericLookupRelation<lookup_div_u16_5_lookup_settings, FF_> {
  public:
    static constexpr const char* NAME = "lookup_div_u16_5";
};
template <typename FF_> using lookup_div_u16_5 = GenericLookup<lookup_div_u16_5_lookup_settings, FF_>;

} // namespace bb