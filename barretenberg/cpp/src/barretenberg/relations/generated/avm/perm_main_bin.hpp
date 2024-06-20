

#pragma once

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class perm_main_bin_permutation_settings {
  public:
    // This constant defines how many columns are bundled together to form each set.
    constexpr static size_t COLUMNS_PER_SET = 6;

    /**
     * @brief If this method returns true on a row of values, then the inverse polynomial at this index. Otherwise the
     * value needs to be set to zero.
     *
     * @details If this is true then permutation takes place in this row
     */

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.main_sel_bin == 1 || in.binary_start == 1);
    }

    /**
     * @brief Get all the entities for the permutation when we don't need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */

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

    /**
     * @brief Get all the entities for the permutation when need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */

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
using perm_main_bin_relation = GenericPermutationRelation<perm_main_bin_permutation_settings, FF_>;
template <typename FF_> using perm_main_bin = GenericPermutation<perm_main_bin_permutation_settings, FF_>;

} // namespace bb
