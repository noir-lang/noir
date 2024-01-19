#include <array>
#include <tuple>

#include "./ecc_transcript_relation.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief ECCVMTranscriptRelationImpl evaluates the correctness of the ECCVM transcript columns
 *
 * @details The transcript relations directly evaluate the correctness of `add, eq, reset` operations.
 * `mul` operations are lazily evaluated. The output of multiscalar multiplications is present in
 * `transcript_msm_x, transcript_msm_y` columns. A set equality check is used to validate these
 * have been correctly read from a table produced by the relations in `ecc_msm_relation.hpp`.
 *
 * Sequential `mul` opcodes are interpreted as a multiscalar multiplication.
 * The column `transcript_msm_count` tracks the number of muls in a given multiscalar multiplication.
 *
 * The column `transcript_pc` tracks a "point counter" value, that describes the number of multiplications
 * that must be evaluated.
 *
 * One mul opcode can generate up to TWO multiplications. Each 128-bit scalar `z1, z2` is treated as an independent mul.
 * The purpose of this is to reduce the length of the MSM algorithm evalauted in `ecc_msm_relation.hpp` to 128 bits
 * (from 256 bits). Many scalar muls required to recursively verify a proof are only 128-bits in length; this prevents
 * us doing redundant computation.
 * @tparam FF
 * @tparam AccumulatorTypes
 * @tparam PolynomialTypes
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMTranscriptRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                                 const AllEntities& in,
                                                 const Parameters& /*unused*/,
                                                 const FF& scaling_factor)
{
    using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    auto z1 = View(in.transcript_z1);
    auto z2 = View(in.transcript_z2);
    auto z1_zero = View(in.transcript_z1zero);
    auto z2_zero = View(in.transcript_z2zero);
    auto op = View(in.transcript_op);
    auto q_add = View(in.transcript_add);
    auto q_mul = View(in.transcript_mul);
    auto q_mul_shift = View(in.transcript_mul_shift);
    auto q_eq = View(in.transcript_eq);
    auto msm_transition = View(in.transcript_msm_transition);
    auto msm_count = View(in.transcript_msm_count);
    auto msm_count_shift = View(in.transcript_msm_count_shift);
    auto pc = View(in.transcript_pc);
    auto pc_shift = View(in.transcript_pc_shift);
    auto transcript_accumulator_x_shift = View(in.transcript_accumulator_x_shift);
    auto transcript_accumulator_y_shift = View(in.transcript_accumulator_y_shift);
    auto transcript_accumulator_x = View(in.transcript_accumulator_x);
    auto transcript_accumulator_y = View(in.transcript_accumulator_y);
    auto transcript_msm_x = View(in.transcript_msm_x);
    auto transcript_msm_y = View(in.transcript_msm_y);
    auto transcript_Px = View(in.transcript_Px);
    auto transcript_Py = View(in.transcript_Py);
    auto is_accumulator_empty = View(in.transcript_accumulator_empty);
    auto lagrange_first = View(in.lagrange_first);
    auto lagrange_last = View(in.lagrange_last);
    auto is_accumulator_empty_shift = View(in.transcript_accumulator_empty_shift);
    auto q_reset_accumulator = View(in.transcript_reset_accumulator);
    auto lagrange_second = View(in.lagrange_second);
    auto transcript_collision_check = View(in.transcript_collision_check);

    auto is_not_first_row = (-lagrange_first + 1);
    auto is_not_first_or_last_row = (-lagrange_first + -lagrange_last + 1);
    /**
     * @brief Validate correctness of z1_zero, z2_zero.
     * If z1_zero = 0 and operation is a MUL, we will write a scalar mul instruction into our multiplication table.
     * If z1_zero = 1 and operation is a MUL, we will NOT write a scalar mul instruction.
     * (same with z2_zero).
     * z1_zero / z2_zero is user-defined.
     * We constraint z1_zero such that if z1_zero == 1, we require z1 == 0. (same for z2_zero).
     * We do *NOT* constrain z1 != 0 if z1_zero = 0. If the user sets z1_zero = 0 and z1 = 0,
     * this will add a scalar mul instruction into the multiplication table, where the scalar multiplier is 0.
     * This is inefficient but will still produce the correct output.
     */
    std::get<0>(accumulator) += (z1 * z1_zero) * scaling_factor; // if z1_zero = 1, z1 must be 0
    std::get<1>(accumulator) += (z2 * z2_zero) * scaling_factor;

    /**
     * @brief Validate `op` opcode is well formed.
     * `op` is defined to be q_reset_accumulator + 2 * q_eq + 4 * q_mul + 8 * q_add,
     * where q_reset_accumulator, q_eq, q_mul, q_add are all boolean
     * (TODO: bool constrain these efficiently #2223)
     */
    auto tmp = q_add + q_add;
    tmp += q_mul;
    tmp += tmp;
    tmp += q_eq;
    tmp += tmp;
    tmp += q_reset_accumulator;
    std::get<2>(accumulator) += (tmp - op) * scaling_factor;

    /**
     * @brief Validate `pc` is updated correctly.
     * pc stands for Point Counter. It decrements by 1 for every 128-bit multiplication operation.
     * If q_mul = 1, pc decrements by !z1_zero + !z2_zero, else pc decrements by 0
     * @note pc starts out at its max value and decrements down to 0. This keeps the degree of the pc polynomial smol
     */
    Accumulator pc_delta = pc - pc_shift;
    std::get<3>(accumulator) +=
        is_not_first_row * (pc_delta - q_mul * ((-z1_zero + 1) + (-z2_zero + 1))) * scaling_factor;

    /**
     * @brief Validate `msm_transition` is well-formed.
     *
     * If the current row is the last mul instruction in a multiscalar multiplication, msm_transition = 1.
     * i.e. if q_mul == 1 and q_mul_shift == 0, msm_transition = 1, else is 0
     */
    auto msm_transition_check = q_mul * (-q_mul_shift + 1);
    std::get<4>(accumulator) += (msm_transition - msm_transition_check) * scaling_factor;

    /**
     * @brief Validate `msm_count` resets when we end a multiscalar multiplication.
     * msm_count tracks the number of scalar muls in the current active multiscalar multiplication.
     * (if no msm active, msm_count == 0)
     * If current row ends an MSM, `msm_count_shift = 0` (msm_count value at next row)
     */
    std::get<5>(accumulator) += (msm_transition * msm_count_shift) * scaling_factor;

    /**
     * @brief Validate `msm_count` updates correctly for mul operations.
     * msm_count updates by (!z1_zero + !z2_zero) if current op is a mul instruction (and msm is not terminating at next
     * row).
     */
    auto msm_count_delta = msm_count_shift - msm_count; // degree 4
    std::get<6>(accumulator) += is_not_first_row * (-msm_transition + 1) *
                                (msm_count_delta - q_mul * ((-z1_zero + 1) + (-z2_zero + 1))) * scaling_factor;

    /**
     * @brief Add multiscalar multiplication result into transcript accumulator.
     * If `msm_transition == 1`, we expect msm output to be present on (transcript_msm_x, transcript_msm_y).
     * (this is enforced via a lookup protocol).
     * If `is_accumulator_empty == 0`, we ADD msm output into transcript_accumulator.
     * If `is_accumulator_empty = =1`, we ASSIGN msm output to transcript_accumulator.
     * @note the output of an msm cannot be point at infinity (will create unsatisfiable constraints in
     * ecc_msm_relation). We assume this does not affect statistical completeness for honest provers. We should validate
     * this!
     */
    auto add_msm_into_accumulator = msm_transition * (-is_accumulator_empty + 1);
    auto x3 = transcript_accumulator_x_shift;
    auto y3 = transcript_accumulator_y_shift;
    auto x1 = transcript_accumulator_x;
    auto y1 = transcript_accumulator_y;
    auto x2 = transcript_msm_x;
    auto y2 = transcript_msm_y;
    auto tmpx = (x3 + x2 + x1) * (x2 - x1) * (x2 - x1) - (y2 - y1) * (y2 - y1);
    auto tmpy = (y3 + y1) * (x2 - x1) - (y2 - y1) * (x1 - x3);
    std::get<7>(accumulator) += tmpx * add_msm_into_accumulator * scaling_factor; // degree 5
    std::get<8>(accumulator) += tmpy * add_msm_into_accumulator * scaling_factor; // degree 4

    /**
     * @brief If is_accumulator_empty == 1, assign transcript_accumulator output into accumulator
     *
     * @note The accumulator point for all operations at row `i` is the accumulator point at row `i + 1`!
     */
    auto assign_msm_into_accumulator = msm_transition * is_accumulator_empty;
    std::get<9>(accumulator) += assign_msm_into_accumulator * (x3 - x2) * scaling_factor; // degree 3
    std::get<10>(accumulator) += assign_msm_into_accumulator * (y3 - y2) * scaling_factor;

    /**
     * @brief Constrain `add` opcode.
     *
     * add will add the input point in (transcript_Px, transcript_Py) into the accumulator.
     * Correctly handles case where accumulator is point at infinity.
     * TODO: need to add constraints to rule out point doubling case (x2 != x1)
     * TODO: need to assert input point is on the curve!
     */
    x2 = transcript_Px;
    y2 = transcript_Py;
    auto add_into_accumulator = q_add * (-is_accumulator_empty + 1);
    tmpx = (x3 + x2 + x1) * (x2 - x1) * (x2 - x1) - (y2 - y1) * (y2 - y1);
    tmpy = (y3 + y1) * (x2 - x1) - (y2 - y1) * (x1 - x3);
    std::get<11>(accumulator) += tmpx * add_into_accumulator * scaling_factor; // degree 5
    std::get<12>(accumulator) += tmpy * add_into_accumulator * scaling_factor; // degree 4
    auto assign_into_accumulator = q_add * is_accumulator_empty;
    std::get<13>(accumulator) += (x3 - x2) * assign_into_accumulator * scaling_factor; // degree 3
    std::get<14>(accumulator) += (y3 - y2) * assign_into_accumulator * scaling_factor;

    /**
     * @brief Opcode exclusion tests. We have the following assertions:
     * 1. If q_mul = 1, (q_add, eq, reset) are zero
     * 2. If q_reset = 1, is_accumulator_empty at next row = 1
     * 3. If q_add = 1 OR msm_transition = 1, is_accumulator_empty at next row = 0
     * 4. If q_add = 0 AND msm_transition = 0 AND q_reset_accumulator = 0, is_accumulator at next row = current row
     * value
     * @note point 3: both q_add = 1, msm_transition = 1 cannot occur because of point 1 (msm_transition only 1 when
     * q_mul 1) we can use a slightly more efficient relation than a pure binary OR
     */
    std::get<15>(accumulator) += q_mul * (q_add + q_eq + q_reset_accumulator) * scaling_factor;
    std::get<16>(accumulator) += q_add * (q_mul + q_eq + q_reset_accumulator) * scaling_factor;
    std::get<17>(accumulator) += q_reset_accumulator * (-is_accumulator_empty_shift + 1) * scaling_factor;
    std::get<18>(accumulator) += (q_add + msm_transition) * is_accumulator_empty_shift * scaling_factor;
    auto accumulator_state_not_modified = -(q_add + msm_transition + q_reset_accumulator) + 1;
    std::get<19>(accumulator) += accumulator_state_not_modified * is_not_first_or_last_row *
                                 (is_accumulator_empty_shift - is_accumulator_empty) * scaling_factor;

    /**
     * @brief `eq` opcode.
     * If eq = 1, assert transcript_Px/y = transcript_accumulator_x/y.
     * If eq = 1, assert is_accumulator_empty = 0 (input point cannot be point at infinity)
     */
    std::get<20>(accumulator) += q_eq * (transcript_accumulator_x - transcript_Px) * scaling_factor;
    std::get<21>(accumulator) +=
        q_eq * (-is_accumulator_empty + 1) * (transcript_accumulator_y - transcript_Py) * scaling_factor;
    std::get<22>(accumulator) += q_eq * is_accumulator_empty * scaling_factor;

    // validate selectors are boolean (put somewhere else? these are low degree)
    std::get<23>(accumulator) += q_eq * (q_eq - 1) * scaling_factor;
    std::get<24>(accumulator) += q_add * (q_add - 1) * scaling_factor;
    std::get<25>(accumulator) += q_mul * (q_mul - 1) * scaling_factor;
    std::get<26>(accumulator) += q_reset_accumulator * (q_reset_accumulator - 1) * scaling_factor;
    std::get<27>(accumulator) += msm_transition * (msm_transition - 1) * scaling_factor;
    std::get<28>(accumulator) += is_accumulator_empty * (is_accumulator_empty - 1) * scaling_factor;
    std::get<29>(accumulator) += z1_zero * (z1_zero - 1) * scaling_factor;
    std::get<30>(accumulator) += z2_zero * (z2_zero - 1) * scaling_factor;

    /**
     * @brief Initial condition check on 1st row.
     * We require the following values are 0 on 1st row:
     * is_accumulator_empty = 1
     * msm_count = 0
     * note...actually second row? bleurgh
     * NOTE: we want pc = 0 at lagrange_last :o
     */
    std::get<31>(accumulator) += lagrange_second * (-is_accumulator_empty + 1) * scaling_factor;
    std::get<32>(accumulator) += lagrange_second * msm_count * scaling_factor;

    /**
     * @brief On-curve validation checks.
     * If q_mul = 1 OR q_add = 1 OR q_eq = 1, require (transcript_Px, transcript_Py) is valid ecc point
     * q_mul/q_add/q_eq mutually exclusive, can represent as sum of 3
     */
    const auto validate_on_curve = q_mul; // q_add + q_mul + q_eq;
    const auto on_curve_check =
        transcript_Py * transcript_Py - transcript_Px * transcript_Px * transcript_Px - get_curve_b();
    std::get<33>(accumulator) += validate_on_curve * on_curve_check * scaling_factor;

    /**
     * @brief If performing an add, validate x-coordintes of inputs do not collide.
     * If adding msm output into accumulator, validate x-coordinates of inputs do not collide
     */
    auto x_coordinate_collision_check =
        add_msm_into_accumulator * ((transcript_msm_x - transcript_accumulator_x) * transcript_collision_check - FF(1));
    x_coordinate_collision_check +=
        add_into_accumulator * ((transcript_Px - transcript_accumulator_x) * transcript_collision_check - FF(1));
    std::get<34>(accumulator) += x_coordinate_collision_check * scaling_factor;
}

template class ECCVMTranscriptRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationImpl, flavor::ECCVM);

} // namespace bb::honk::sumcheck
