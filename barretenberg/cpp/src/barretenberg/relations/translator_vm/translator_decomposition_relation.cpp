#include "barretenberg/relations/translator_vm/translator_decomposition_relation.hpp"
#include "barretenberg/flavor/goblin_translator.hpp"

namespace bb {

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
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void GoblinTranslatorDecompositionRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulators,
                                                               const AllEntities& in,
                                                               const Parameters&,
                                                               const FF& scaling_factor)
{
    static constexpr size_t NUM_LIMB_BITS = 68;       // Number of bits in a standard limb used for bigfield operations
    static constexpr size_t NUM_MICRO_LIMB_BITS = 14; // Number of bits in a standard limb used for bigfield operations

    // Value to multiply an element by to perform an appropriate shift
    static constexpr auto LIMB_SHIFT = FF(uint256_t(1) << NUM_LIMB_BITS);

    // Values to multiply an element by to perform an appropriate shift
    static constexpr auto MICRO_LIMB_SHIFT = FF(uint256_t(1) << NUM_MICRO_LIMB_BITS);
    static constexpr auto MICRO_LIMB_SHIFTx2 = MICRO_LIMB_SHIFT * MICRO_LIMB_SHIFT;
    static constexpr auto MICRO_LIMB_SHIFTx3 = MICRO_LIMB_SHIFTx2 * MICRO_LIMB_SHIFT;
    static constexpr auto MICRO_LIMB_SHIFTx4 = MICRO_LIMB_SHIFTx3 * MICRO_LIMB_SHIFT;
    static constexpr auto MICRO_LIMB_SHIFTx5 = MICRO_LIMB_SHIFTx4 * MICRO_LIMB_SHIFT;

    // Shifts used to constrain ranges further
    static constexpr auto SHIFT_12_TO_14 =
        FF(4); // Shift used to range constrain the last microlimb of 68-bit limbs (standard limbs)
    static constexpr auto SHIFT_10_TO_14 =
        FF(16); // Shift used to range constrain the last microlimb of 52-bit limb (top quotient limb)
    static constexpr auto SHIFT_8_TO_14 = FF(64); // Shift used to range constrain the last microlimb of 50-bit
                                                  // limbs (top limb of standard 254-bit value)
    static constexpr auto SHIFT_4_TO_14 =
        FF(1024); // Shift used to range constrain the last mircrolimb of 60-bit limbs from z scalars

    using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    auto p_x_low_limbs = View(in.p_x_low_limbs);
    auto p_x_low_limbs_range_constraint_0 = View(in.p_x_low_limbs_range_constraint_0);
    auto p_x_low_limbs_range_constraint_1 = View(in.p_x_low_limbs_range_constraint_1);
    auto p_x_low_limbs_range_constraint_2 = View(in.p_x_low_limbs_range_constraint_2);
    auto p_x_low_limbs_range_constraint_3 = View(in.p_x_low_limbs_range_constraint_3);
    auto p_x_low_limbs_range_constraint_4 = View(in.p_x_low_limbs_range_constraint_4);
    auto p_x_low_limbs_shift = View(in.p_x_low_limbs_shift);
    auto p_x_low_limbs_range_constraint_0_shift = View(in.p_x_low_limbs_range_constraint_0_shift);
    auto p_x_low_limbs_range_constraint_1_shift = View(in.p_x_low_limbs_range_constraint_1_shift);
    auto p_x_low_limbs_range_constraint_2_shift = View(in.p_x_low_limbs_range_constraint_2_shift);
    auto p_x_low_limbs_range_constraint_3_shift = View(in.p_x_low_limbs_range_constraint_3_shift);
    auto p_x_low_limbs_range_constraint_4_shift = View(in.p_x_low_limbs_range_constraint_4_shift);
    auto p_x_high_limbs = View(in.p_x_high_limbs);
    auto p_x_high_limbs_range_constraint_0 = View(in.p_x_high_limbs_range_constraint_0);
    auto p_x_high_limbs_range_constraint_1 = View(in.p_x_high_limbs_range_constraint_1);
    auto p_x_high_limbs_range_constraint_2 = View(in.p_x_high_limbs_range_constraint_2);
    auto p_x_high_limbs_range_constraint_3 = View(in.p_x_high_limbs_range_constraint_3);
    auto p_x_high_limbs_range_constraint_4 = View(in.p_x_high_limbs_range_constraint_4);
    auto p_x_high_limbs_shift = View(in.p_x_high_limbs_shift);
    auto p_x_high_limbs_range_constraint_0_shift = View(in.p_x_high_limbs_range_constraint_0_shift);
    auto p_x_high_limbs_range_constraint_1_shift = View(in.p_x_high_limbs_range_constraint_1_shift);
    auto p_x_high_limbs_range_constraint_2_shift = View(in.p_x_high_limbs_range_constraint_2_shift);
    auto p_x_high_limbs_range_constraint_3_shift = View(in.p_x_high_limbs_range_constraint_3_shift);
    auto p_y_low_limbs = View(in.p_y_low_limbs);
    auto p_y_low_limbs_range_constraint_0 = View(in.p_y_low_limbs_range_constraint_0);
    auto p_y_low_limbs_range_constraint_1 = View(in.p_y_low_limbs_range_constraint_1);
    auto p_y_low_limbs_range_constraint_2 = View(in.p_y_low_limbs_range_constraint_2);
    auto p_y_low_limbs_range_constraint_3 = View(in.p_y_low_limbs_range_constraint_3);
    auto p_y_low_limbs_range_constraint_4 = View(in.p_y_low_limbs_range_constraint_4);
    auto p_y_low_limbs_shift = View(in.p_y_low_limbs_shift);
    auto p_y_low_limbs_range_constraint_0_shift = View(in.p_y_low_limbs_range_constraint_0_shift);
    auto p_y_low_limbs_range_constraint_1_shift = View(in.p_y_low_limbs_range_constraint_1_shift);
    auto p_y_low_limbs_range_constraint_2_shift = View(in.p_y_low_limbs_range_constraint_2_shift);
    auto p_y_low_limbs_range_constraint_3_shift = View(in.p_y_low_limbs_range_constraint_3_shift);
    auto p_y_low_limbs_range_constraint_4_shift = View(in.p_y_low_limbs_range_constraint_4_shift);
    auto p_y_high_limbs = View(in.p_y_high_limbs);
    auto p_y_high_limbs_range_constraint_0 = View(in.p_y_high_limbs_range_constraint_0);
    auto p_y_high_limbs_range_constraint_1 = View(in.p_y_high_limbs_range_constraint_1);
    auto p_y_high_limbs_range_constraint_2 = View(in.p_y_high_limbs_range_constraint_2);
    auto p_y_high_limbs_range_constraint_3 = View(in.p_y_high_limbs_range_constraint_3);
    auto p_y_high_limbs_range_constraint_4 = View(in.p_y_high_limbs_range_constraint_4);
    auto p_y_high_limbs_shift = View(in.p_y_high_limbs_shift);
    auto p_y_high_limbs_range_constraint_0_shift = View(in.p_y_high_limbs_range_constraint_0_shift);
    auto p_y_high_limbs_range_constraint_1_shift = View(in.p_y_high_limbs_range_constraint_1_shift);
    auto p_y_high_limbs_range_constraint_2_shift = View(in.p_y_high_limbs_range_constraint_2_shift);
    auto p_y_high_limbs_range_constraint_3_shift = View(in.p_y_high_limbs_range_constraint_3_shift);
    auto z_low_limbs = View(in.z_low_limbs);
    auto z_low_limbs_range_constraint_0 = View(in.z_low_limbs_range_constraint_0);
    auto z_low_limbs_range_constraint_1 = View(in.z_low_limbs_range_constraint_1);
    auto z_low_limbs_range_constraint_2 = View(in.z_low_limbs_range_constraint_2);
    auto z_low_limbs_range_constraint_3 = View(in.z_low_limbs_range_constraint_3);
    auto z_low_limbs_range_constraint_4 = View(in.z_low_limbs_range_constraint_4);
    auto z_low_limbs_shift = View(in.z_low_limbs_shift);
    auto z_low_limbs_range_constraint_0_shift = View(in.z_low_limbs_range_constraint_0_shift);
    auto z_low_limbs_range_constraint_1_shift = View(in.z_low_limbs_range_constraint_1_shift);
    auto z_low_limbs_range_constraint_2_shift = View(in.z_low_limbs_range_constraint_2_shift);
    auto z_low_limbs_range_constraint_3_shift = View(in.z_low_limbs_range_constraint_3_shift);
    auto z_low_limbs_range_constraint_4_shift = View(in.z_low_limbs_range_constraint_4_shift);
    auto z_high_limbs = View(in.z_high_limbs);
    auto z_high_limbs_range_constraint_0 = View(in.z_high_limbs_range_constraint_0);
    auto z_high_limbs_range_constraint_1 = View(in.z_high_limbs_range_constraint_1);
    auto z_high_limbs_range_constraint_2 = View(in.z_high_limbs_range_constraint_2);
    auto z_high_limbs_range_constraint_3 = View(in.z_high_limbs_range_constraint_3);
    auto z_high_limbs_range_constraint_4 = View(in.z_high_limbs_range_constraint_4);
    auto z_high_limbs_shift = View(in.z_high_limbs_shift);
    auto z_high_limbs_range_constraint_0_shift = View(in.z_high_limbs_range_constraint_0_shift);
    auto z_high_limbs_range_constraint_1_shift = View(in.z_high_limbs_range_constraint_1_shift);
    auto z_high_limbs_range_constraint_2_shift = View(in.z_high_limbs_range_constraint_2_shift);
    auto z_high_limbs_range_constraint_3_shift = View(in.z_high_limbs_range_constraint_3_shift);
    auto z_high_limbs_range_constraint_4_shift = View(in.z_high_limbs_range_constraint_4_shift);
    auto accumulators_binary_limbs_0 = View(in.accumulators_binary_limbs_0);
    auto accumulators_binary_limbs_1 = View(in.accumulators_binary_limbs_1);
    auto accumulators_binary_limbs_2 = View(in.accumulators_binary_limbs_2);
    auto accumulators_binary_limbs_3 = View(in.accumulators_binary_limbs_3);
    auto accumulator_low_limbs_range_constraint_0 = View(in.accumulator_low_limbs_range_constraint_0);
    auto accumulator_low_limbs_range_constraint_1 = View(in.accumulator_low_limbs_range_constraint_1);
    auto accumulator_low_limbs_range_constraint_2 = View(in.accumulator_low_limbs_range_constraint_2);
    auto accumulator_low_limbs_range_constraint_3 = View(in.accumulator_low_limbs_range_constraint_3);
    auto accumulator_low_limbs_range_constraint_4 = View(in.accumulator_low_limbs_range_constraint_4);
    auto accumulator_low_limbs_range_constraint_0_shift = View(in.accumulator_low_limbs_range_constraint_0_shift);
    auto accumulator_low_limbs_range_constraint_1_shift = View(in.accumulator_low_limbs_range_constraint_1_shift);
    auto accumulator_low_limbs_range_constraint_2_shift = View(in.accumulator_low_limbs_range_constraint_2_shift);
    auto accumulator_low_limbs_range_constraint_3_shift = View(in.accumulator_low_limbs_range_constraint_3_shift);
    auto accumulator_low_limbs_range_constraint_4_shift = View(in.accumulator_low_limbs_range_constraint_4_shift);
    auto accumulator_high_limbs_range_constraint_0 = View(in.accumulator_high_limbs_range_constraint_0);
    auto accumulator_high_limbs_range_constraint_1 = View(in.accumulator_high_limbs_range_constraint_1);
    auto accumulator_high_limbs_range_constraint_2 = View(in.accumulator_high_limbs_range_constraint_2);
    auto accumulator_high_limbs_range_constraint_3 = View(in.accumulator_high_limbs_range_constraint_3);
    auto accumulator_high_limbs_range_constraint_4 = View(in.accumulator_high_limbs_range_constraint_4);
    auto accumulator_high_limbs_range_constraint_0_shift = View(in.accumulator_high_limbs_range_constraint_0_shift);
    auto accumulator_high_limbs_range_constraint_1_shift = View(in.accumulator_high_limbs_range_constraint_1_shift);
    auto accumulator_high_limbs_range_constraint_2_shift = View(in.accumulator_high_limbs_range_constraint_2_shift);
    auto accumulator_high_limbs_range_constraint_3_shift = View(in.accumulator_high_limbs_range_constraint_3_shift);
    auto quotient_low_binary_limbs = View(in.quotient_low_binary_limbs);
    auto quotient_low_limbs_range_constraint_0 = View(in.quotient_low_limbs_range_constraint_0);
    auto quotient_low_limbs_range_constraint_1 = View(in.quotient_low_limbs_range_constraint_1);
    auto quotient_low_limbs_range_constraint_2 = View(in.quotient_low_limbs_range_constraint_2);
    auto quotient_low_limbs_range_constraint_3 = View(in.quotient_low_limbs_range_constraint_3);
    auto quotient_low_limbs_range_constraint_4 = View(in.quotient_low_limbs_range_constraint_4);
    auto quotient_low_binary_limbs_shift = View(in.quotient_low_binary_limbs_shift);
    auto quotient_low_limbs_range_constraint_0_shift = View(in.quotient_low_limbs_range_constraint_0_shift);
    auto quotient_low_limbs_range_constraint_1_shift = View(in.quotient_low_limbs_range_constraint_1_shift);
    auto quotient_low_limbs_range_constraint_2_shift = View(in.quotient_low_limbs_range_constraint_2_shift);
    auto quotient_low_limbs_range_constraint_3_shift = View(in.quotient_low_limbs_range_constraint_3_shift);
    auto quotient_low_limbs_range_constraint_4_shift = View(in.quotient_low_limbs_range_constraint_4_shift);
    auto quotient_high_binary_limbs = View(in.quotient_high_binary_limbs);
    auto quotient_high_limbs_range_constraint_0 = View(in.quotient_high_limbs_range_constraint_0);
    auto quotient_high_limbs_range_constraint_1 = View(in.quotient_high_limbs_range_constraint_1);
    auto quotient_high_limbs_range_constraint_2 = View(in.quotient_high_limbs_range_constraint_2);
    auto quotient_high_limbs_range_constraint_3 = View(in.quotient_high_limbs_range_constraint_3);
    auto quotient_high_limbs_range_constraint_4 = View(in.quotient_high_limbs_range_constraint_4);
    auto quotient_high_binary_limbs_shift = View(in.quotient_high_binary_limbs_shift);
    auto quotient_high_limbs_range_constraint_0_shift = View(in.quotient_high_limbs_range_constraint_0_shift);
    auto quotient_high_limbs_range_constraint_1_shift = View(in.quotient_high_limbs_range_constraint_1_shift);
    auto quotient_high_limbs_range_constraint_2_shift = View(in.quotient_high_limbs_range_constraint_2_shift);
    auto quotient_high_limbs_range_constraint_3_shift = View(in.quotient_high_limbs_range_constraint_3_shift);
    auto relation_wide_limbs = View(in.relation_wide_limbs);
    auto relation_wide_limbs_range_constraint_0 = View(in.relation_wide_limbs_range_constraint_0);
    auto relation_wide_limbs_range_constraint_1 = View(in.relation_wide_limbs_range_constraint_1);
    auto relation_wide_limbs_range_constraint_2 = View(in.relation_wide_limbs_range_constraint_2);
    auto relation_wide_limbs_range_constraint_3 = View(in.relation_wide_limbs_range_constraint_3);
    auto p_x_high_limbs_range_constraint_tail_shift = View(in.p_x_high_limbs_range_constraint_tail_shift);
    auto accumulator_high_limbs_range_constraint_tail_shift =
        View(in.accumulator_high_limbs_range_constraint_tail_shift);
    auto relation_wide_limbs_shift = View(in.relation_wide_limbs_shift);
    auto relation_wide_limbs_range_constraint_0_shift = View(in.relation_wide_limbs_range_constraint_0_shift);
    auto relation_wide_limbs_range_constraint_1_shift = View(in.relation_wide_limbs_range_constraint_1_shift);
    auto relation_wide_limbs_range_constraint_2_shift = View(in.relation_wide_limbs_range_constraint_2_shift);
    auto relation_wide_limbs_range_constraint_3_shift = View(in.relation_wide_limbs_range_constraint_3_shift);
    auto p_y_high_limbs_range_constraint_tail_shift = View(in.p_y_high_limbs_range_constraint_tail_shift);
    auto quotient_high_limbs_range_constraint_tail_shift = View(in.quotient_high_limbs_range_constraint_tail_shift);
    auto p_x_low_limbs_range_constraint_tail = View(in.p_x_low_limbs_range_constraint_tail);
    auto p_x_low_limbs_range_constraint_tail_shift = View(in.p_x_low_limbs_range_constraint_tail_shift);
    auto p_x_high_limbs_range_constraint_tail = View(in.p_x_high_limbs_range_constraint_tail);
    auto p_x_high_limbs_range_constraint_4_shift = View(in.p_x_high_limbs_range_constraint_4_shift);
    auto p_y_low_limbs_range_constraint_tail = View(in.p_y_low_limbs_range_constraint_tail);
    auto p_y_low_limbs_range_constraint_tail_shift = View(in.p_y_low_limbs_range_constraint_tail_shift);
    auto p_y_high_limbs_range_constraint_tail = View(in.p_y_high_limbs_range_constraint_tail);
    auto p_y_high_limbs_range_constraint_4_shift = View(in.p_y_high_limbs_range_constraint_4_shift);
    auto z_low_limbs_range_constraint_tail = View(in.z_low_limbs_range_constraint_tail);
    auto z_low_limbs_range_constraint_tail_shift = View(in.z_low_limbs_range_constraint_tail_shift);
    auto z_high_limbs_range_constraint_tail = View(in.z_high_limbs_range_constraint_tail);
    auto z_high_limbs_range_constraint_tail_shift = View(in.z_high_limbs_range_constraint_tail_shift);
    auto accumulator_low_limbs_range_constraint_tail = View(in.accumulator_low_limbs_range_constraint_tail);
    auto accumulator_low_limbs_range_constraint_tail_shift = View(in.accumulator_low_limbs_range_constraint_tail_shift);
    auto accumulator_high_limbs_range_constraint_tail = View(in.accumulator_high_limbs_range_constraint_tail);
    auto accumulator_high_limbs_range_constraint_4_shift = View(in.accumulator_high_limbs_range_constraint_4_shift);
    auto quotient_low_limbs_range_constraint_tail = View(in.quotient_low_limbs_range_constraint_tail);
    auto quotient_low_limbs_range_constraint_tail_shift = View(in.quotient_low_limbs_range_constraint_tail_shift);
    auto quotient_high_limbs_range_constraint_tail = View(in.quotient_high_limbs_range_constraint_tail);
    auto quotient_high_limbs_range_constraint_4_shift = View(in.quotient_high_limbs_range_constraint_4_shift);
    auto x_lo_y_hi = View(in.x_lo_y_hi);
    auto x_hi_z_1 = View(in.x_hi_z_1);
    auto y_lo_z_2 = View(in.y_lo_z_2);
    auto x_lo_y_hi_shift = View(in.x_lo_y_hi_shift);
    auto x_hi_z_1_shift = View(in.x_hi_z_1_shift);
    auto y_lo_z_2_shift = View(in.y_lo_z_2_shift);
    auto lagrange_odd_in_minicircuit = View(in.lagrange_odd_in_minicircuit);

    // Contributions that decompose 50, 52, 68 or 84 bit limbs used for computation into range-constrained chunks
    // Contribution 1 , P_x lowest limb decomposition
    auto tmp_1 = ((p_x_low_limbs_range_constraint_0 + p_x_low_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                   p_x_low_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                   p_x_low_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                   p_x_low_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                  p_x_low_limbs);
    tmp_1 *= lagrange_odd_in_minicircuit;
    tmp_1 *= scaling_factor;
    std::get<0>(accumulators) += tmp_1;

    // Contribution 2 , P_x second lowest limb decomposition
    auto tmp_2 = ((p_x_low_limbs_range_constraint_0_shift + p_x_low_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                   p_x_low_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                   p_x_low_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                   p_x_low_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
                  p_x_low_limbs_shift);
    tmp_2 *= lagrange_odd_in_minicircuit;
    tmp_2 *= scaling_factor;
    std::get<1>(accumulators) += tmp_2;

    // Contribution 3 , P_x third limb decomposition
    auto tmp_3 = ((p_x_high_limbs_range_constraint_0 + p_x_high_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                   p_x_high_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                   p_x_high_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                   p_x_high_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                  p_x_high_limbs);
    tmp_3 *= lagrange_odd_in_minicircuit;
    tmp_3 *= scaling_factor;
    std::get<2>(accumulators) += tmp_3;

    // Contribution 4 , P_x highest limb decomposition
    auto tmp_4 =
        ((p_x_high_limbs_range_constraint_0_shift + p_x_high_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
          p_x_high_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
          p_x_high_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3) -
         p_x_high_limbs_shift);
    tmp_4 *= lagrange_odd_in_minicircuit;
    tmp_4 *= scaling_factor;
    std::get<3>(accumulators) += tmp_4;

    // Contribution 5 , P_y lowest limb decomposition
    auto tmp_5 = ((p_y_low_limbs_range_constraint_0 + p_y_low_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                   p_y_low_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                   p_y_low_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                   p_y_low_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                  p_y_low_limbs);
    tmp_5 *= lagrange_odd_in_minicircuit;
    tmp_5 *= scaling_factor;
    std::get<4>(accumulators) += tmp_5;

    // Contribution 6 , P_y second lowest limb decomposition
    auto tmp_6 = ((p_y_low_limbs_range_constraint_0_shift + p_y_low_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                   p_y_low_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                   p_y_low_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                   p_y_low_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
                  p_y_low_limbs_shift);
    tmp_6 *= lagrange_odd_in_minicircuit;
    tmp_6 *= scaling_factor;
    std::get<5>(accumulators) += tmp_6;

    // Contribution 7 , P_y third limb decomposition
    auto tmp_7 = ((p_y_high_limbs_range_constraint_0 + p_y_high_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                   p_y_high_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                   p_y_high_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                   p_y_high_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                  p_y_high_limbs);
    tmp_7 *= lagrange_odd_in_minicircuit;
    tmp_7 *= scaling_factor;
    std::get<6>(accumulators) += tmp_7;

    // Contribution 8 , P_y highest limb decomposition
    auto tmp_8 =
        ((p_y_high_limbs_range_constraint_0_shift + p_y_high_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
          p_y_high_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
          p_y_high_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3) -
         p_y_high_limbs_shift);
    tmp_8 *= lagrange_odd_in_minicircuit;
    tmp_8 *= scaling_factor;
    std::get<7>(accumulators) += tmp_8;

    // Contribution 9 , z_1 low limb decomposition
    auto tmp_9 =
        ((z_low_limbs_range_constraint_0 + z_low_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
          z_low_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 + z_low_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
          z_low_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
         z_low_limbs);
    tmp_9 *= lagrange_odd_in_minicircuit;
    tmp_9 *= scaling_factor;
    std::get<8>(accumulators) += tmp_9;

    // Contribution 10 , z_2 low limb decomposition
    auto tmp_10 = ((z_low_limbs_range_constraint_0_shift + z_low_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    z_low_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    z_low_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                    z_low_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
                   z_low_limbs_shift);
    tmp_10 *= lagrange_odd_in_minicircuit;
    tmp_10 *= scaling_factor;
    std::get<9>(accumulators) += tmp_10;

    // Contribution 11 , z_1 high limb decomposition
    auto tmp_11 =
        ((z_high_limbs_range_constraint_0 + z_high_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
          z_high_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 + z_high_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
          z_high_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
         z_high_limbs);
    tmp_11 *= lagrange_odd_in_minicircuit;
    tmp_11 *= scaling_factor;
    std::get<10>(accumulators) += tmp_11;

    // Contribution 12 , z_2 high limb decomposition
    auto tmp_12 = ((z_high_limbs_range_constraint_0_shift + z_high_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    z_high_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    z_high_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                    z_high_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
                   z_high_limbs_shift);
    tmp_12 *= lagrange_odd_in_minicircuit;
    tmp_12 *= scaling_factor;
    std::get<11>(accumulators) += tmp_12;

    // Contribution 13 , accumulator lowest limb decomposition
    auto tmp_13 =
        ((accumulator_low_limbs_range_constraint_0 + accumulator_low_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
          accumulator_low_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
          accumulator_low_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
          accumulator_low_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
         accumulators_binary_limbs_0);
    tmp_13 *= lagrange_odd_in_minicircuit;
    tmp_13 *= scaling_factor;
    std::get<12>(accumulators) += tmp_13;
    // Contribution 14 , accumulator second limb decomposition
    auto tmp_14 = ((accumulator_low_limbs_range_constraint_0_shift +
                    accumulator_low_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    accumulator_low_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    accumulator_low_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                    accumulator_low_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
                   accumulators_binary_limbs_1);
    tmp_14 *= lagrange_odd_in_minicircuit;
    tmp_14 *= scaling_factor;
    std::get<13>(accumulators) += tmp_14;

    // Contribution 15 , accumulator second highest limb decomposition
    auto tmp_15 =
        ((accumulator_high_limbs_range_constraint_0 + accumulator_high_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
          accumulator_high_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
          accumulator_high_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
          accumulator_high_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
         accumulators_binary_limbs_2);
    tmp_15 *= lagrange_odd_in_minicircuit;
    tmp_15 *= scaling_factor;
    std::get<14>(accumulators) += tmp_15;
    // Contribution 16 , accumulator highest limb decomposition
    auto tmp_16 = ((accumulator_high_limbs_range_constraint_0_shift +
                    accumulator_high_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    accumulator_high_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    accumulator_high_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3) -
                   accumulators_binary_limbs_3);
    tmp_16 *= lagrange_odd_in_minicircuit;
    tmp_16 *= scaling_factor;
    std::get<15>(accumulators) += tmp_16;

    // Contribution 15 , quotient lowest limb decomposition
    auto tmp_17 = ((quotient_low_limbs_range_constraint_0 + quotient_low_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                    quotient_low_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                    quotient_low_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                    quotient_low_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                   quotient_low_binary_limbs);
    tmp_17 *= lagrange_odd_in_minicircuit;
    tmp_17 *= scaling_factor;
    std::get<16>(accumulators) += tmp_17;
    // Contribution 16 , quotient second lowest limb decomposition
    auto tmp_18 =
        ((quotient_low_limbs_range_constraint_0_shift + quotient_low_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
          quotient_low_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
          quotient_low_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
          quotient_low_limbs_range_constraint_4_shift * MICRO_LIMB_SHIFTx4) -
         quotient_low_binary_limbs_shift);
    tmp_18 *= lagrange_odd_in_minicircuit;
    tmp_18 *= scaling_factor;
    std::get<17>(accumulators) += tmp_18;

    // Contribution 19 , quotient second highest limb decomposition
    auto tmp_19 = ((quotient_high_limbs_range_constraint_0 + quotient_high_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                    quotient_high_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                    quotient_high_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                    quotient_high_limbs_range_constraint_4 * MICRO_LIMB_SHIFTx4) -
                   quotient_high_binary_limbs);
    tmp_19 *= lagrange_odd_in_minicircuit;
    tmp_19 *= scaling_factor;
    std::get<18>(accumulators) += tmp_19;
    // Contribution 20 , quotient highest limb decomposition
    auto tmp_20 = ((quotient_high_limbs_range_constraint_0_shift +
                    quotient_high_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    quotient_high_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    quotient_high_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3) -
                   quotient_high_binary_limbs_shift);
    tmp_20 *= lagrange_odd_in_minicircuit;
    tmp_20 *= scaling_factor;
    std::get<19>(accumulators) += tmp_20;

    // Contribution 21 , decomposition of the low wide relation limb used for the bigfield relation.
    // N.B. top microlimbs of relation wide limbs are stored in microlimbs for range constraints of P_x, P_y,
    // accumulator and quotient. This is to save space and because these microlimbs are not used by their namesakes,
    // since top limbs in 254/6-bit values use one less microlimb for the top 50/52-bit limb
    auto tmp_21 = ((relation_wide_limbs_range_constraint_0 + relation_wide_limbs_range_constraint_1 * MICRO_LIMB_SHIFT +
                    relation_wide_limbs_range_constraint_2 * MICRO_LIMB_SHIFTx2 +
                    relation_wide_limbs_range_constraint_3 * MICRO_LIMB_SHIFTx3 +
                    p_x_high_limbs_range_constraint_tail_shift * MICRO_LIMB_SHIFTx4 +
                    accumulator_high_limbs_range_constraint_tail_shift * MICRO_LIMB_SHIFTx5) -
                   relation_wide_limbs);
    tmp_21 *= lagrange_odd_in_minicircuit;
    tmp_21 *= scaling_factor;
    std::get<20>(accumulators) += tmp_21;

    // Contribution 22 , decomposition of high relation limb
    auto tmp_22 = ((relation_wide_limbs_range_constraint_0_shift +
                    relation_wide_limbs_range_constraint_1_shift * MICRO_LIMB_SHIFT +
                    relation_wide_limbs_range_constraint_2_shift * MICRO_LIMB_SHIFTx2 +
                    relation_wide_limbs_range_constraint_3_shift * MICRO_LIMB_SHIFTx3 +
                    p_y_high_limbs_range_constraint_tail_shift * MICRO_LIMB_SHIFTx4 +
                    quotient_high_limbs_range_constraint_tail_shift * MICRO_LIMB_SHIFTx5) -
                   relation_wide_limbs_shift);
    tmp_22 *= lagrange_odd_in_minicircuit;
    tmp_22 *= scaling_factor;
    std::get<21>(accumulators) += tmp_22;

    // Contributions enfocing a reduced range constraint on high limbs (these relation force the last microlimb in
    // each limb to be more severely range constrained)

    // Contribution 23, range constrain the highest microlimb of lowest P.x limb to be 12 bits (68 % 14 = 12)
    auto tmp_23 = p_x_low_limbs_range_constraint_4 * SHIFT_12_TO_14 - p_x_low_limbs_range_constraint_tail;
    tmp_23 *= lagrange_odd_in_minicircuit;
    tmp_23 *= scaling_factor;
    std::get<22>(accumulators) += tmp_23;

    // Contribution 24, range constrain the highest microlimb of second lowest P.x limb to be 12 bits
    auto tmp_24 = p_x_low_limbs_range_constraint_4_shift * SHIFT_12_TO_14 - p_x_low_limbs_range_constraint_tail_shift;
    tmp_24 *= lagrange_odd_in_minicircuit;
    tmp_24 *= scaling_factor;
    std::get<23>(accumulators) += tmp_24;

    // Contribution 25, range constrain the highest microlimb of second highest P.x limb to be 12 bits
    auto tmp_25 = p_x_high_limbs_range_constraint_4 * SHIFT_12_TO_14 - p_x_high_limbs_range_constraint_tail;
    tmp_25 *= lagrange_odd_in_minicircuit;
    tmp_25 *= scaling_factor;
    std::get<24>(accumulators) += tmp_25;

    // Contribution 26, range constrain the highest microilmb of highest P.x limb to be 8 bits (50 % 14 = 8)
    auto tmp_26 = (p_x_high_limbs_range_constraint_3_shift * SHIFT_8_TO_14 - p_x_high_limbs_range_constraint_4_shift);

    tmp_26 *= lagrange_odd_in_minicircuit;
    tmp_26 *= scaling_factor;
    std::get<25>(accumulators) += tmp_26;

    // Contribution 27, range constrain the highest microlimb of lowest P.y limb to be 12 bits (68 % 14 = 12)
    auto tmp_27 = p_y_low_limbs_range_constraint_4 * SHIFT_12_TO_14 - p_y_low_limbs_range_constraint_tail;
    tmp_27 *= lagrange_odd_in_minicircuit;
    tmp_27 *= scaling_factor;
    std::get<26>(accumulators) += tmp_27;

    // Contribution 28, range constrain the highest microlimb of second lowest P.y limb to be 12 bits (68 % 14 = 12)
    auto tmp_28 = p_y_low_limbs_range_constraint_4_shift * SHIFT_12_TO_14 - p_y_low_limbs_range_constraint_tail_shift;
    tmp_28 *= lagrange_odd_in_minicircuit;
    tmp_28 *= scaling_factor;
    std::get<27>(accumulators) += tmp_28;

    // Contribution 29, range constrain the highest microlimb of second highest P.y limb to be 12 bits (68 % 14 =
    // 12)
    auto tmp_29 = p_y_high_limbs_range_constraint_4 * SHIFT_12_TO_14 - p_y_high_limbs_range_constraint_tail;
    tmp_29 *= lagrange_odd_in_minicircuit;
    tmp_29 *= scaling_factor;
    std::get<28>(accumulators) += tmp_29;

    // Contribution 30, range constrain the highest microlimb of highest P.y limb to be 8 bits (50 % 14 = 8)
    auto tmp_30 = (p_y_high_limbs_range_constraint_3_shift * SHIFT_8_TO_14 - p_y_high_limbs_range_constraint_4_shift);

    tmp_30 *= lagrange_odd_in_minicircuit;
    tmp_30 *= scaling_factor;
    std::get<29>(accumulators) += tmp_30;

    // Contribution 31, range constrain the highest microlimb of low z1 limb to be 12 bits (68 % 14 = 12)
    auto tmp_31 = (z_low_limbs_range_constraint_4 * SHIFT_12_TO_14 - z_low_limbs_range_constraint_tail);
    tmp_31 *= lagrange_odd_in_minicircuit;
    tmp_31 *= scaling_factor;
    std::get<30>(accumulators) += tmp_31;

    // Contribution 32, range constrain the highest microlimb of low z2 limb to be 12 bits (68 % 14 = 12)
    auto tmp_32 = (z_low_limbs_range_constraint_4_shift * SHIFT_12_TO_14 - z_low_limbs_range_constraint_tail_shift);
    tmp_32 *= lagrange_odd_in_minicircuit;
    tmp_32 *= scaling_factor;
    std::get<31>(accumulators) += tmp_32;

    // Contribution 33, range constrain the highest microlimb of high z1 limb to be 4 bits (60 % 14 = 12)
    auto tmp_33 = (z_high_limbs_range_constraint_4 * SHIFT_4_TO_14 - z_high_limbs_range_constraint_tail);
    tmp_33 *= lagrange_odd_in_minicircuit;
    tmp_33 *= scaling_factor;
    std::get<32>(accumulators) += tmp_33;

    // Contribution 34, range constrain the highest microlimb of high z2 limb to be 4 bits (60 % 14 = 12)
    auto tmp_34 = (z_high_limbs_range_constraint_4_shift * SHIFT_4_TO_14 - z_high_limbs_range_constraint_tail_shift);
    tmp_34 *= lagrange_odd_in_minicircuit;
    tmp_34 *= scaling_factor;
    std::get<33>(accumulators) += tmp_34;

    // Contribution 35, range constrain the highest microlimb of lowest current accumulator limb to be 12 bits (68 %
    // 14 = 12)
    auto tmp_35 =
        (accumulator_low_limbs_range_constraint_4 * SHIFT_12_TO_14 - accumulator_low_limbs_range_constraint_tail);
    tmp_35 *= lagrange_odd_in_minicircuit;
    tmp_35 *= scaling_factor;
    std::get<34>(accumulators) += tmp_35;

    // Contribution 36, range constrain the highest microlimb of second lowest current accumulator limb to be 12
    // bits (68 % 14 = 12)
    auto tmp_36 = (accumulator_low_limbs_range_constraint_4_shift * SHIFT_12_TO_14 -
                   accumulator_low_limbs_range_constraint_tail_shift);
    tmp_36 *= lagrange_odd_in_minicircuit;
    tmp_36 *= scaling_factor;
    std::get<35>(accumulators) += tmp_36;

    // Contribution 37, range constrain the highest microlimb of second highest current accumulator limb to be 12
    // bits (68 % 14 = 12)
    auto tmp_37 =
        (accumulator_high_limbs_range_constraint_4 * SHIFT_12_TO_14 - accumulator_high_limbs_range_constraint_tail);
    tmp_37 *= lagrange_odd_in_minicircuit;
    tmp_37 *= scaling_factor;
    std::get<36>(accumulators) += tmp_37;

    // Contribution 38, range constrain the highest microlimb of highest current accumulator limb to be 8 bits (50 %
    // 14 = 12)
    auto tmp_38 = (accumulator_high_limbs_range_constraint_3_shift * SHIFT_8_TO_14 -
                   accumulator_high_limbs_range_constraint_4_shift);
    tmp_38 *= lagrange_odd_in_minicircuit;
    tmp_38 *= scaling_factor;
    std::get<37>(accumulators) += tmp_38;

    // Contribution 39, range constrain the highest microlimb of lowest quotient limb to be 12 bits (68 % 14 = 12)
    auto tmp_39 = (quotient_low_limbs_range_constraint_4 * SHIFT_12_TO_14 - quotient_low_limbs_range_constraint_tail);
    tmp_39 *= lagrange_odd_in_minicircuit;
    tmp_39 *= scaling_factor;
    std::get<38>(accumulators) += tmp_39;

    // Contribution 40, range constrain the highest microlimb of second lowest quotient limb to be 12 bits (68 % 14
    // = 12)
    auto tmp_40 =
        (quotient_low_limbs_range_constraint_4_shift * SHIFT_12_TO_14 - quotient_low_limbs_range_constraint_tail_shift);
    tmp_40 *= lagrange_odd_in_minicircuit;
    tmp_40 *= scaling_factor;
    std::get<39>(accumulators) += tmp_40;

    // Contribution 41, range constrain the highest microlimb of second highest quotient limb to be 12 bits (68 % 14
    // = 12)
    auto tmp_41 = (quotient_high_limbs_range_constraint_4 * SHIFT_12_TO_14 - quotient_high_limbs_range_constraint_tail);
    tmp_41 *= lagrange_odd_in_minicircuit;
    tmp_41 *= scaling_factor;
    std::get<40>(accumulators) += tmp_41;

    // Contribution 42, range constrain the highest microlimb of highest quotient limb to be 10 bits (52 % 14 = 12)
    auto tmp_42 =
        (quotient_high_limbs_range_constraint_3_shift * SHIFT_10_TO_14 - quotient_high_limbs_range_constraint_4_shift);
    tmp_42 *= lagrange_odd_in_minicircuit;
    tmp_42 *= scaling_factor;
    std::get<41>(accumulators) += tmp_42;

    // Contributions where we decompose initial EccOpQueue values into 68-bit limbs

    // Contribution 43, decompose x_lo
    auto tmp_43 = (p_x_low_limbs + p_x_low_limbs_shift * LIMB_SHIFT) - x_lo_y_hi;
    tmp_43 *= lagrange_odd_in_minicircuit;
    tmp_43 *= scaling_factor;
    std::get<42>(accumulators) += tmp_43;

    // Contribution 44, decompose x_hi
    auto tmp_44 = (p_x_high_limbs + p_x_high_limbs_shift * LIMB_SHIFT) - x_hi_z_1;
    tmp_44 *= lagrange_odd_in_minicircuit;
    tmp_44 *= scaling_factor;
    std::get<43>(accumulators) += tmp_44;
    // Contribution 45, decompose y_lo
    auto tmp_45 = (p_y_low_limbs + p_y_low_limbs_shift * LIMB_SHIFT) - y_lo_z_2;
    tmp_45 *= lagrange_odd_in_minicircuit;
    tmp_45 *= scaling_factor;
    std::get<44>(accumulators) += tmp_45;

    // Contribution 46, decompose y_hi
    auto tmp_46 = (p_y_high_limbs + p_y_high_limbs_shift * LIMB_SHIFT) - x_lo_y_hi_shift;
    tmp_46 *= lagrange_odd_in_minicircuit;
    tmp_46 *= scaling_factor;
    std::get<45>(accumulators) += tmp_46;

    // Contribution 47, decompose z1
    auto tmp_47 = (z_low_limbs + z_high_limbs * LIMB_SHIFT) - x_hi_z_1_shift;
    tmp_47 *= lagrange_odd_in_minicircuit;
    tmp_47 *= scaling_factor;
    std::get<46>(accumulators) += tmp_47;

    // Contribution 48, decompose z2
    auto tmp_48 = (z_low_limbs_shift + z_high_limbs_shift * LIMB_SHIFT) - y_lo_z_2_shift;
    tmp_48 *= lagrange_odd_in_minicircuit;
    tmp_48 *= scaling_factor;
    std::get<47>(accumulators) += tmp_48;
};

template class GoblinTranslatorDecompositionRelationImpl<bb::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(GoblinTranslatorDecompositionRelationImpl, honk::flavor::GoblinTranslator);

} // namespace bb
