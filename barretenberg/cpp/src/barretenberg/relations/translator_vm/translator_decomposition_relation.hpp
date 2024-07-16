#pragma once
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class TranslatorDecompositionRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH =
        3; // degree(lagrange_odd_in_minicircuit_in_minicircuit(a - a_0 - a_1*2¹⁴ ... - a_l⋅2¹⁴ˡ )) = 2
    static constexpr std::array<size_t, 48> SUBRELATION_PARTIAL_LENGTHS{
        3, // decomposition of P.x limb 0 into microlimbs subrelation
        3, // decomposition of P.x limb 1 into microlimbs subrelation
        3, // decomposition of P.x limb 2 into microlimbs subrelation
        3, // decomposition of P.x limb 3 into microlimbs subrelation
        3, // decomposition of P.y limb 0 into microlimbs subrelation
        3, // decomposition of P.y limb 1 into microlimbs subrelation
        3, // decomposition of P.y limb 2 into microlimbs subrelation
        3, // decomposition of P.y limb 3 into microlimbs subrelation
        3, // decomposition of z1 limb 0 into microlimbs subrelation
        3, // decomposition of z2 limb 0 into microlimbs subrelation
        3, // decomposition of z1 limb 1 into microlimbs subrelation
        3, // decomposition of z2 limb 1 into microlimbs subrelation
        3, // decomposition of accumulator limb 0 into microlimbs subrelation
        3, // decomposition of accumulator limb 1 into microlimbs subrelation
        3, // decomposition of accumulator limb 2 into microlimbs subrelation
        3, // decomposition of accumulator limb 3 into microlimbs subrelation
        3, // decomposition of quotient limb 0 into microlimbs subrelation
        3, // decomposition of quotient limb 1 into microlimbs subrelation
        3, // decomposition of quotient limb 2 into microlimbs subrelation
        3, // decomposition of quotient limb 3 into microlimbs subrelation
        3, // decomposition of low relation wide limb into microlimbs subrelation
        3, // decomposition of high relation wide limb into microlimbs subrelation
        3, // stricter constraint on highest microlimb of P.x limb 0 subrelation
        3, // stricter constraint on highest microlimb of P.x limb 1 subrelation
        3, // stricter constraint on highest microlimb of P.x limb 2 subrelation
        3, // stricter constraint on highest microlimb of P.x limb 3 subrelation
        3, // stricter constraint on highest microlimb of P.y limb 0 subrelation
        3, // stricter constraint on highest microlimb of P.y limb 1 subrelation
        3, // stricter constraint on highest microlimb of P.y limb 2 subrelation
        3, // stricter constraint on highest microlimb of P.y limb 3 subrelation
        3, // stricter constraint on highest microlimb of z1 limb 0 subrelation
        3, // stricter constraint on highest microlimb of z2 limb 0 subrelation
        3, // stricter constraint on highest microlimb of z1 limb 1 subrelation
        3, // stricter constraint on highest microlimb of z2 limb 1 subrelation
        3, // stricter constraint on highest microlimb of accumulator limb 0 subrelation
        3, // stricter constraint on highest microlimb of accumulator limb 1 subrelation
        3, // stricter constraint on highest microlimb of accumulator limb 2 subrelation
        3, // stricter constraint on highest microlimb of accumulator limb 3 subrelation
        3, // stricter constraint on highest microlimb of quotient limb 0 subrelation
        3, // stricter constraint on highest microlimb of quotient limb 1 subrelation
        3, // stricter constraint on highest microlimb of quotient limb 2 subrelation
        3, // stricter constraint on highest microlimb of quotient limb 3 subrelation
        3, // decomposition of x_lo into 2 limbs subrelation
        3, // decomposition of x_hi into 2 limbs subrelation
        3, // decomposition of y_lo into 2 limbs subrelation
        3, // decomposition of y_hi into 2 limbs subrelation
        3, // decomposition of z1 into 2 limbs subrelation
        3  // decomposition of z2 into 2 limbs subrelation
    };
    /**
     * @brief For ZK-Flavors: Upper bound on the degrees of subrelations considered as polynomials only in witness
polynomials,
     * i.e. all selectors and public polynomials are treated as constants. The subrelation witness degree does not
     * exceed the subrelation partial degree given by SUBRELATION_PARTIAL_LENGTH - 1.
     */
    static constexpr std::array<size_t, 48> SUBRELATION_WITNESS_DEGREES{
        2, // decomposition of P.x limb 0 into microlimbs subrelation
        2, // decomposition of P.x limb 1 into microlimbs subrelation
        2, // decomposition of P.x limb 2 into microlimbs subrelation
        2, // decomposition of P.x limb 3 into microlimbs subrelation
        2, // decomposition of P.y limb 0 into microlimbs subrelation
        2, // decomposition of P.y limb 1 into microlimbs subrelation
        2, // decomposition of P.y limb 2 into microlimbs subrelation
        2, // decomposition of P.y limb 3 into microlimbs subrelation
        2, // decomposition of z1 limb 0 into microlimbs subrelation
        2, // decomposition of z2 limb 0 into microlimbs subrelation
        2, // decomposition of z1 limb 1 into microlimbs subrelation
        2, // decomposition of z2 limb 1 into microlimbs subrelation
        2, // decomposition of accumulator limb 0 into microlimbs subrelation
        2, // decomposition of accumulator limb 1 into microlimbs subrelation
        2, // decomposition of accumulator limb 2 into microlimbs subrelation
        2, // decomposition of accumulator limb 3 into microlimbs subrelation
        2, // decomposition of quotient limb 0 into microlimbs subrelation
        2, // decomposition of quotient limb 1 into microlimbs subrelation
        2, // decomposition of quotient limb 2 into microlimbs subrelation
        2, // decomposition of quotient limb 3 into microlimbs subrelation
        2, // decomposition of low relation wide limb into microlimbs subrelation
        2, // decomposition of high relation wide limb into microlimbs subrelation
        2, // stricter constraint on highest microlimb of P.x limb 0 subrelation
        2, // stricter constraint on highest microlimb of P.x limb 1 subrelation
        2, // stricter constraint on highest microlimb of P.x limb 2 subrelation
        2, // stricter constraint on highest microlimb of P.x limb 3 subrelation
        2, // stricter constraint on highest microlimb of P.y limb 0 subrelation
        2, // stricter constraint on highest microlimb of P.y limb 1 subrelation
        2, // stricter constraint on highest microlimb of P.y limb 2 subrelation
        2, // stricter constraint on highest microlimb of P.y limb 3 subrelation
        2, // stricter constraint on highest microlimb of z1 limb 0 subrelation
        2, // stricter constraint on highest microlimb of z2 limb 0 subrelation
        2, // stricter constraint on highest microlimb of z1 limb 1 subrelation
        2, // stricter constraint on highest microlimb of z2 limb 1 subrelation
        2, // stricter constraint on highest microlimb of accumulator limb 0 subrelation
        2, // stricter constraint on highest microlimb of accumulator limb 1 subrelation
        2, // stricter constraint on highest microlimb of accumulator limb 2 subrelation
        2, // stricter constraint on highest microlimb of accumulator limb 3 subrelation
        2, // stricter constraint on highest microlimb of quotient limb 0 subrelation
        2, // stricter constraint on highest microlimb of quotient limb 1 subrelation
        2, // stricter constraint on highest microlimb of quotient limb 2 subrelation
        2, // stricter constraint on highest microlimb of quotient limb 3 subrelation
        2, // decomposition of x_lo into 2 limbs subrelation
        2, // decomposition of x_hi into 2 limbs subrelation
        2, // decomposition of y_lo into 2 limbs subrelation
        2, // decomposition of y_hi into 2 limbs subrelation
        2, // decomposition of z1 into 2 limbs subrelation
        2  // decomposition of z2 into 2 limbs subrelation
    };

    /**
     * @brief Returns true if the contribution from all subrelations for the provided inputs is identically zero
     *
     */
    template <typename AllEntities> inline static bool skip(const AllEntities& in)
    {
        return in.lagrange_odd_in_minicircuit.is_zero();
    }

    /**
     * @brief Expression for decomposition of various values into smaller limbs or microlimbs.
     * @details This relation enforces three types of subrelations:
     * 1) A subrelation decomposing a value from the transcript (for example, z1) into 68-bit limbs. These relations
     * will have the structure `lagrange_odd_in_minicircuit⋅(a - a_low - a_high⋅2⁶⁸)`
     * 2) A subrelation decomposing a value  of one of the limbs used in bigfield computation (for example, the lower
     * wide relation limb) into 14-bit limbs. These relations will have the structure `lagrange_odd_in_minicircuit⋅(a -
     * a_0 - a_1⋅2¹⁴ -
     * ....)` 3) A subrelation making a microlimb range constraint more constraining. For example, we want to constrain
     * some values to 12 bits instead of 14. So we add a constraint `lagrange_odd_in_minicircuit⋅(a_highest⋅4 -
     * a_tail)`. In a separate relation both a_highest and a_tail are constrained to be 14 bits, but this relation
     * changes the constraint on a_highest to be 12 bits.
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

template <typename FF> using TranslatorDecompositionRelation = Relation<TranslatorDecompositionRelationImpl<FF>>;

} // namespace bb
