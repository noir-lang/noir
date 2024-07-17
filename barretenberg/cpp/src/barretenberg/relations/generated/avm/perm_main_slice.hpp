#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_slice_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 7;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_slice_gadget == 1 || in.slice_sel_start == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_slice,
                                     in.main_sel_slice_gadget,
                                     in.main_sel_slice_gadget,
                                     in.slice_sel_start,
                                     in.main_clk,
                                     in.main_space_id,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_mem_addr_c,
                                     in.main_sel_op_calldata_copy,
                                     in.main_sel_op_external_return,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_col_offset,
                                     in.slice_cnt,
                                     in.slice_addr,
                                     in.slice_sel_cd_cpy,
                                     in.slice_sel_return);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_slice,
                                     in.main_sel_slice_gadget,
                                     in.main_sel_slice_gadget,
                                     in.slice_sel_start,
                                     in.main_clk,
                                     in.main_space_id,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_mem_addr_c,
                                     in.main_sel_op_calldata_copy,
                                     in.main_sel_op_external_return,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_col_offset,
                                     in.slice_cnt,
                                     in.slice_addr,
                                     in.slice_sel_cd_cpy,
                                     in.slice_sel_return);
    }
};

template <typename FF_>
class perm_main_slice_relation : public GenericPermutationRelation<perm_main_slice_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "PERM_MAIN_SLICE";
};
template <typename FF_> using perm_main_slice = GenericPermutation<perm_main_slice_permutation_settings, FF_>;

} // namespace bb