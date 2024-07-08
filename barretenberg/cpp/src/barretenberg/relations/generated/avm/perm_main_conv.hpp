#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_conv_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 4;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_op_radix_le == 1 || in.conversion_sel_to_radix_le == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_conv,
                                     in.main_sel_op_radix_le,
                                     in.main_sel_op_radix_le,
                                     in.conversion_sel_to_radix_le,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ic,
                                     in.main_id,
                                     in.conversion_clk,
                                     in.conversion_input,
                                     in.conversion_radix,
                                     in.conversion_num_limbs);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_conv,
                                     in.main_sel_op_radix_le,
                                     in.main_sel_op_radix_le,
                                     in.conversion_sel_to_radix_le,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ic,
                                     in.main_id,
                                     in.conversion_clk,
                                     in.conversion_input,
                                     in.conversion_radix,
                                     in.conversion_num_limbs);
    }
};

template <typename FF_>
class perm_main_conv_relation : public GenericPermutationRelation<perm_main_conv_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_main_conv";
};
template <typename FF_> using perm_main_conv = GenericPermutation<perm_main_conv_permutation_settings, FF_>;

} // namespace bb