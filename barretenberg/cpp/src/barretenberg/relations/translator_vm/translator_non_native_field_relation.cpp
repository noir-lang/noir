#include "barretenberg/relations/translator_vm/translator_non_native_field_relation.hpp"
#include "barretenberg/flavor/goblin_translator.hpp"

namespace bb {
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
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void GoblinTranslatorNonNativeFieldRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulators,
                                                                const AllEntities& in,
                                                                const Parameters& params,
                                                                const FF& scaling_factor)
{

    using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    static constexpr size_t NUM_LIMB_BITS = 68;
    static constexpr FF shift = FF(uint256_t(1) << NUM_LIMB_BITS);
    static constexpr FF shiftx2 = FF(uint256_t(1) << (NUM_LIMB_BITS * 2));
    static constexpr FF shiftx3 = FF(uint256_t(1) << (NUM_LIMB_BITS * 3));
    static constexpr uint512_t MODULUS_U512 = uint512_t(curve::BN254::BaseField::modulus);
    static constexpr uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (NUM_LIMB_BITS << 2);
    static constexpr uint512_t NEGATIVE_PRIME_MODULUS = BINARY_BASIS_MODULUS - MODULUS_U512;
    static constexpr std::array<FF, 5> NEGATIVE_MODULUS_LIMBS = {
        FF(NEGATIVE_PRIME_MODULUS.slice(0, NUM_LIMB_BITS).lo),
        FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
        FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
        FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
        -FF(curve::BN254::BaseField::modulus)
    };

    const auto& evaluation_input_x_0 = params.evaluation_input_x[0];
    const auto& evaluation_input_x_1 = params.evaluation_input_x[1];
    const auto& evaluation_input_x_2 = params.evaluation_input_x[2];
    const auto& evaluation_input_x_3 = params.evaluation_input_x[3];
    const auto& evaluation_input_x_4 = params.evaluation_input_x[4];
    // for j < 4,  v_i_j is the j-th limb of v^{1+i}
    // v_i_4 is v^{1+i} in the native field
    const auto& v_0_0 = params.batching_challenge_v[0][0];
    const auto& v_0_1 = params.batching_challenge_v[0][1];
    const auto& v_0_2 = params.batching_challenge_v[0][2];
    const auto& v_0_3 = params.batching_challenge_v[0][3];
    const auto& v_0_4 = params.batching_challenge_v[0][4];
    const auto& v_1_0 = params.batching_challenge_v[1][0];
    const auto& v_1_1 = params.batching_challenge_v[1][1];
    const auto& v_1_2 = params.batching_challenge_v[1][2];
    const auto& v_1_3 = params.batching_challenge_v[1][3];
    const auto& v_1_4 = params.batching_challenge_v[1][4];
    const auto& v_2_0 = params.batching_challenge_v[2][0];
    const auto& v_2_1 = params.batching_challenge_v[2][1];
    const auto& v_2_2 = params.batching_challenge_v[2][2];
    const auto& v_2_3 = params.batching_challenge_v[2][3];
    const auto& v_2_4 = params.batching_challenge_v[2][4];
    const auto& v_3_0 = params.batching_challenge_v[3][0];
    const auto& v_3_1 = params.batching_challenge_v[3][1];
    const auto& v_3_2 = params.batching_challenge_v[3][2];
    const auto& v_3_3 = params.batching_challenge_v[3][3];
    const auto& v_3_4 = params.batching_challenge_v[3][4];

    const auto& op = View(in.op);
    const auto& p_x_low_limbs = View(in.p_x_low_limbs);
    const auto& p_y_low_limbs = View(in.p_y_low_limbs);
    const auto& p_x_high_limbs = View(in.p_x_high_limbs);
    const auto& p_y_high_limbs = View(in.p_y_high_limbs);
    const auto& accumulators_binary_limbs_0 = View(in.accumulators_binary_limbs_0);
    const auto& accumulators_binary_limbs_1 = View(in.accumulators_binary_limbs_1);
    const auto& accumulators_binary_limbs_2 = View(in.accumulators_binary_limbs_2);
    const auto& accumulators_binary_limbs_3 = View(in.accumulators_binary_limbs_3);
    const auto& z_low_limbs = View(in.z_low_limbs);
    const auto& z_high_limbs = View(in.z_high_limbs);
    const auto& quotient_low_binary_limbs = View(in.quotient_low_binary_limbs);
    const auto& quotient_high_binary_limbs = View(in.quotient_high_binary_limbs);
    const auto& p_x_low_limbs_shift = View(in.p_x_low_limbs_shift);
    const auto& p_y_low_limbs_shift = View(in.p_y_low_limbs_shift);
    const auto& p_x_high_limbs_shift = View(in.p_x_high_limbs_shift);
    const auto& p_y_high_limbs_shift = View(in.p_y_high_limbs_shift);
    const auto& accumulators_binary_limbs_0_shift = View(in.accumulators_binary_limbs_0_shift);
    const auto& accumulators_binary_limbs_1_shift = View(in.accumulators_binary_limbs_1_shift);
    const auto& accumulators_binary_limbs_2_shift = View(in.accumulators_binary_limbs_2_shift);
    const auto& accumulators_binary_limbs_3_shift = View(in.accumulators_binary_limbs_3_shift);
    const auto& z_low_limbs_shift = View(in.z_low_limbs_shift);
    const auto& z_high_limbs_shift = View(in.z_high_limbs_shift);
    const auto& quotient_low_binary_limbs_shift = View(in.quotient_low_binary_limbs_shift);
    const auto& quotient_high_binary_limbs_shift = View(in.quotient_high_binary_limbs_shift);
    const auto& relation_wide_limbs = View(in.relation_wide_limbs);
    const auto& relation_wide_limbs_shift = View(in.relation_wide_limbs_shift);
    const auto& lagrange_odd_in_minicircuit = View(in.lagrange_odd_in_minicircuit);

    // Contribution (1) Computing the mod 2²⁷² relation over lower 136 bits
    // clang-format off
        // the index-0 limb
        auto tmp = accumulators_binary_limbs_0_shift * evaluation_input_x_0 
                   + op 
                   + p_x_low_limbs     * v_0_0 
                   + p_y_low_limbs     * v_1_0 
                   + z_low_limbs       * v_2_0 
                   + z_low_limbs_shift * v_3_0
                   + quotient_low_binary_limbs * NEGATIVE_MODULUS_LIMBS[0] 
                   - accumulators_binary_limbs_0;
        
        // the index-1 limb
        tmp += (accumulators_binary_limbs_1_shift   * evaluation_input_x_0 
                   + accumulators_binary_limbs_0_shift * evaluation_input_x_1 
                   + p_x_low_limbs       * v_0_1
                   + p_x_low_limbs_shift * v_0_0
                   + p_y_low_limbs       * v_1_1
                   + p_y_low_limbs_shift * v_1_0
                   + z_low_limbs         * v_2_1
                   + z_high_limbs        * v_2_0
                   + z_low_limbs_shift   * v_3_1
                   + z_high_limbs_shift  * v_3_0
                   + quotient_low_binary_limbs       * NEGATIVE_MODULUS_LIMBS[1] 
                   + quotient_low_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[0] 
                   - accumulators_binary_limbs_1)
                * shift ;
    // clang-format on
    // subtract large value; vanishing shows the desired relation holds on low 136-bit limb
    tmp -= relation_wide_limbs * shiftx2;
    tmp *= lagrange_odd_in_minicircuit;
    tmp *= scaling_factor;
    std::get<0>(accumulators) += tmp;

    // Contribution (2) Computing the 2²⁷² relation over higher 136 bits
    // why declare another temporary?
    // clang-format off
        // the index-2 limb, with a carry from the previous calculation
        tmp = relation_wide_limbs
              + accumulators_binary_limbs_2_shift * evaluation_input_x_0
              + accumulators_binary_limbs_1_shift * evaluation_input_x_1
              + accumulators_binary_limbs_0_shift * evaluation_input_x_2
              + p_x_high_limbs      * v_0_0
              + p_x_low_limbs_shift * v_0_1
              + p_x_low_limbs       * v_0_2
              + p_y_high_limbs      * v_1_0
              + p_y_low_limbs_shift * v_1_1
              + p_y_low_limbs       * v_1_2
              + z_high_limbs        * v_2_1
              + z_low_limbs         * v_2_2
              + z_high_limbs_shift  * v_3_1
              + z_low_limbs_shift   * v_3_2
              + quotient_high_binary_limbs      * NEGATIVE_MODULUS_LIMBS[0]
              + quotient_low_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[1]
              + quotient_low_binary_limbs       * NEGATIVE_MODULUS_LIMBS[2] 
              - accumulators_binary_limbs_2;

        // the index-2 limb
        tmp += (accumulators_binary_limbs_3_shift   * evaluation_input_x_0
                   + accumulators_binary_limbs_2_shift   * evaluation_input_x_1
                   + accumulators_binary_limbs_1_shift * evaluation_input_x_2
                   + accumulators_binary_limbs_0_shift * evaluation_input_x_3
                   + p_x_high_limbs_shift * v_0_0
                   + p_x_high_limbs       * v_0_1
                   + p_x_low_limbs_shift  * v_0_2
                   + p_x_low_limbs        * v_0_3
                   + p_y_high_limbs_shift * v_1_0
                   + p_y_high_limbs       * v_1_1
                   + p_y_low_limbs_shift  * v_1_2
                   + p_y_low_limbs        * v_1_3
                   + z_high_limbs         * v_2_2
                   + z_low_limbs          * v_2_3
                   + z_high_limbs_shift   * v_3_2
                   + z_low_limbs_shift    * v_3_3
                   + quotient_high_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[0]
                   + quotient_high_binary_limbs       * NEGATIVE_MODULUS_LIMBS[1]
                   + quotient_low_binary_limbs_shift  * NEGATIVE_MODULUS_LIMBS[2]
                   + quotient_low_binary_limbs        * NEGATIVE_MODULUS_LIMBS[3] 
                   - accumulators_binary_limbs_3)
                * shift;
    // clang-format on
    // subtract large value; vanishing shows the desired relation holds on high 136-bit limb
    tmp -= relation_wide_limbs_shift * shiftx2;
    tmp *= lagrange_odd_in_minicircuit;
    tmp *= scaling_factor;
    std::get<1>(accumulators) += tmp;

    const auto reconstruct_from_two = [](const auto& l0, const auto& l1) { return l0 + l1 * shift; };

    const auto reconstruct_from_four = [](const auto& l0, const auto& l1, const auto& l2, const auto& l3) {
        return l0 + l1 * shift + l2 * shiftx2 + l3 * shiftx3;
    };

    // Reconstructing native versions of values
    auto reconstructed_p_x =
        reconstruct_from_four(p_x_low_limbs, p_x_low_limbs_shift, p_x_high_limbs, p_x_high_limbs_shift);
    auto reconstructed_p_y =
        reconstruct_from_four(p_y_low_limbs, p_y_low_limbs_shift, p_y_high_limbs, p_y_high_limbs_shift);
    auto reconstructed_previous_accumulator = reconstruct_from_four(accumulators_binary_limbs_0_shift,
                                                                    accumulators_binary_limbs_1_shift,
                                                                    accumulators_binary_limbs_2_shift,
                                                                    accumulators_binary_limbs_3_shift);
    auto reconstructed_current_accumulator = reconstruct_from_four(accumulators_binary_limbs_0,
                                                                   accumulators_binary_limbs_1,
                                                                   accumulators_binary_limbs_2,
                                                                   accumulators_binary_limbs_3);
    auto reconstructed_z1 = reconstruct_from_two(z_low_limbs, z_high_limbs);
    auto reconstructed_z2 = reconstruct_from_two(z_low_limbs_shift, z_high_limbs_shift);
    auto reconstructed_quotient = reconstruct_from_four(quotient_low_binary_limbs,
                                                        quotient_low_binary_limbs_shift,
                                                        quotient_high_binary_limbs,
                                                        quotient_high_binary_limbs_shift);

    // Contribution (3). Evaluating integer relation over native field
    // clang-format off
        // the native limb index is 4
        tmp = reconstructed_previous_accumulator * evaluation_input_x_4 
                     + op 
                     + reconstructed_p_x * v_0_4
                     + reconstructed_p_y * v_1_4
                     + reconstructed_z1  * v_2_4
                     + reconstructed_z2  * v_3_4
                     + reconstructed_quotient * NEGATIVE_MODULUS_LIMBS[4] 
                     - reconstructed_current_accumulator;
    // clang-format on
    tmp *= lagrange_odd_in_minicircuit;
    tmp *= scaling_factor;
    std::get<2>(accumulators) += tmp;
};

template class GoblinTranslatorNonNativeFieldRelationImpl<bb::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(GoblinTranslatorNonNativeFieldRelationImpl, honk::flavor::GoblinTranslator);

} // namespace bb
