#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class DeltaRangeConstraintRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 4> SUBRELATION_PARTIAL_LENGTHS{
        6, // range constrain sub-relation 1
        6, // range constrain sub-relation 2
        6, // range constrain sub-relation 3
        6  // range constrain sub-relation 4
    };
    /**
     * @brief For ZK-Flavors: The degrees of subrelations considered as polynomials only in witness polynomials,
     * i.e. all selectors and public polynomials are treated as constants.
     *
     */
    static constexpr std::array<size_t, 4> SUBRELATION_WITNESS_DEGREES{
        3, // range constrain sub-relation 1
        3, // range constrain sub-relation 2
        3, // range constrain sub-relation 3
        3  // range constrain sub-relation 4
    };

    /**
     * @brief Returns true if the contribution from all subrelations for the provided inputs is identically zero
     *
     */
    template <typename AllEntities> inline static bool skip(const AllEntities& in)
    {
        return in.q_delta_range.is_zero();
    }

    /**
     * @brief Expression for the generalized permutation sort gate.
     * @details The relation is defined as C(in(X)...) =
     *    q_delta_range * \sum{ i = [0, 3]} \alpha^i D_i(D_i - 1)(D_i - 2)(D_i - 3)
     *      where
     *      D_0 = w_2 - w_1
     *      D_1 = w_3 - w_2
     *      D_2 = w_4 - w_3
     *      D_3 = w_1_shift - w_4
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    inline static void accumulate(ContainerOverSubrelations& accumulators,
                                  const AllEntities& in,
                                  const Parameters&,
                                  const FF& scaling_factor)
    {
        BB_OP_COUNT_TIME_NAME("DeltaRange::accumulate");
        using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
        using View = typename Accumulator::View;
        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);
        auto w_4 = View(in.w_4);
        auto w_1_shift = View(in.w_l_shift);
        auto q_delta_range = View(in.q_delta_range);

        static const FF minus_one = FF(-1);
        static const FF minus_two = FF(-2);

        // Compute wire differences
        auto delta_1 = w_2 - w_1;
        auto delta_2 = w_3 - w_2;
        auto delta_3 = w_4 - w_3;
        auto delta_4 = w_1_shift - w_4;

        // Contribution (1)
        auto tmp_1 = (delta_1 + minus_one).sqr() + minus_one;
        tmp_1 *= (delta_1 + minus_two).sqr() + minus_one;
        tmp_1 *= q_delta_range;
        tmp_1 *= scaling_factor;
        std::get<0>(accumulators) += tmp_1;

        // Contribution (2)
        auto tmp_2 = (delta_2 + minus_one).sqr() + minus_one;
        tmp_2 *= (delta_2 + minus_two).sqr() + minus_one;
        tmp_2 *= q_delta_range;
        tmp_2 *= scaling_factor;
        std::get<1>(accumulators) += tmp_2;

        // Contribution (3)
        auto tmp_3 = (delta_3 + minus_one).sqr() + minus_one;
        tmp_3 *= (delta_3 + minus_two).sqr() + minus_one;
        tmp_3 *= q_delta_range;
        tmp_3 *= scaling_factor;
        std::get<2>(accumulators) += tmp_3;

        // Contribution (4)
        auto tmp_4 = (delta_4 + minus_one).sqr() + minus_one;
        tmp_4 *= (delta_4 + minus_two).sqr() + minus_one;
        tmp_4 *= q_delta_range;
        tmp_4 *= scaling_factor;
        std::get<3>(accumulators) += tmp_4;
    };
};

template <typename FF> using DeltaRangeConstraintRelation = Relation<DeltaRangeConstraintRelationImpl<FF>>;

} // namespace bb