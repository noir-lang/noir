#include "ecc_wnaf_relation.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief ECCVMWnafRelationImpl evaluates relations that convert scalar multipliers into 4-bit WNAF slices
 * @details Each WNAF slice is a 4-bit slice representing one of 16 integers { -15, -13, ..., 15 }
 * Each WNAF slice is represented via two 2-bit columns (precompute_s1hi, ..., precompute_s4lo)
 * One 128-bit scalar multiplier is processed across 8 rows, indexed by a round variable.
 * The following table describes the structure for one scalar.
 *
 * | point_transition | round | slices          | skew   | scalar_sum                      |
 * | ---------------- | ----- | --------------- | ------ | ------------------------------- |
 * | 0                | 0     | s0,s1,s2,s3     | 0      | 0                               |
 * | 0                | 1     | s4,s5,s6,s7     | 0      | \sum_{i=0}^4 16^i * s_{31 - i}  |
 * | 0                | 2     | s8,s9,s10,s11   | 0      | \sum_{i=0}^8 16^i * s_{31 - i}  |
 * | 0                | 3     | s12,s13,s14,s14 | 0      | \sum_{i=0}^12 16^i * s_{31 - i} |
 * | 0                | 4     | s16,s17,s18,s19 | 0      | \sum_{i=0}^16 16^i * s_{31 - i} |
 * | 0                | 5     | s20,s21,s22,s23 | 0      | \sum_{i=0}^20 16^i * s_{31 - i} |
 * | 0                | 6     | s24,s25,s26,s27 | 0      | \sum_{i=0}^24 16^i * s_{31 - i} |
 * | 1                | 7     | s28,s29,s30,s31 | s_skew | \sum_{i=0}^28 16^i * s_{31 - i} |
 *
 * The value of the input scalar is equal to the following:
 *
 * scalar = 2^16 * scalar_sum + 2^12 * s31 + 2^8 * s30 + 2^4 * s29 + s28 - s_skew
 * We use a set equality check in `ecc_set_relation.hpp` to validate the above value maps to the correct input
 * scalar for a given value of `pc`.
 *
 * The column `point_transition` is committed to by the Prover, we must constrain it is correctly computed (see
 * `ecc_point_table_relation.cpp` for details)
 *
 * @tparam FF
 * @tparam AccumulatorTypes
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMWnafRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                           const AllEntities& in,
                                           const Parameters& /*unused*/,
                                           const FF& scaling_factor)
{
    using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    auto scalar_sum = View(in.precompute_scalar_sum);
    auto scalar_sum_new = View(in.precompute_scalar_sum_shift);
    auto q_transition = View(in.precompute_point_transition);
    auto round = View(in.precompute_round);
    auto round_shift = View(in.precompute_round_shift);
    auto pc = View(in.precompute_pc);
    auto pc_shift = View(in.precompute_pc_shift);
    // precompute_select is a boolean column. We only evaluate the ecc_wnaf_relation and the ecc_point_table_relation if
    // `precompute_select=1`
    auto precompute_select = View(in.precompute_select);
    auto precompute_select_shift = View(in.precompute_select_shift);

    const auto& precompute_skew = View(in.precompute_skew);

    const std::array<View, 8> slices{
        View(in.precompute_s1hi), View(in.precompute_s1lo), View(in.precompute_s2hi), View(in.precompute_s2lo),
        View(in.precompute_s3hi), View(in.precompute_s3lo), View(in.precompute_s4hi), View(in.precompute_s4lo),
    };

    const auto range_constraint_slice_to_2_bits = [&scaling_factor](const View& s, auto& acc) {
        acc += s * (s - 1) * (s - 2) * (s - 3) * scaling_factor;
    };

    const auto convert_to_wnaf = [](const View& s0, const View& s1) {
        auto t = s0 + s0;
        t += t;
        t += s1;
        auto naf = t + t - 15;
        return naf;
    };

    const auto scaled_transition = q_transition * scaling_factor;
    const auto scaled_transition_is_zero = -scaled_transition + scaling_factor;
    /**
     * @brief Constrain each of our scalar slice chunks (s1, ..., s8) to be 2 bits.
     * Doing range checks this way vs permutation-based range check removes need to create sorted list + grand product
     * polynomial. Probably cheaper even if we have to split each 4-bit WNAF slice into 2-bit chunks.
     */
    range_constraint_slice_to_2_bits(slices[0], std::get<0>(accumulator));
    range_constraint_slice_to_2_bits(slices[1], std::get<1>(accumulator));
    range_constraint_slice_to_2_bits(slices[2], std::get<2>(accumulator));
    range_constraint_slice_to_2_bits(slices[3], std::get<3>(accumulator));
    range_constraint_slice_to_2_bits(slices[4], std::get<4>(accumulator));
    range_constraint_slice_to_2_bits(slices[5], std::get<5>(accumulator));
    range_constraint_slice_to_2_bits(slices[6], std::get<6>(accumulator));
    range_constraint_slice_to_2_bits(slices[7], std::get<7>(accumulator));

    /**
     * @brief If we are processing a new scalar (q_transition = 1), validate that the first slice is positive.
     *        This requires us to validate slice1 is in the range [8, ... 15].
     *        (when converted into wnaf form this maps to the range [1, 3, ..., 15]).
     *        We do this to ensure the final scalar sum is positive.
     *        We already know slice1 is in the range [0, ..., 15]
     *        To check the range [8, ..., 15] we validate the most significant 2 bits (s1) are >=2
     */
    const auto s1_shift = View(in.precompute_s1hi_shift);
    const auto s1_shift_msb_set = (s1_shift - 2) * (s1_shift - 3);
    std::get<20>(accumulator) += scaled_transition * precompute_select_shift * s1_shift_msb_set;

    /**
     * @brief Convert each pair of 2-bit scalar slices into a 4-bit windowed-non-adjacent-form slice.
     * Conversion from binary -> wnaf = 2 * binary - 15.
     * Converts a value in [0, ..., 15] into [-15, -13, -11, -9, -7, -5, -3, -1, 1, 3, 5, 7, 9, 11 , 13, 15].
     * We use WNAF representation to avoid case where we are conditionally adding a point in our MSM algo.
     */
    const auto w0 = convert_to_wnaf(slices[0], slices[1]);
    const auto w1 = convert_to_wnaf(slices[2], slices[3]);
    const auto w2 = convert_to_wnaf(slices[4], slices[5]);
    const auto w3 = convert_to_wnaf(slices[6], slices[7]);

    /**
     * @brief Slice consistency check.
     * We require that `scalar_sum` on the next row correctly accumulates the 4  WNAF slices present on the current row
     * (i.e. 16 WNAF bits).
     * i.e. next_scalar_sum - 2^{16} * current_scalar_sum - 2^12 * w_0 - 2^8 * w_1 - 2^4 * w_2 - w_3 = 0
     * @note We only perform slice_consistency check when next row is processing the same scalar as the current row!
     *       i.e. when q_transition  = 0
     * TODO(@zac-williamson) Optimize WNAF use (#2224)
     */
    auto row_slice = w0;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += w1;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += w2;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += row_slice;
    row_slice += w3;
    auto sum_delta = scalar_sum * FF(1ULL << 16) + row_slice;
    const auto check_sum = scalar_sum_new - sum_delta;
    std::get<8>(accumulator) += precompute_select * check_sum * scaled_transition_is_zero;

    /**
     * @brief Round transition logic.
     * Goal: `round` is an integer in [0, ... 7] that tracks how many slices we have processed for a given scalar.
     * i.e. number of 4-bit WNAF slices processed = round * 4.
     * We apply the following constraints:
     * If q_transition = 0, round increments by 1 between rows.
     * If q_transition = 1, round value at current row = 7
     * If q_transition = 1, round value at next row = 0
     * Question: is this sufficient? We don't actually range constrain `round` (expensive if we don't need to!).
     * Let us analyze...
     * 1. When `q_transition = 1`, we use a set membership check to map the tuple of (pc, scalar_sum) into a set.
     * We compare this set with an equivalent set generated from the transcript columns. The sets must match.
     * 2. Only case where, at row `i`, a Prover can set `round` to value > 7 is if `q_transition = 0` for all j > i.
     *  `precompute_pc` decrements by 1 when `q_transition` = 1
     * We can infer from 1, 2, that if `round > 7`, the resulting wnafs will map into a set at a value of `pc` that is
     * greater than all valid msm pc values (assuming the set equivalence check on the scalar sums is satisfied).
     * The resulting msm output of such a computation cannot be mapped into the set of msm outputs in
     * the transcript columns (see relations in ecc_msm_relation.cpp).
     * Conclusion: not applying a strict range-check on `round` does not affect soundness (TODO(@zac-williamson)
     * validate this! #2225)
     */
    // We combine checks 0, 1 into a single relation
    // q_transition * (round - 7) + (-q_transition + 1) * (round_shift - round - 1)
    // => q_transition * (round - 7 - round_shift + round + 1) + (round_shift - round - 1)
    // => q_transition * (2 * round - round_shift - 6) + (round_shift - round - 1)
    const auto round_check = round_shift - round - 1;
    std::get<9>(accumulator) += precompute_select * scaled_transition * ((round - round_check - 7) + round_check);
    std::get<10>(accumulator) += precompute_select * scaled_transition * round_shift;

    /**
     * @brief Scalar transition checks.
     * 1: if q_transition = 1, scalar_sum_new = 0
     * 2: if q_transition = 0, pc at next row = pc at current row
     * 3: if q_transition = 1, pc at next row = pc at current row - 1 (decrements by 1)
     * (we combine 2 and 3 into a single relation)
     */
    std::get<11>(accumulator) += precompute_select * scalar_sum_new * scaled_transition;
    // (2, 3 combined): q_transition * (pc - pc_shift - 1) + (-q_transition + 1) * (pc_shift - pc)
    // => q_transition * (-2 * (pc_shift - pc) - 1) + (pc_shift - pc)
    const auto pc_delta = pc_shift - pc;
    std::get<12>(accumulator) +=
        precompute_select * (scaled_transition * ((-pc_delta - pc_delta - 1)) + pc_delta * scaling_factor);

    /**
     * @brief Validate skew is 0 or 7
     * 7 is the wnaf representation of -1.
     * We have one skew variable per scalar multiplier. We can only represent odd integers in WNAF form.
     * If input scalar is even, we must subtract 1 from WNAF scalar sum to get actual value (i.e. where skew = 7)
     * We use skew in two places.
     * 1: when validating sum of wnaf slices matches input scalar (we add skew to scalar_sum in ecc_set_relation)
     * 2: in ecc_msm_relation. Final MSM round uses skew to conditionally subtract a point from the accumulator
     */
    std::get<13>(accumulator) += precompute_select * (precompute_skew * (precompute_skew - 7)) * scaling_factor;

    const auto precompute_select_zero = (-precompute_select + 1) * scaling_factor;
    std::get<14>(accumulator) += precompute_select_zero * (w0 + 15);
    std::get<15>(accumulator) += precompute_select_zero * (w1 + 15);
    std::get<16>(accumulator) += precompute_select_zero * (w2 + 15);
    std::get<17>(accumulator) += precompute_select_zero * (w3 + 15);

    std::get<18>(accumulator) += precompute_select_zero * round;
    std::get<19>(accumulator) += precompute_select_zero * pc;

    // TODO(@zac-williamson #2226)
    // if precompute_select = 0, validate pc, round, slice values are all zero
    // If we do this we can reduce the degree of the set equivalence relations
    // (currently when checking pc/round/wnaf tuples from WNAF columns match those from MSM columns,
    //  we conditionally include tuples depending on if precompute_select = 1 (for WNAF columns) or if q_add1/2/3/4 = 1
    //  (for MSM columns).
    // If we KNOW that the wnaf tuple values are 0 when precompute_select = 0, we can remove the conditional checks in
    // the set equivalence relation
}

template class ECCVMWnafRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationImpl, flavor::ECCVM);

} // namespace bb::honk::sumcheck
