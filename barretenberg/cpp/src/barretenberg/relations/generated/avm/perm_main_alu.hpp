#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_alu_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 16;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_alu == 1 || in.alu_sel_alu == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_alu,
                                     in.main_sel_alu,
                                     in.main_sel_alu,
                                     in.alu_sel_alu,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_ic,
                                     in.main_sel_op_add,
                                     in.main_sel_op_sub,
                                     in.main_sel_op_mul,
                                     in.main_sel_op_div,
                                     in.main_sel_op_eq,
                                     in.main_sel_op_not,
                                     in.main_sel_op_cast,
                                     in.main_sel_op_lt,
                                     in.main_sel_op_lte,
                                     in.main_sel_op_shr,
                                     in.main_sel_op_shl,
                                     in.main_alu_in_tag,
                                     in.alu_clk,
                                     in.alu_ia,
                                     in.alu_ib,
                                     in.alu_ic,
                                     in.alu_op_add,
                                     in.alu_op_sub,
                                     in.alu_op_mul,
                                     in.alu_op_div,
                                     in.alu_op_eq,
                                     in.alu_op_not,
                                     in.alu_op_cast,
                                     in.alu_op_lt,
                                     in.alu_op_lte,
                                     in.alu_op_shr,
                                     in.alu_op_shl,
                                     in.alu_in_tag);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_alu,
                                     in.main_sel_alu,
                                     in.main_sel_alu,
                                     in.alu_sel_alu,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_ic,
                                     in.main_sel_op_add,
                                     in.main_sel_op_sub,
                                     in.main_sel_op_mul,
                                     in.main_sel_op_div,
                                     in.main_sel_op_eq,
                                     in.main_sel_op_not,
                                     in.main_sel_op_cast,
                                     in.main_sel_op_lt,
                                     in.main_sel_op_lte,
                                     in.main_sel_op_shr,
                                     in.main_sel_op_shl,
                                     in.main_alu_in_tag,
                                     in.alu_clk,
                                     in.alu_ia,
                                     in.alu_ib,
                                     in.alu_ic,
                                     in.alu_op_add,
                                     in.alu_op_sub,
                                     in.alu_op_mul,
                                     in.alu_op_div,
                                     in.alu_op_eq,
                                     in.alu_op_not,
                                     in.alu_op_cast,
                                     in.alu_op_lt,
                                     in.alu_op_lte,
                                     in.alu_op_shr,
                                     in.alu_op_shl,
                                     in.alu_in_tag);
    }
};

template <typename FF_>
class perm_main_alu_relation : public GenericPermutationRelation<perm_main_alu_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_main_alu";
};
template <typename FF_> using perm_main_alu = GenericPermutation<perm_main_alu_permutation_settings, FF_>;

} // namespace bb