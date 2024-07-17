#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_mem_a_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 9;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_mem_op_a == 1 || in.mem_sel_op_a == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_mem_a,
                                     in.main_sel_mem_op_a,
                                     in.main_sel_mem_op_a,
                                     in.mem_sel_op_a,
                                     in.main_clk,
                                     in.main_space_id,
                                     in.main_mem_addr_a,
                                     in.main_ia,
                                     in.main_rwa,
                                     in.main_r_in_tag,
                                     in.main_w_in_tag,
                                     in.main_sel_mov_ia_to_ic,
                                     in.main_sel_op_cmov,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val,
                                     in.mem_rw,
                                     in.mem_r_in_tag,
                                     in.mem_w_in_tag,
                                     in.mem_sel_mov_ia_to_ic,
                                     in.mem_sel_op_cmov);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_mem_a,
                                     in.main_sel_mem_op_a,
                                     in.main_sel_mem_op_a,
                                     in.mem_sel_op_a,
                                     in.main_clk,
                                     in.main_space_id,
                                     in.main_mem_addr_a,
                                     in.main_ia,
                                     in.main_rwa,
                                     in.main_r_in_tag,
                                     in.main_w_in_tag,
                                     in.main_sel_mov_ia_to_ic,
                                     in.main_sel_op_cmov,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val,
                                     in.mem_rw,
                                     in.mem_r_in_tag,
                                     in.mem_w_in_tag,
                                     in.mem_sel_mov_ia_to_ic,
                                     in.mem_sel_op_cmov);
    }
};

template <typename FF_>
class perm_main_mem_a_relation : public GenericPermutationRelation<perm_main_mem_a_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "PERM_MAIN_MEM_A";
};
template <typename FF_> using perm_main_mem_a = GenericPermutation<perm_main_mem_a_permutation_settings, FF_>;

} // namespace bb