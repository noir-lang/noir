#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class TranslatorDeltaRangeConstraintRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // degree((lagrange_last-1) * D(D - 1)(D - 2)(D - 3)) = 5

    static constexpr std::array<size_t, 10> SUBRELATION_PARTIAL_LENGTHS{
        6, // ordered_range_constraints_0 step in {0,1,2,3} subrelation
        6, // ordered_range_constraints_1 step in {0,1,2,3} subrelation
        6, // ordered_range_constraints_2 step in {0,1,2,3} subrelation
        6, // ordered_range_constraints_3 step in {0,1,2,3} subrelation
        6, // ordered_range_constraints_4 step in {0,1,2,3} subrelation
        3, // ordered_range_constraints_0 ends with defined maximum value subrelation
        3, // ordered_range_constraints_1 ends with defined maximum value subrelation
        3, // ordered_range_constraints_2 ends with defined maximum value subrelation
        3, // ordered_range_constraints_3 ends with defined maximum value subrelation
        3  // ordered_range_constraints_4 ends with defined maximum value subrelation

    };
    /**
     * @brief For ZK-Flavors: Upper bound on the degrees of subrelations considered as polynomials only in witness
polynomials,
     * i.e. all selectors and public polynomials are treated as constants. The subrelation witness degree does not
     * exceed the subrelation partial degree given by SUBRELATION_PARTIAL_LENGTH - 1.
     */
    static constexpr std::array<size_t, 10> SUBRELATION_WITNESS_DEGREES{
        5, // ordered_range_constraints_0 step in {0,1,2,3} subrelation
        5, // ordered_range_constraints_1 step in {0,1,2,3} subrelation
        5, // ordered_range_constraints_2 step in {0,1,2,3} subrelation
        5, // ordered_range_constraints_3 step in {0,1,2,3} subrelation
        5, // ordered_range_constraints_4 step in {0,1,2,3} subrelation
        2, // ordered_range_constraints_0 ends with defined maximum value subrelation
        2, // ordered_range_constraints_1 ends with defined maximum value subrelation
        2, // ordered_range_constraints_2 ends with defined maximum value subrelation
        2, // ordered_range_constraints_3 ends with defined maximum value subrelation
        2  // ordered_range_constraints_4 ends with defined maximum value subrelation

    };
    /**
     * @brief Expression for the generalized permutation sort relation
     *
     * @details The relation enforces 2 constraints on each of the ordered_range_constraints wires:
     * 1) 2 sequential values are non-descending and have a difference of at most 3, except for the value at last index
     * 2) The value at last index is  2¹⁴ - 1
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulators,
                           const AllEntities& in,
                           const Parameters&,
                           const FF& scaling_factor);
};

template <typename FF>
using TranslatorDeltaRangeConstraintRelation = Relation<TranslatorDeltaRangeConstraintRelationImpl<FF>>;

} // namespace bb
