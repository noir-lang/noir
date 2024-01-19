#include "ecc_point_table_relation.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief ECCVMPointTableRelationImpl
 * @details These relations define the set of point lookup tables we will use in `ecc_msm_relation.hpp`, to evaluate
 * multiscalar multiplication. For every point [P] = (Px, Py) involved in an MSM, we need to do define a lookup
 * table out of the following points: { -15[P], -13[P], -11[P], -9[P], -7[P], -5[P], -3[P], -[P] }
 * ECCVMPointTableRelationImpl defines relations that define the lookup table.
 *
 * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMPointTableRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                                 const AllEntities& in,
                                                 const Parameters& /*unused*/,
                                                 const FF& scaling_factor)
{
    using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    const auto& Tx = View(in.precompute_tx);
    const auto& Tx_shift = View(in.precompute_tx_shift);
    const auto& Ty = View(in.precompute_ty);
    const auto& Ty_shift = View(in.precompute_ty_shift);
    const auto& Dx = View(in.precompute_dx);
    const auto& Dx_shift = View(in.precompute_dx_shift);
    const auto& Dy = View(in.precompute_dy);
    const auto& Dy_shift = View(in.precompute_dy_shift);
    const auto& precompute_point_transition = View(in.precompute_point_transition);
    const auto& lagrange_first = View(in.lagrange_first);

    /**
     * @brief Row structure
     *
     * Consider the set of (128-bit scalar multiplier, point, pc) tuples in the transcript columns.
     * The point table columns process one tuple every 8 rows. The tuple with the largest pc value is first.
     * When transitioning between tuple elements, pc decrements by 1.
     *
     * The following table gives an example for two points.
     * In the table, the point associated with `pc = 1` is labelled P.
     *               the point associated with `pc = 0` is labelled Q.
     *
     * | precompute_pc | precompute_point_transition  | precompute_round | Tx    | Ty    | Dx   | Dy   |
     * | -------- | ----------------------- | ----------- | ----- | ----- | ---- | ---- |
     * | 1        | 0                       |           0 |15P.x | 15P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           1 |13P.x | 13P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           2 |11P.x | 11P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           3 | 9P.x |  9P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           4 | 7P.x |  7P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           5 | 5P.x |  5P.y | 2P.x | 2P.y |
     * | 1        | 0                       |           6 | 3P.x |  3P.y | 2P.x | 2P.y |
     * | 1        | 1                       |           7 |  P.x |   P.y | 2P.x | 2P.y |
     * | 0        | 0                       |           0 |15Q.x | 15Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           1 |13Q.x | 13Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           2 |11Q.x | 11Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           3 | 9Q.x |  9Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           4 | 7Q.x |  7Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           5 | 5Q.x |  5Q.y | 2Q.x | 2Q.y |
     * | 0        | 0                       |           6 | 3Q.x |  3Q.y | 2Q.x | 2Q.y |
     * | 0        | 1                       |           7 |  Q.x |   Q.y | 2Q.x | 2Q.y |
     *
     * We apply the following relations to constrain the above table:
     *
     * 1. If precompute_point_transition = 0, (Dx, Dy) = (Dx_shift, Dy_shift)
     * 2. If precompute_point_transition = 1, (Dx, Dy) = 2 (Px, Py)
     * 3. If precompute_point_transition = 0, (Tx, Ty) = (Tx_shift, Ty_shift) + (Dx, Dy)
     *
     * The relations that constrain `precompute_point_transition` and `precompute_pc` are in `ecc_wnaf_relation.hpp`
     *
     * When precompute_point_transition = 1, we use a strict lookup protocol in `ecc_set_relation.hpp` to validate (pc,
     * Tx, Ty) belong to the set of points present in our transcript columns.
     * ("strict" lookup protocol = every item in the table must be read from once, and only once)
     *
     * For every row, we use a lookup protocol in `ecc_lookup_relation.hpp` to write the following tuples into a lookup
     * table:
     * 1. (pc, 15 - precompute_round, Tx, Ty)
     * 2. (pc, precompute_round, Tx, -Ty)
     *
     * The value `15 - precompute_round` describes the multiplier applied to P at the current row.
     * (this can be expanded into a wnaf value by taking `2x - 15` where `x = 15 - precompute_round`) .
     * The value `precompute_round` describes the *negative multiplier* applied to P at the current row.
     * This is also expanded into a wnaf value by taking `2x - 15` where `x = precompute_round`.
     *
     * The following table describes how taking (15 - precompute_round) for positive values and (precompute_round) for
     * negative values produces the WNAF slice values that correspond to the multipliers for (Tx, Ty) and (Tx, -Ty):
     *
     * | Tx    | Ty    | x = 15 - precompute_round | 2x - 15 | y = precompute_round | 2y - 15 |
     * | ----- | ----- | -------------------- | ------- | --------------- | ------- |
     * | 15P.x | 15P.y | 15                   |      15 |               0 |     -15 |
     * | 13P.x | 13P.y | 14                   |      13 |               1 |     -13 |
     * | 11P.x | 11P.y | 13                   |      11 |               2 |     -11 |
     * |  9P.x |  9P.y | 12                   |       9 |               3 |      -9 |
     * |  7P.x |  7P.y | 11                   |       7 |               4 |      -7 |
     * |  5P.x |  5P.y | 10                   |       5 |               5 |      -5 |
     * |  3P.x |  3P.y |  9                   |       3 |               6 |      -3 |
     * |   P.x |   P.y |  8                   |       1 |               7 |      -1 |
     */

    /**
     * @brief Validate Dx, Dy correctness relation
     *
     * When computing a point table for point [P] = (Px, Py), we require [D] (Dx, Dy) = 2.[P]
     * If all other relations are satisfied, we know that (Tx, Ty) = (Px, Py)
     * i.e. (Dx, Dy) = 2(Px, Py) when precompute_round_transition = 1.
     *
     * Double formula:
     * x_3 = 9x^4 / 4y^2 - 2x
     * y_3 = (3x^2 / 2y) * (x - x_3) - y
     *
     * Expanding into relations:
     * (x_3 + 2x) * 4y^2 - 9x^4 = 0
     * (y3 + y) * 2y - 3x^2 * (x - x_3) = 0
     */
    auto two_x = Tx + Tx;
    auto three_x = two_x + Tx;
    auto three_xx = Tx * three_x;
    auto nine_xxxx = three_xx * three_xx;
    auto two_y = Ty + Ty;
    auto four_yy = two_y * two_y;
    auto x_double_check = (Dx + two_x) * four_yy - nine_xxxx;
    auto y_double_check = (Ty + Dy) * two_y + three_xx * (Dx - Tx);
    std::get<0>(accumulator) += precompute_point_transition * x_double_check * scaling_factor;
    std::get<1>(accumulator) += precompute_point_transition * y_double_check * scaling_factor;

    /**
     * @brief If precompute_round_transition = 0, (Dx_shift, Dy_shift) = (Dx, Dy)
     *
     * 1st row is empty => don't apply if lagrange_first == 1
     */
    std::get<2>(accumulator) +=
        (-lagrange_first + 1) * (-precompute_point_transition + 1) * (Dx - Dx_shift) * scaling_factor;
    std::get<3>(accumulator) +=
        (-lagrange_first + 1) * (-precompute_point_transition + 1) * (Dy - Dy_shift) * scaling_factor;

    /**
     * @brief Valdiate (Tx, Ty) is correctly computed from (Tx_shift, Ty_shift), (Dx, Dy).
     *        If precompute_round_transition = 0, [T] = [T_shift] + [D].
     *
     * Add formula:
     * x_3 = (y_2 - y_1)^2 / (x_2 - x_1)^2 - x_2 - x_1
     * y_3 = ((y_2 - y_1) / (x_2 - x_1)) * (x_1 - x_3) - y_1
     *
     * Expanding into relations:
     * (x_3 + x_2 + x_1) * (x_2 - x_1)^2 - (y_2 - y_1)^2 = 0
     * (y_3 + y_1) * (x_2 - x_1) + (x_3 - x_1) * (y_2 - y_1) = 0
     *
     * We don't need to check for incomplete point addition edge case (x_1 == x_2)
     * TODO explain why (computing simple point multiples cannot trigger the edge cases, but need to serve a proof of
     * this...)
     */
    const auto& x1 = Tx_shift;
    const auto& y1 = Ty_shift;
    const auto& x2 = Dx;
    const auto& y2 = Dy;
    const auto& x3 = Tx;
    const auto& y3 = Ty;
    const auto lambda_numerator = y2 - y1;
    const auto lambda_denominator = x2 - x1;
    auto x_add_check = (x3 + x2 + x1) * lambda_denominator * lambda_denominator - lambda_numerator * lambda_numerator;
    auto y_add_check = (y3 + y1) * lambda_denominator + (x3 - x1) * lambda_numerator;
    std::get<4>(accumulator) +=
        (-lagrange_first + 1) * (-precompute_point_transition + 1) * x_add_check * scaling_factor;
    std::get<5>(accumulator) +=
        (-lagrange_first + 1) * (-precompute_point_transition + 1) * y_add_check * scaling_factor;
}

template class ECCVMPointTableRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationImpl, flavor::ECCVM);

} // namespace bb::honk::sumcheck
