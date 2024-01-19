#include "ecc_msm_relation.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief MSM relations that evaluate the Strauss multiscalar multiplication algorithm.
 *
 * @details
 * The Strauss algorithm for a size-k MSM takes scalars/points (a_i, [P_i]) for i = 0 to k-1.
 * The specific algoritm we use is the following:
 *
 * PHASE 1: Precomputation (performed in ecc_wnaf_relation.hpp, ecc_point_table_relation.hpp)
 * Each scalar a_i is split into 4-bit WNAF slices s_{j, i} for j = 0 to 31, and a skew bool skew_i
 * For each point [P_i] a size-16 lookup table of points, T_i, is computed { [-15 P_i], [-13 P_i], ..., [15 P_i] }
 *
 * PHASE 2: MSM evaluation
 * MSM evaluation is split into 32 rounds that operate on an accumulator point [Acc]
 * The first 31 rounds are composed of an ADDITION round and a DOUBLE round.
 * The final 32nd round is composed of an ADDITION round and a SKEW round.
 *
 * ADDITION round (round = j):
 * [Acc] = [Acc] + T_i[a_{i, j}] for all i in [0, ... k-1]
 *
 * DOUBLE round:
 * [Acc] = 16 * [Acc] (four point doublings)
 *
 * SKEW round:
 * If skew_i == 1, [Acc] = [Acc] - [P_i] for all i in [0, ..., k - 1]
 *
 * The relations in ECCVMMSMRelationImpl constrain the ADDITION, DOUBLE and SKEW rounds
 * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMMSMRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                          const AllEntities& in,
                                          const Parameters& /*unused*/,
                                          const FF& scaling_factor)
{
    using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    const auto& x1 = View(in.msm_x1);
    const auto& y1 = View(in.msm_y1);
    const auto& x2 = View(in.msm_x2);
    const auto& y2 = View(in.msm_y2);
    const auto& x3 = View(in.msm_x3);
    const auto& y3 = View(in.msm_y3);
    const auto& x4 = View(in.msm_x4);
    const auto& y4 = View(in.msm_y4);
    const auto& collision_inverse1 = View(in.msm_collision_x1);
    const auto& collision_inverse2 = View(in.msm_collision_x2);
    const auto& collision_inverse3 = View(in.msm_collision_x3);
    const auto& collision_inverse4 = View(in.msm_collision_x4);
    const auto& lambda1 = View(in.msm_lambda1);
    const auto& lambda2 = View(in.msm_lambda2);
    const auto& lambda3 = View(in.msm_lambda3);
    const auto& lambda4 = View(in.msm_lambda4);
    const auto& lagrange_first = View(in.lagrange_first);
    const auto& add1 = View(in.msm_add1);
    const auto& add1_shift = View(in.msm_add1_shift);
    const auto& add2 = View(in.msm_add2);
    const auto& add3 = View(in.msm_add3);
    const auto& add4 = View(in.msm_add4);
    const auto& acc_x = View(in.msm_accumulator_x);
    const auto& acc_y = View(in.msm_accumulator_y);
    const auto& acc_x_shift = View(in.msm_accumulator_x_shift);
    const auto& acc_y_shift = View(in.msm_accumulator_y_shift);
    const auto& slice1 = View(in.msm_slice1);
    const auto& slice2 = View(in.msm_slice2);
    const auto& slice3 = View(in.msm_slice3);
    const auto& slice4 = View(in.msm_slice4);
    const auto& msm_transition = View(in.msm_transition);
    const auto& msm_transition_shift = View(in.msm_transition_shift);
    const auto& round = View(in.msm_round);
    const auto& round_shift = View(in.msm_round_shift);
    const auto& q_add = View(in.msm_add);
    const auto& q_add_shift = View(in.msm_add_shift);
    const auto& q_skew = View(in.msm_skew);
    const auto& q_skew_shift = View(in.msm_skew_shift);
    const auto& q_double = View(in.msm_double);
    const auto& q_double_shift = View(in.msm_double_shift);
    const auto& msm_size = View(in.msm_size_of_msm);
    // const auto& msm_size_shift = View(in.msm_size_of_msm_shift);
    const auto& pc = View(in.msm_pc);
    const auto& pc_shift = View(in.msm_pc_shift);
    const auto& count = View(in.msm_count);
    const auto& count_shift = View(in.msm_count_shift);
    auto is_not_first_row = (-lagrange_first + 1);

    /**
     * @brief Evaluating ADDITION rounds
     *
     * This comment describes the algorithm we want the Prover to perform.
     * The relations we constrain are supposed to make an honest Prover compute witnesses consistent with the following:
     *
     * For an MSM of size-k...
     *
     * Algorithm to determine if round at shifted row is an ADDITION round:
     *     1. count_shift < msm_size
     *     2. round != 32
     *
     * Algorithm to process MSM ADDITION round:
     * 1. If `round == 0` set `count = 0`
     * 2. For j = pc + count, perform the following:
     * 2a.      If j + 3 < k: [P_{j + 3}] = T_{j+ 3}[slice_{j + 3}]
     * 2b.      If j + 2 < k: [P_{j + 2}] = T_{j+ 2}[slice_{j + 2}]
     * 2c.      If j + 1 < k: [P_{j + 1}] = T_{j+ 1}[slice_{j + 1}]
     * 2d.                    [P_{j}]     = T_{j}[slice_{j}]
     * 2e.      If j + 3 < k: [Acc_shift] = [Acc] + [P_j] + [P_{j+1}] + [P_{j+2}] + [P_{j+3}]
     * 2f. Else If j + 2 < k: [Acc_shift] = [Acc] + [P_j] + [P_{j+1}] + [P_{j+2}]
     * 2g. Else IF j + 1 < k: [Acc_shift] = [Acc] + [P_j] + [P_{j+1}]
     * 2h. Else             : [Acc_shift] = [Acc] + [P_j]
     * 3. `count_shift = count + 1 + (j + 1 < k) + (j + 2 < k) + (j + 3 < k)`
     */

    /**
     * @brief Constraining addition rounds
     *
     * The boolean column q_add describes whether a round is an ADDITION round.
     * The values of q_add are Prover-defined. We need to ensure they set q_add correctly.
     * We rely on the following statements that we assume are constrained to be true (from other relations):
     *      1. The set of reads into (pc, round, wnaf_slice) is constructed when q_add = 1
     *      2. The set of reads into (pc, round, wnaf_slice) must match the set of writes from the point_table columns
     *      3. The set of writes into (pc, round, wnaf_slice) from the point table columns is correct
     *      4. `round` only updates when `q_add = 1` at current row and `q_add = 0` at next row
     * If a Prover sets `q_add = 0` when an honest Prover would set `q_add = 1`,
     * this will produce an inequality in the set of reads / writes into the (pc, round, wnaf_slice) table.
     *
     * The addition algorithm has several IF/ELSE statements based on comparing `count` with `msm_size`.
     * Instead of directly constraining these, we define 4 boolean columns `q_add1, q_add2, q_add3, q_add4`.
     * Like `q_add`, their values are Prover-defined. We need to ensure they are set correctly.
     * We update the above conditions on reads into (pc, round, wnaf_slice) to the following:
     *      1. The set of reads into (pc_{count}, round, wnaf_slice_{count}) is constructed when q_add = 1 AND q_add1 =
     * 1
     *      2. The set of reads into (pc_{count + 1}, round, wnaf_slice_{count + 1}) is constructed when q_add = 1 AND
     * q_add2 = 1
     *      3. The set of reads into (pc_{count + 2}, round, wnaf_slice_{count + 2}) is constructed when q_add = 1 AND
     * q_add3 = 1
     *      4. The set of reads into (pc_{count + 3}, round, wnaf_slice_{count + 3}) is constructed when q_add = 1 AND
     * q_add4 = 1
     *
     * To ensure that all q_addi values are correctly set we apply consistency checks to q_add1/q_add2/q_add3/q_add4:
     * 1. If q_add2 = 1, require q_add1 = 1
     * 2. If q_add3 = 1, require q_add2 = 1
     * 3. If q_add4 = 1, require q_add3 = 1
     * 4. If q_add1_shift = 1 AND round does not update between rows, require q_add4 = 1
     *
     * We want to use all of the above to reason about the set of reads into (pc, round, wnaf_slice).
     * The goal is to conclude that any case where the Prover incorrectly sets q_add/q_add1/q_add2/q_add3/q_add4 will
     * produce a set inequality between the reads/writes into (pc, round, wnaf_slice)
     */

    /**
     * @brief Addition relation
     *
     * All addition operations in ECCVMMSMRelationImpl are conditional additions!
     * This method returns two Accumulators that represent x/y coord of output.
     * Output is either an addition of inputs, or xa/ya dpeending on value of `selector`.
     * Additionally, we require `lambda = 0` if `selector = 0`.
     * The `collision_relation` accumulator tracks a subrelation that validates xb != xa.
     * Repeated calls to this method will increase the max degree of the Accumulator output
     * Degree of x_out, y_out = max degree of x_a/x_b + 1
     * 4 Iterations will produce an output degree of 6
     */
    auto add = [&](auto& xb,
                   auto& yb,
                   auto& xa,
                   auto& ya,
                   auto& lambda,
                   auto& selector,
                   auto& relation,
                   auto& collision_relation) {
        // (L * (xb - xa) - yb - ya) * s = 0
        // L * (1 - s) = 0
        // (combine) (L * (xb - xa - 1) - yb - ya) * s + L = 0
        relation += selector * (lambda * (xb - xa - 1) - (yb - ya)) + lambda;
        collision_relation += selector * (xb - xa);
        // x3 = L.L + (-xb - xa) * q + (1 - q) xa
        auto x_out = lambda * lambda + (-xb - xa - xa) * selector + xa;

        // y3 = L . (xa - x3) - ya * q + (1 - q) ya
        auto y_out = lambda * (xa - x_out) + (-ya - ya) * selector + ya;
        return std::array<Accumulator, 2>{ x_out, y_out };
    };

    // ADD operations (if row represents ADD round, not SKEW or DOUBLE)
    Accumulator add_relation(0);
    Accumulator x1_collision_relation(0);
    Accumulator x2_collision_relation(0);
    Accumulator x3_collision_relation(0);
    Accumulator x4_collision_relation(0);
    // If msm_transition = 1, we have started a new MSM. We need to treat the current value of [Acc] as the point at
    // infinity!
    auto add_into_accumulator = -msm_transition + 1;
    auto [x_t1, y_t1] = add(acc_x, acc_y, x1, y1, lambda1, add_into_accumulator, add_relation, x1_collision_relation);
    auto [x_t2, y_t2] = add(x2, y2, x_t1, y_t1, lambda2, add2, add_relation, x2_collision_relation);
    auto [x_t3, y_t3] = add(x3, y3, x_t2, y_t2, lambda3, add3, add_relation, x3_collision_relation);
    auto [x_t4, y_t4] = add(x4, y4, x_t3, y_t3, lambda4, add4, add_relation, x4_collision_relation);

    // Validate accumulator output matches ADD output if q_add = 1
    // (this is a degree-6 relation)
    std::get<0>(accumulator) += q_add * (acc_x_shift - x_t4) * scaling_factor;
    std::get<1>(accumulator) += q_add * (acc_y_shift - y_t4) * scaling_factor;
    std::get<2>(accumulator) += q_add * add_relation * scaling_factor;

    /**
     * @brief doubles a point.
     *
     * Degree of x_out = 2
     * Degree of y_out = 3
     * Degree of relation = 4
     */
    auto dbl = [&](auto& x, auto& y, auto& lambda, auto& relation) {
        auto two_x = x + x;
        relation += lambda * (y + y) - (two_x + x) * x;
        auto x_out = lambda * lambda - two_x;
        auto y_out = lambda * (x - x_out) - y;
        return std::array<Accumulator, 2>{ x_out, y_out };
    };

    /**
     * @brief
     *
     * Algorithm to determine if round is a DOUBLE round:
     *    1. count_shift >= msm_size
     *    2. round != 32
     *
     * Algorithm to process MSM DOUBLE round:
     * [Acc_shift] = (([Acc].double()).double()).double()
     *
     * As with additions, the column q_double describes whether row is a double round. It is Prover-defined.
     * The value of `msm_round` can only update when `q_double = 1` and we use this to ensure Prover correctly sets
     * `q_double`. (see round transition relations further down)
     */
    Accumulator double_relation(0);
    auto [x_d1, y_d1] = dbl(acc_x, acc_y, lambda1, double_relation);
    auto [x_d2, y_d2] = dbl(x_d1, y_d1, lambda2, double_relation);
    auto [x_d3, y_d3] = dbl(x_d2, y_d2, lambda3, double_relation);
    auto [x_d4, y_d4] = dbl(x_d3, y_d3, lambda4, double_relation);
    std::get<10>(accumulator) += q_double * (acc_x_shift - x_d4) * scaling_factor;
    std::get<11>(accumulator) += q_double * (acc_y_shift - y_d4) * scaling_factor;
    std::get<12>(accumulator) += q_double * double_relation * scaling_factor;

    /**
     * @brief SKEW operations
     * When computing x * [P], if x is even we must subtract [P] from accumulator
     * (this is because our windowed non-adjacent-form can only represent odd numbers)
     * Round 32 represents "skew" round.
     * If scalar slice == 7, we add into accumulator (point_table[7] maps to -[P])
     * If scalar slice == 0, we do not add into accumulator
     * i.e. for the skew round we can use the slice values as our "selector" when doing conditional point adds
     */
    Accumulator skew_relation(0);
    static constexpr FF inverse_seven = FF(7).invert();
    auto skew1_select = slice1 * inverse_seven;
    auto skew2_select = slice2 * inverse_seven;
    auto skew3_select = slice3 * inverse_seven;
    auto skew4_select = slice4 * inverse_seven;
    Accumulator x1_skew_collision_relation(0);
    Accumulator x2_skew_collision_relation(0);
    Accumulator x3_skew_collision_relation(0);
    Accumulator x4_skew_collision_relation(0);
    // add skew points iff row is a SKEW row AND slice = 7 (point_table[7] maps to -[P])
    // N.B. while it would be nice to have one `add` relation for both ADD and SKEW rounds,
    // this would increase degree of sumcheck identity vs evaluating them separately.
    // This is because, for add rounds, the result of adding [P1], [Acc] is [P1 + Acc] or [P1]
    //             but for skew rounds, the result of adding [P1], [Acc] is [P1 + Acc] or [Acc]
    auto [x_s1, y_s1] = add(x1, y1, acc_x, acc_y, lambda1, skew1_select, skew_relation, x1_skew_collision_relation);
    auto [x_s2, y_s2] = add(x2, y2, x_s1, y_s1, lambda2, skew2_select, skew_relation, x2_skew_collision_relation);
    auto [x_s3, y_s3] = add(x3, y3, x_s2, y_s2, lambda3, skew3_select, skew_relation, x3_skew_collision_relation);
    auto [x_s4, y_s4] = add(x4, y4, x_s3, y_s3, lambda4, skew4_select, skew_relation, x4_skew_collision_relation);

    // Validate accumulator output matches SKEW output if q_skew = 1
    // (this is a degree-6 relation)
    std::get<3>(accumulator) += q_skew * (acc_x_shift - x_s4) * scaling_factor;
    std::get<4>(accumulator) += q_skew * (acc_y_shift - y_s4) * scaling_factor;
    std::get<5>(accumulator) += q_skew * skew_relation * scaling_factor;

    // Check x-coordinates do not collide if row is an ADD row or a SKEW row
    // if either q_add or q_skew = 1, an inverse should exist for each computed relation
    // Step 1: construct boolean selectors that describe whether we added a point at the current row
    const auto add_first_point = add_into_accumulator * q_add + q_skew * skew1_select;
    const auto add_second_point = add2 * q_add + q_skew * skew2_select;
    const auto add_third_point = add3 * q_add + q_skew * skew3_select;
    const auto add_fourth_point = add4 * q_add + q_skew * skew4_select;
    // Step 2: construct the delta between x-coordinates for each point add (depending on if row is ADD or SKEW)
    const auto x1_delta = x1_skew_collision_relation * q_skew + x1_collision_relation * q_add;
    const auto x2_delta = x2_skew_collision_relation * q_skew + x2_collision_relation * q_add;
    const auto x3_delta = x3_skew_collision_relation * q_skew + x3_collision_relation * q_add;
    const auto x4_delta = x4_skew_collision_relation * q_skew + x4_collision_relation * q_add;
    // Step 3: x_delta * inverse - 1 = 0 if we performed a point addition (else x_delta * inverse = 0)
    std::get<6>(accumulator) += (x1_delta * collision_inverse1 - add_first_point) * scaling_factor;
    std::get<7>(accumulator) += (x2_delta * collision_inverse2 - add_second_point) * scaling_factor;
    std::get<8>(accumulator) += (x3_delta * collision_inverse3 - add_third_point) * scaling_factor;
    std::get<9>(accumulator) += (x4_delta * collision_inverse4 - add_fourth_point) * scaling_factor;

    // Validate that if q_add = 1 or q_skew = 1, add1 also is 1
    // TODO(@zac-williamson) Once we have a stable base to work off of, remove q_add1 and replace with q_msm_add +
    // q_msm_skew (issue #2222)
    std::get<32>(accumulator) += (add1 - q_add - q_skew) * scaling_factor;

    // If add_i = 0, slice_i = 0
    // When add_i = 0, force slice_i to ALSO be 0
    std::get<13>(accumulator) += (-add1 + 1) * slice1 * scaling_factor;
    std::get<14>(accumulator) += (-add2 + 1) * slice2 * scaling_factor;
    std::get<15>(accumulator) += (-add3 + 1) * slice3 * scaling_factor;
    std::get<16>(accumulator) += (-add4 + 1) * slice4 * scaling_factor;

    // only one of q_skew, q_double, q_add can be nonzero
    std::get<17>(accumulator) += (q_add * q_double + q_add * q_skew + q_double * q_skew) * scaling_factor;

    // We look up wnaf slices by mapping round + pc -> slice
    // We use an exact set membership check to validate that
    // wnafs written in wnaf_relation == wnafs read in msm relation
    // We use `add1/add2/add3/add4` to flag whether we are performing a wnaf read op
    // We can set these to be Prover-defined as the set membership check implicitly ensures that the correct reads
    // have occurred.
    // if msm_transition = 0, round_shift - round = 0 or 1
    const auto round_delta = round_shift - round;

    // ROUND TRANSITION LOGIC (when round does not change)
    // If msm_transition = 0 (next row) then round_delta = 0 or 1
    const auto round_transition = round_delta * (-msm_transition_shift + 1);
    std::get<18>(accumulator) += round_transition * (round_delta - 1) * scaling_factor;

    // ROUND TRANSITION LOGIC (when round DOES change)
    // round_transition describes whether we are transitioning between rounds of an MSM
    // If round_transition = 1, the next row is either a double (if round != 31) or we are adding skew (if round ==
    // 31) round_transition * skew * (round - 31) = 0 (if round tx and skew, round == 31) round_transition * (skew +
    // double - 1) = 0 (if round tx, skew XOR double = 1) i.e. if round tx and round != 31, double = 1
    std::get<19>(accumulator) += round_transition * q_skew_shift * (round - 31) * scaling_factor;
    std::get<20>(accumulator) += round_transition * (q_skew_shift + q_double_shift - 1) * scaling_factor;

    // if no double or no skew, round_delta = 0
    std::get<21>(accumulator) += round_transition * (-q_double_shift + 1) * (-q_skew_shift + 1) * scaling_factor;

    // if double, next double != 1
    std::get<22>(accumulator) += q_double * q_double_shift * scaling_factor;

    // if double, next add = 1
    std::get<23>(accumulator) += q_double * (-q_add_shift + 1) * scaling_factor;

    // updating count
    // if msm_transition = 0 and round_transition = 0, count_shift = count + add1 + add2 + add3 + add4
    // todo: we need this?
    std::get<24>(accumulator) += (-msm_transition_shift + 1) * (-round_delta + 1) *
                                 (count_shift - count - add1 - add2 - add3 - add4) * scaling_factor;

    std::get<25>(accumulator) +=
        is_not_first_row * (-msm_transition_shift + 1) * round_delta * count_shift * scaling_factor;

    // if msm_transition = 1, count_shift = 0
    std::get<26>(accumulator) += is_not_first_row * msm_transition_shift * count_shift * scaling_factor;

    // if msm_transition = 1, pc = pc_shift + msm_size
    // `ecc_set_relation` ensures `msm_size` maps to `transcript.msm_count` for the current value of `pc`
    std::get<27>(accumulator) += is_not_first_row * msm_transition_shift * (msm_size + pc_shift - pc) * scaling_factor;

    // Addition continuity checks
    // We want to RULE OUT the following scenarios:
    // Case 1: add2 = 1, add1 = 0
    // Case 2: add3 = 1, add2 = 0
    // Case 3: add4 = 1, add3 = 0
    // These checks ensure that the current row does not skip points (for both ADD and SKEW ops)
    // This is part of a wider set of checks we use to ensure that all point data is used in the assigned
    // multiscalar multiplication operation.
    // (and not in a different MSM operation)
    std::get<28>(accumulator) += add2 * (-add1 + 1) * scaling_factor;
    std::get<29>(accumulator) += add3 * (-add2 + 1) * scaling_factor;
    std::get<30>(accumulator) += add4 * (-add3 + 1) * scaling_factor;

    // Final continuity check.
    // If an addition spans two rows, we need to make sure that the following scenario is RULED OUT:
    //   add4 = 0 on the CURRENT row, add1 = 1 on the NEXT row
    // We must apply the above for the two cases:
    // Case 1: q_add = 1 on the CURRENT row, q_add = 1 on the NEXT row
    // Case 2: q_skew = 1 on the CURRENT row, q_skew = 1 on the NEXT row
    // (i.e. if q_skew = 1, q_add_shift = 1 this implies an MSM transition so we skip this continuity check)
    std::get<31>(accumulator) +=
        (q_add * q_add_shift + q_skew * q_skew_shift) * (-add4 + 1) * add1_shift * scaling_factor;

    // remaining checks (done in ecc_set_relation.hpp, ecc_lookup_relation.hpp)
    // when transition occurs, perform set membership lookup on (accumulator / pc / msm_size)
    // perform set membership lookups on add_i * (pc / round / slice_i)
    // perform lookups on (pc / slice_i / x / y)
}

template class ECCVMMSMRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationImpl, flavor::ECCVM);

} // namespace bb::honk::sumcheck
