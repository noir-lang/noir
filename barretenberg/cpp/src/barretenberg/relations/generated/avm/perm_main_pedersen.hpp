#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_pedersen_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 2;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_op_pedersen == 1 || in.pedersen_sel_pedersen == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_pedersen,
                                     in.main_sel_op_pedersen,
                                     in.main_sel_op_pedersen,
                                     in.pedersen_sel_pedersen,
                                     in.main_clk,
                                     in.main_ia,
                                     in.pedersen_clk,
                                     in.pedersen_input);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_pedersen,
                                     in.main_sel_op_pedersen,
                                     in.main_sel_op_pedersen,
                                     in.pedersen_sel_pedersen,
                                     in.main_clk,
                                     in.main_ia,
                                     in.pedersen_clk,
                                     in.pedersen_input);
    }
};

template <typename FF_>
class perm_main_pedersen_relation : public GenericPermutationRelation<perm_main_pedersen_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "perm_main_pedersen";
};
template <typename FF_> using perm_main_pedersen = GenericPermutation<perm_main_pedersen_permutation_settings, FF_>;

} // namespace bb