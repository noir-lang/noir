#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_slice_mem_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 5;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.slice_sel_mem_active == 1 || in.mem_sel_op_slice == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_slice_mem,
                                     in.slice_sel_mem_active,
                                     in.slice_sel_mem_active,
                                     in.mem_sel_op_slice,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_addr,
                                     in.slice_val,
                                     in.slice_sel_cd_cpy,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val,
                                     in.mem_rw);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_slice_mem,
                                     in.slice_sel_mem_active,
                                     in.slice_sel_mem_active,
                                     in.mem_sel_op_slice,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_addr,
                                     in.slice_val,
                                     in.slice_sel_cd_cpy,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val,
                                     in.mem_rw);
    }
};

template <typename FF_>
class perm_slice_mem_relation : public GenericPermutationRelation<perm_slice_mem_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_slice_mem";
};
template <typename FF_> using perm_slice_mem = GenericPermutation<perm_slice_mem_permutation_settings, FF_>;

} // namespace bb