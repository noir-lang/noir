#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class GoblinTranslatorPermutationRelationImpl {
  public:
    using FF = FF_;
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 7;

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        7, // grand product construction sub-relation
        3  // left-shiftable polynomial sub-relation
    };

    inline static auto& get_grand_product_polynomial(auto& in) { return in.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& in) { return in.z_perm_shift; }

    template <typename Accumulator, typename AllEntities, typename Parameters>
    inline static Accumulator compute_grand_product_numerator(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        auto concatenated_range_constraints_0 = View(in.concatenated_range_constraints_0);
        auto concatenated_range_constraints_1 = View(in.concatenated_range_constraints_1);
        auto concatenated_range_constraints_2 = View(in.concatenated_range_constraints_2);
        auto concatenated_range_constraints_3 = View(in.concatenated_range_constraints_3);

        auto ordered_extra_range_constraints_numerator = View(in.ordered_extra_range_constraints_numerator);

        const auto& gamma = params.gamma;
        return (concatenated_range_constraints_0 + gamma) * (concatenated_range_constraints_1 + gamma) *
               (concatenated_range_constraints_2 + gamma) * (concatenated_range_constraints_3 + gamma) *
               (ordered_extra_range_constraints_numerator + gamma);
    }

    template <typename Accumulator, typename AllEntities, typename Parameters>
    inline static Accumulator compute_grand_product_denominator(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        auto ordered_range_constraints_0 = View(in.ordered_range_constraints_0);
        auto ordered_range_constraints_1 = View(in.ordered_range_constraints_1);
        auto ordered_range_constraints_2 = View(in.ordered_range_constraints_2);
        auto ordered_range_constraints_3 = View(in.ordered_range_constraints_3);
        auto ordered_range_constraints_4 = View(in.ordered_range_constraints_4);

        const auto& gamma = params.gamma;

        return (ordered_range_constraints_0 + gamma) * (ordered_range_constraints_1 + gamma) *
               (ordered_range_constraints_2 + gamma) * (ordered_range_constraints_3 + gamma) *
               (ordered_range_constraints_4 + gamma);
    }
    /**
     * @brief Compute contribution of the goblin translator permutation relation for a given edge (internal function)
     *
     * @details There are 2 relations associated with enforcing the set permutation relation
     * This file handles the relation that confirms faithful calculation of the grand
     * product polynomial Z_perm.
     *
     *  C(in(X)...) =
     *      ( z_perm(X) + lagrange_first(X) )*P(X)
     *         - ( z_perm_shift(X) + lagrange_last(X))*Q(X),
     * where P(X) = Prod_{i=0:4} numerator_polynomial_i(X) + γ
     *       Q(X) = Prod_{i=0:4} ordered_range_constraint_i(X) + γ
     * the first 4 numerator polynomials are concatenated range constraint polynomials and the last one is the constant
     * extra numerator
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */

    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulators,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor);
};

template <typename FF>
using GoblinTranslatorPermutationRelation = Relation<GoblinTranslatorPermutationRelationImpl<FF>>;

} // namespace bb
