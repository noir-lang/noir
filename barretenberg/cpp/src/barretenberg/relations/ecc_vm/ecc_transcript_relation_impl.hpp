#pragma once
#include <array>
#include <tuple>

#include "./ecc_transcript_relation.hpp"
#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb {
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

    static const auto offset_generator = [&]() {
        static constexpr auto offset_generator_base = bb::g1::derive_generators("ECCVM_OFFSET_GENERATOR", 1)[0];
        static bb::g1::affine_element result =
            bb::g1::affine_element(bb::g1::element(offset_generator_base) * grumpkin::fq(uint256_t(1) << 124));
        static const FF qx = result.x;
        static const FF qy = result.y;
        static const Accumulator ox(qx);
        static const Accumulator oy(qy);
        return std::array<Accumulator, 2>{ ox, oy };
    };
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
    auto transcript_msm_x = View(in.transcript_msm_intermediate_x);
    auto transcript_msm_y = View(in.transcript_msm_intermediate_y);
    auto transcript_Px = View(in.transcript_Px);
    auto transcript_Py = View(in.transcript_Py);
    auto is_accumulator_empty = View(in.transcript_accumulator_empty);
    auto lagrange_first = View(in.lagrange_first);
    auto lagrange_last = View(in.lagrange_last);
    auto is_accumulator_empty_shift = View(in.transcript_accumulator_empty_shift);
    auto q_reset_accumulator = View(in.transcript_reset_accumulator);
    auto lagrange_second = View(in.lagrange_second);
    auto transcript_Pinfinity = View(in.transcript_base_infinity);
    auto transcript_Px_inverse = View(in.transcript_base_x_inverse);
    auto transcript_Py_inverse = View(in.transcript_base_y_inverse);
    auto transcript_add_x_equal = View(in.transcript_add_x_equal);
    auto transcript_add_y_equal = View(in.transcript_add_y_equal);
    auto transcript_add_lambda = View(in.transcript_add_lambda);
    auto transcript_msm_infinity = View(in.transcript_msm_infinity);

    auto is_not_first_row = (-lagrange_first + 1);
    auto is_not_last_row = (-lagrange_last + 1);
    auto is_not_first_or_last_row = (-lagrange_first + -lagrange_last + 1);
    auto is_not_infinity = (-transcript_Pinfinity + 1);
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
    std::get<0>(accumulator) += (z1 * z1_zero) * scaling_factor; // if z1_zero = 1, z1 must be 0. degree 2
    std::get<1>(accumulator) += (z2 * z2_zero) * scaling_factor; // degree 2

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
    std::get<2>(accumulator) += (tmp - op) * scaling_factor; // degree 1

    /**
     * @brief Validate `pc` is updated correctly.
     * pc stands for Point Counter. It decrements by 1 for every 128-bit multiplication operation.
     * If q_mul = 1, pc decrements by !z1_zero + !z2_zero, else pc decrements by 0
     * @note pc starts out at its max value and decrements down to 0. This keeps the degree of the pc polynomial smol
     */
    Accumulator pc_delta = pc - pc_shift;
    auto num_muls_in_row = ((-z1_zero + 1) + (-z2_zero + 1)) * (-transcript_Pinfinity + 1);
    std::get<3>(accumulator) += is_not_first_row * (pc_delta - q_mul * num_muls_in_row) * scaling_factor; // degree 4

    /**
     * @brief Validate `msm_transition` is well-formed.
     *
     * If the current row is the last mul instruction in a multiscalar multiplication, msm_transition = 1.
     * i.e. if q_mul == 1 and q_mul_shift == 0, msm_transition = 1, else is 0
     * We also require that `msm_count + [current msm number] > 0`
     */
    auto msm_transition_check = q_mul * (-q_mul_shift + 1); // degree 2
    // auto num_muls_total = msm_count + num_muls_in_row;
    auto msm_count_zero_at_transition = View(in.transcript_msm_count_zero_at_transition);
    auto msm_count_at_transition_inverse = View(in.transcript_msm_count_at_transition_inverse);

    auto msm_count_total = msm_count + num_muls_in_row; // degree 3

    auto msm_count_zero_at_transition_check = msm_count_zero_at_transition * msm_count_total;
    msm_count_zero_at_transition_check +=
        (msm_count_total * msm_count_at_transition_inverse - 1) * (-msm_count_zero_at_transition + 1);
    std::get<4>(accumulator) += msm_transition_check * msm_count_zero_at_transition_check * scaling_factor; // degree 3

    // Validate msm_transition_msm_count is correct
    // ensure msm_transition is zero if count is zero
    std::get<5>(accumulator) +=
        (msm_transition - msm_transition_check * (-msm_count_zero_at_transition + 1)) * scaling_factor; // degree 3

    /**
     * @brief Validate `msm_count` resets when we end a multiscalar multiplication.
     * msm_count tracks the number of scalar muls in the current active multiscalar multiplication.
     * (if no msm active, msm_count == 0)
     * If current row ends an MSM, `msm_count_shift = 0` (msm_count value at next row)
     */
    std::get<6>(accumulator) += (msm_transition * msm_count_shift) * scaling_factor; // degree 2

    /**
     * @brief Validate `msm_count` updates correctly for mul operations.
     * msm_count updates by (!z1_zero + !z2_zero) if current op is a mul instruction (and msm is not terminating at next
     * row).
     */
    auto msm_count_delta = msm_count_shift - msm_count; // degree 4
    auto num_counts = ((-z1_zero + 1) + (-z2_zero + 1)) * (-transcript_Pinfinity + 1);
    std::get<7>(accumulator) +=
        is_not_first_row * (-msm_transition + 1) * (msm_count_delta - q_mul * (num_counts)) * scaling_factor;

    /**
     * @brief Opcode exclusion tests. We have the following assertions:
     * 1. If q_mul = 1, (q_add, eq, reset) are zero
     * 2. If q_add = 1, (q_mul, eq, reset) are zero
     * 3. If q_eq =  1, (q_add, q_mul) are zero (is established by previous 2 checks)
     */
    auto opcode_exclusion_relation = q_mul * (q_add + q_eq + q_reset_accumulator);
    opcode_exclusion_relation += q_add * (q_mul + q_eq + q_reset_accumulator);
    std::get<8>(accumulator) += opcode_exclusion_relation * scaling_factor; // degree 2

    /**
     * @brief `eq` opcode.
     * Let lhs = transcript_P and rhs = transcript_accumulator
     * If eq = 1, we must validate the following cases:
     * IF lhs and rhs are not at infinity THEN lhs == rhs
     * ELSE lhs and rhs are BOTH points at infinity
     **/
    auto both_infinity = transcript_Pinfinity * is_accumulator_empty;
    auto both_not_infinity = (-transcript_Pinfinity + 1) * (-is_accumulator_empty + 1);
    auto infinity_exclusion_check = transcript_Pinfinity + is_accumulator_empty - both_infinity - both_infinity;
    auto eq_x_diff = transcript_Px - transcript_accumulator_x;
    auto eq_y_diff = transcript_Py - transcript_accumulator_y;
    auto eq_x_diff_relation = q_eq * (eq_x_diff * both_not_infinity + infinity_exclusion_check); // degree 4
    auto eq_y_diff_relation = q_eq * (eq_y_diff * both_not_infinity + infinity_exclusion_check); // degree 4
    std::get<9>(accumulator) += eq_x_diff_relation * scaling_factor;                             // degree 4
    std::get<10>(accumulator) += eq_y_diff_relation * scaling_factor;                            // degree 4

    /**
     * @brief Initial condition check on 1st row.
     * We require the following values are 0 on 1st row:
     * is_accumulator_empty = 1
     * msm_count = 0
     * note...actually second row? bleurgh
     * NOTE: we want pc = 0 at lagrange_last :o
     */
    std::get<11>(accumulator) += lagrange_second * (-is_accumulator_empty + 1) * scaling_factor; // degree 2
    std::get<12>(accumulator) += lagrange_second * msm_count * scaling_factor;                   // degree 2

    /**
     * @brief On-curve validation checks.
     * If q_mul = 1 OR q_add = 1 OR q_eq = 1, require (transcript_Px, transcript_Py) is valid ecc point
     * q_mul/q_add/q_eq mutually exclusive, can represent as sum of 3
     */
    const auto validate_on_curve = q_mul + q_add + q_mul + q_eq;
    const auto on_curve_check =
        transcript_Py * transcript_Py - transcript_Px * transcript_Px * transcript_Px - get_curve_b();
    std::get<13>(accumulator) += validate_on_curve * on_curve_check * is_not_infinity * scaling_factor; // degree 6

    /**
     * @brief Validate relations from ECC Group Operations are well formed
     *
     */
    {
        Accumulator transcript_lambda_relation(0);
        auto is_double = transcript_add_x_equal * transcript_add_y_equal;
        auto is_add = (-transcript_add_x_equal + 1);
        auto add_result_is_infinity = transcript_add_x_equal * (-transcript_add_y_equal + 1); // degree 2
        auto rhs_x = transcript_accumulator_x;
        auto rhs_y = transcript_accumulator_y;
        auto out_x = transcript_accumulator_x_shift;
        auto out_y = transcript_accumulator_y_shift;
        auto lambda = transcript_add_lambda;
        auto lhs_x = transcript_Px * q_add + transcript_msm_x * msm_transition;
        auto lhs_y = transcript_Py * q_add + transcript_msm_y * msm_transition;
        auto lhs_infinity = transcript_Pinfinity * q_add + transcript_msm_infinity * msm_transition;
        auto rhs_infinity = is_accumulator_empty;
        auto result_is_lhs = rhs_infinity * (-lhs_infinity + 1);                                      // degree 2
        auto result_is_rhs = (-rhs_infinity + 1) * lhs_infinity;                                      // degree 2
        auto result_infinity_from_inputs = lhs_infinity * rhs_infinity;                               // degree 2
        auto result_infinity_from_operation = transcript_add_x_equal * (-transcript_add_y_equal + 1); // degree 2
        // infinity_from_inputs and infinity_from_operation mutually exclusive so we can perform an OR by adding
        // (mutually exclusive because if result_infinity_from_inputs then transcript_add_y_equal = 1 (both y are 0)
        auto result_is_infinity = result_infinity_from_inputs + result_infinity_from_operation; // degree 2
        auto any_add_is_active = q_add + msm_transition;

        // Valdiate `transcript_add_lambda` is well formed if we are adding msm output into accumulator
        {
            Accumulator transcript_msm_lambda_relation(0);
            auto msm_x = transcript_msm_x;
            auto msm_y = transcript_msm_y;
            // Group operation is point addition
            {
                auto lambda_denominator = (rhs_x - msm_x);
                auto lambda_numerator = (rhs_y - msm_y);
                auto lambda_relation = lambda * lambda_denominator - lambda_numerator; // degree 2
                transcript_msm_lambda_relation += lambda_relation * is_add;            // degree 3
            }
            // Group operation is point doubling
            {
                auto lambda_denominator = msm_y + msm_y;
                auto lambda_numerator = msm_x * msm_x * 3;
                auto lambda_relation = lambda * lambda_denominator - lambda_numerator; // degree 2
                transcript_msm_lambda_relation += lambda_relation * is_double;         // degree 4
            }
            auto transcript_add_or_dbl_from_msm_output_is_valid =
                (-transcript_msm_infinity + 1) * (-is_accumulator_empty + 1);                 // degree 2
            transcript_msm_lambda_relation *= transcript_add_or_dbl_from_msm_output_is_valid; // degree 6
            // No group operation because of points at infinity
            {
                auto lambda_relation_invalid =
                    (transcript_msm_infinity + is_accumulator_empty + add_result_is_infinity); // degree 2
                auto lambda_relation = lambda * lambda_relation_invalid;                       // degree 4
                transcript_msm_lambda_relation += lambda_relation;                             // (still degree 6)
            }
            transcript_lambda_relation = transcript_msm_lambda_relation * msm_transition; // degree 7
        }
        // Valdiate `transcript_add_lambda` is well formed if we are adding base point into accumulator
        {
            Accumulator transcript_add_lambda_relation(0);
            auto add_x = transcript_Px;
            auto add_y = transcript_Py;
            // Group operation is point addition
            {
                auto lambda_denominator = (rhs_x - add_x);
                auto lambda_numerator = (rhs_y - add_y);
                auto lambda_relation = lambda * lambda_denominator - lambda_numerator; // degree 2
                transcript_add_lambda_relation += lambda_relation * is_add;            // degree 3
            }
            // Group operation is point doubling
            {
                auto lambda_denominator = add_y + add_y;
                auto lambda_numerator = add_x * add_x * 3;
                auto lambda_relation = lambda * lambda_denominator - lambda_numerator; // degree 2
                transcript_add_lambda_relation += lambda_relation * is_double;         // degree 4
            }
            auto transcript_add_or_dbl_from_add_output_is_valid =
                (-transcript_Pinfinity + 1) * (-is_accumulator_empty + 1);                    // degree 2
            transcript_add_lambda_relation *= transcript_add_or_dbl_from_add_output_is_valid; // degree 6
            // No group operation because of points at infinity
            {
                auto lambda_relation_invalid =
                    (transcript_Pinfinity + is_accumulator_empty + add_result_is_infinity); // degree 2
                auto lambda_relation = lambda * lambda_relation_invalid;                    // degree 4
                transcript_add_lambda_relation += lambda_relation;                          // (still degree 6)
            }
            transcript_lambda_relation += transcript_add_lambda_relation * q_add;
            std::get<14>(accumulator) += transcript_lambda_relation * scaling_factor; // degree 7
        }
        /**
         * @brief Validate transcript_accumulator_x_shift / transcript_accumulator_y_shift are well formed.
         *        Conditions (one of the following):
         *        1. The result of a group operation involving transcript_accumulator and msm_output (q_add = 1)
         *        2. The result of a group operation involving transcript_accumulator and transcript_P (msm_transition =
         * 1)
         *        3. Is equal to transcript_accumulator (no group operation, no reset)
         *        4. Is 0 (reset)
         */
        {
            auto lambda_sqr = lambda * lambda;
            // add relation that validates result_infinity_from_operation * result_is_infinity = 0

            // N.B. these relations rely on the fact that `lambda = 0` if we are not evaluating add/double formula
            // (i.e. one or both outputs are points at infinity, or produce a point at infinity)
            // This should be validated by the lambda_relation
            auto x3 = lambda_sqr - lhs_x - rhs_x;          // degree 2
            x3 += result_is_lhs * (rhs_x + lhs_x + lhs_x); // degree 4
            x3 += result_is_rhs * (lhs_x + rhs_x + rhs_x); // degree 4
            x3 += result_is_infinity * (lhs_x + rhs_x);    // degree 4
            auto y3 = lambda * (lhs_x - out_x) - lhs_y;    // degree 3
            y3 += result_is_lhs * (lhs_y + lhs_y);         // degree 4
            y3 += result_is_rhs * (lhs_y + rhs_y);         // degree 4
            y3 += result_is_infinity * lhs_y;              // degree 4

            auto propagate_transcript_accumulator = (-q_add - msm_transition - q_reset_accumulator + 1);
            auto add_point_x_relation = (x3 - out_x) * any_add_is_active; // degree 5
            add_point_x_relation +=
                propagate_transcript_accumulator * is_not_last_row * (out_x - transcript_accumulator_x);
            // validate out_x = 0 if q_reset_accumulator = 1
            add_point_x_relation += (out_x * q_reset_accumulator);
            auto add_point_y_relation = (y3 - out_y) * any_add_is_active; // degree 5
            add_point_y_relation +=
                propagate_transcript_accumulator * is_not_last_row * (out_y - transcript_accumulator_y);
            // validate out_y = 0 if q_reset_accumulator = 1
            add_point_y_relation += (out_y * q_reset_accumulator);
            std::get<15>(accumulator) += add_point_x_relation * scaling_factor; // degree 5
            std::get<16>(accumulator) += add_point_y_relation * scaling_factor; // degree 5
        }

        // step 1: subtract offset generator from msm_accumulator
        // this might produce a point at infinity
        {
            const auto offset = offset_generator();
            const auto x1 = offset[0];
            const auto y1 = -offset[1];
            const auto x2 = View(in.transcript_msm_x);
            const auto y2 = View(in.transcript_msm_y);
            const auto x3 = View(in.transcript_msm_intermediate_x);
            const auto y3 = View(in.transcript_msm_intermediate_y);
            const auto transcript_msm_infinity = View(in.transcript_msm_infinity);
            // cases:
            // x2 == x1, y2 == y1
            // x2 != x1
            // (x2 - x1)
            const auto x_term = (x3 + x2 + x1) * (x2 - x1) * (x2 - x1) - (y2 - y1) * (y2 - y1); // degree 3
            const auto y_term = (x1 - x3) * (y2 - y1) - (x2 - x1) * (y1 + y3);                  // degree 2
            // IF msm_infinity = false, transcript_msm_intermediate_x/y is either the result of subtracting offset
            // generator from msm_x/y IF msm_infinity = true, transcript_msm_intermediate_x/y is 0
            const auto transcript_offset_generator_subtract_x =
                x_term * (-transcript_msm_infinity + 1) + transcript_msm_infinity * x3; // degree 4
            const auto transcript_offset_generator_subtract_y =
                y_term * (-transcript_msm_infinity + 1) + transcript_msm_infinity * y3; // degree 3
            std::get<17>(accumulator) +=
                msm_transition * transcript_offset_generator_subtract_x * scaling_factor; // degree 5
            std::get<18>(accumulator) +=
                msm_transition * transcript_offset_generator_subtract_y * scaling_factor; // degree 5

            // validate transcript_msm_infinity is correct
            // if transcript_msm_infinity = 1, (x2 == x1) and (y2 + y1 == 0)
            const auto x_diff = x2 - x1;
            const auto y_sum = y2 + y1;
            std::get<19>(accumulator) += msm_transition * transcript_msm_infinity * x_diff * scaling_factor; // degree 3
            std::get<20>(accumulator) += msm_transition * transcript_msm_infinity * y_sum * scaling_factor;  // degree 3
            // if transcript_msm_infinity = 1, then x_diff must have an inverse
            const auto transcript_msm_x_inverse = View(in.transcript_msm_x_inverse);
            const auto inverse_term = (-transcript_msm_infinity + 1) * (x_diff * transcript_msm_x_inverse - 1);
            std::get<21>(accumulator) += msm_transition * inverse_term * scaling_factor; // degree 3
        }

        /**
         * @brief Validate `is_accumulator_empty` is updated correctly
         * An add operation can produce a point at infinity
         * Resetting the accumulator produces a point at infinity
         * If we are not adding, performing an msm or resetting the accumulator, is_accumulator_empty should not update
         */
        auto accumulator_infinity_preserve_flag = (-(q_add + msm_transition + q_reset_accumulator) + 1); // degree 1
        auto accumulator_infinity_preserve = accumulator_infinity_preserve_flag *
                                             (is_accumulator_empty - is_accumulator_empty_shift) *
                                             is_not_first_or_last_row;                               // degree 3
        auto accumulator_infinity_q_reset = q_reset_accumulator * (-is_accumulator_empty_shift + 1); // degree 2
        auto accumulator_infinity_from_add =
            any_add_is_active * (result_is_infinity - is_accumulator_empty_shift); // degree 3
        auto accumulator_infinity_relation =
            accumulator_infinity_preserve +
            (accumulator_infinity_q_reset + accumulator_infinity_from_add) * is_not_first_row; // degree 4
        std::get<22>(accumulator) += accumulator_infinity_relation * scaling_factor;           // degree 4

        /**
         * @brief Validate `transcript_add_x_equal` is well-formed
         *        If lhs_x == rhs_x, transcript_add_x_equal = 1
         *        If transcript_add_x_equal = 0, a valid inverse must exist for (lhs_x - rhs_x)
         */
        auto x_diff = lhs_x - rhs_x;                                                                        // degree 2
        auto x_product = transcript_Px_inverse * (-transcript_add_x_equal + 1) + transcript_add_x_equal;    // degree 2
        auto x_constant = transcript_add_x_equal - 1;                                                       // degree 1
        auto transcript_add_x_equal_check_relation = (x_diff * x_product + x_constant) * any_add_is_active; // degree 5
        std::get<23>(accumulator) += transcript_add_x_equal_check_relation * scaling_factor;                // degree 5

        /**
         * @brief Validate `transcript_add_y_equal` is well-formed
         *        If lhs_y == rhs_y, transcript_add_y_equal = 1
         *        If transcript_add_y_equal = 0, a valid inverse must exist for (lhs_y - rhs_y)
         */
        auto y_diff = lhs_y - rhs_y;
        auto y_product = transcript_Py_inverse * (-transcript_add_y_equal + 1) + transcript_add_y_equal;
        auto y_constant = transcript_add_y_equal - 1;
        auto transcript_add_y_equal_check_relation = (y_diff * y_product + y_constant) * any_add_is_active;
        std::get<24>(accumulator) += transcript_add_y_equal_check_relation * scaling_factor; // degree 5
    }
}
} // namespace bb
