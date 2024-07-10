#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_cd_mem_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 4;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.slice_sel_cd_cpy == 1 || in.mem_sel_op_cd_cpy == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_cd_mem,
                                     in.slice_sel_cd_cpy,
                                     in.slice_sel_cd_cpy,
                                     in.mem_sel_op_cd_cpy,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_addr,
                                     in.slice_val,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_cd_mem,
                                     in.slice_sel_cd_cpy,
                                     in.slice_sel_cd_cpy,
                                     in.mem_sel_op_cd_cpy,
                                     in.slice_clk,
                                     in.slice_space_id,
                                     in.slice_addr,
                                     in.slice_val,
                                     in.mem_clk,
                                     in.mem_space_id,
                                     in.mem_addr,
                                     in.mem_val);
    }
};

template <typename FF_>
class perm_cd_mem_relation : public GenericPermutationRelation<perm_cd_mem_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_cd_mem";
};
template <typename FF_> using perm_cd_mem = GenericPermutation<perm_cd_mem_permutation_settings, FF_>;

} // namespace bb