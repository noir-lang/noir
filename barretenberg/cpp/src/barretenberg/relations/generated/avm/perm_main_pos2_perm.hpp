#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_pos2_perm_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 3;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_op_poseidon2 == 1 || in.poseidon2_sel_poseidon_perm == 1);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_pos2_perm,
                                     in.main_sel_op_poseidon2,
                                     in.main_sel_op_poseidon2,
                                     in.poseidon2_sel_poseidon_perm,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.poseidon2_clk,
                                     in.poseidon2_input,
                                     in.poseidon2_output);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.perm_main_pos2_perm,
                                     in.main_sel_op_poseidon2,
                                     in.main_sel_op_poseidon2,
                                     in.poseidon2_sel_poseidon_perm,
                                     in.main_clk,
                                     in.main_ia,
                                     in.main_ib,
                                     in.poseidon2_clk,
                                     in.poseidon2_input,
                                     in.poseidon2_output);
    }
};

template <typename FF_>
class perm_main_pos2_perm_relation : public GenericPermutationRelation<perm_main_pos2_perm_permutation_settings, FF_> {
  public:
    static constexpr const char* NAME = "PERM_MAIN_POS2_PERM";
};
template <typename FF_> using perm_main_pos2_perm = GenericPermutation<perm_main_pos2_perm_permutation_settings, FF_>;

} // namespace bb