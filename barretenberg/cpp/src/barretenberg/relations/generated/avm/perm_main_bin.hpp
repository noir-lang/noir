#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_bin_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 6;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_bin == 1 || in.binary_start == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_bin,
                                     in.main_sel_bin,
                                     in.main_sel_bin,
                                     in.binary_start,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_ic,
                                     in.main_bin_op_id,
                                     in.main_r_in_tag,
                                     in.binary_clk,
                                     in.binary_acc_ia,
                                     in.binary_acc_ib,
                                     in.binary_acc_ic,
                                     in.binary_op_id,
                                     in.binary_in_tag);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_bin,
                                     in.main_sel_bin,
                                     in.main_sel_bin,
                                     in.binary_start,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.main_ic,
                                     in.main_bin_op_id,
                                     in.main_r_in_tag,
                                     in.binary_clk,
                                     in.binary_acc_ia,
                                     in.binary_acc_ib,
                                     in.binary_acc_ic,
                                     in.binary_op_id,
                                     in.binary_in_tag);
    }
};

template <typename FF_>
class perm_main_bin_relation : public GenericPermutationRelation<perm_main_bin_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_main_bin";
};
template <typename FF_> using perm_main_bin = GenericPermutation<perm_main_bin_permutation_settings, FF_>;

} // namespace bb