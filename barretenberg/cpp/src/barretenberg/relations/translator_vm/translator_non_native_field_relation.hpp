#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class GoblinTranslatorNonNativeFieldRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr std::array<size_t, 3> SUBRELATION_PARTIAL_LENGTHS{
        3, // Lower wide limb subrelation (checks result is 0 mod 2¹³⁶)
        3, // Higher wide limb subrelation (checks result is 0 in higher mod 2¹³⁶),
        3  // Prime subrelation (checks result in native field)
    };

    /**
     * @brief Expression for the computation of Goblin Translator accumulator in integers through 68-bit limbs and
     * native field (prime) limb
     * @details This relation is a part of system of relations that enforce a formula in non-native field (base field of
     * bn254 curve Fp (p - modulus of Fp)). We are trying to compute:
     *
     * `current_accumulator = previous_accumulator ⋅ x + op + P.x ⋅ v + P.y ⋅ v² +z1 ⋅ v³ + z2 ⋅ v⁴ mod p`.
     *
     * However, we can only operate in Fr (scalar field of bn254) with
     * modulus r. To emulate arithmetic in Fp we rephrase the equation in integers:
     *
     * `previous_accumulator ⋅ x + op + P.x ⋅ v + P.y ⋅ v² +z1 ⋅ v³ + z2 ⋅ v⁴ - quotient⋅p - current_accumulator = 0`
     *
     * We can't operate over unbounded integers, but since we know the maximum value of each element (we also treat
     * powers of v as new constants constrained to 254 bits) we know that the maximum values of the sum of the positive
     * products is ~2⁵¹⁴, so we only need to make sure that no overflow happens till that bound. We calculate integer
     * logic until the bound 2²⁷²⋅r (which is more than 2⁵¹⁴) by using the representations modulo 2²⁷² (requires limb
     * computation over native scalar field) and r (native scalar field computation).
     *
     * We perform modulo 2²⁷² computations by separating each of values into 4 68-bit limbs (z1 and z2 are just two
     * since they represent the values < 2¹²⁸ and op is just itself). Then we compute the first subrelation (index means
     * sublimb and we use 2²⁷² - p instead of -p):
     * `      previous_accumulator[0]⋅x[0] + op + P.x[0]⋅v[0] + P.y[0]⋅v²[0] + z1[0] ⋅ v³[0] + z2[0] ⋅ v⁴[0]
     *          + quotient[0]⋅(-p)[0] - current_accumulator[0]
     * + 2⁶⁸⋅(previous_accumulator[1]⋅x[0] +      P.x[1]⋅v[0] + P.y[1]⋅v²[0] + z1[1] ⋅ v³[0] + z2[1] ⋅ v⁴[0]
     *          + quotient[1]⋅(-p)[0] +
     *        previous_accumulator[0]⋅x[1] +      P.x[0]⋅v[1] + P.y[0]⋅v²[1] + z1[0] ⋅ v³[1] + z2[0] ⋅ v⁴[1]
     *          + quotient[0]⋅(-p)[1] - current_accumulator[1])
     *  - 2¹³⁶⋅relation_wide_lower_limb
     *  == 0`
     *
     * We use 2 relation wide limbs which are called wide, because they contain the results of products (like you needed
     * EDX:EAX in x86 to hold the product results of two standard 32-bit registers) and because they are constrained to
     * 84 bits instead of 68 or lower by other relations.
     *
     * We show that the evaluation in 2 lower limbs results in relation_wide_lower_limb multiplied by 2¹³⁶. If
     * relation_wide_lower_limb is propertly constrained (this is performed in other relations), then that means that
     * the lower 136 bits of the result are 0. This is the first subrelation.
     *
     * We then use the relation_wide_lower_limb as carry and add it to the next expression, computing the evaluation in
     * higher bits (carry + combinations of limbs (0,2), (1,1), (2,0), (0,3), (2,1), (1,2), (0,3)) and checking that it
     * results in 2¹³⁶⋅relation_wide_higher_limb. This ensures that the logic was sound modulo 2²⁷². This is the second
     * subrelation.
     *
     * Finally, we check that the relation holds in the native field. For this we reconstruct each value, for example:
     * `previous_accumulator_native =        previous_accumulator[0] + 2⁶⁸ ⋅previous_accumulator[1]
     *                                + 2¹³⁶⋅previous_accumulator[2] + 2²⁰⁴⋅previous accumulator[3] mod r`
     *
     * Then the last subrelation is simply checking the integer equation in this native form
     *
     * All of these subrelations are multiplied by lagrange_odd_in_minicircuit, which is a polynomial with 1 at each odd
     * index less than the size of the mini-circuit (16 times smaller than the final circuit and the only part over
     * which we need to calculate non-permutation relations). All other indices are set to zero. Each EccOpQueue entry
     * (operation) occupies 2 rows in bn254 transcripts. So the Goblin Translator VM has a 2-row cycle and we need to
     * switch the checks being performed depending on which row we are at right now. We have half a cycle of
     * accumulation, where we perform this computation, and half a cycle where we just copy accumulator data.
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
using GoblinTranslatorNonNativeFieldRelation = Relation<GoblinTranslatorNonNativeFieldRelationImpl<FF>>;

} // namespace bb
