#pragma once
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"

namespace bb::stdlib {

/**
 * @brief Compute an offset generator for use in biggroup tables
 *
 *@details Sometimes the points from which we construct the tables are going to be dependent in such a way that
 *combining them for constructing the table is not possible without handling the edgecases such as the point at infinity
 *and doubling. To avoid handling those we add multiples of this offset generator to the points.
 *
 * @param num_rounds
 */
template <typename C, class Fq, class Fr, class G>
typename G::affine_element element<C, Fq, Fr, G>::compute_table_offset_generator()
{
    constexpr typename G::affine_element offset_generator =
        G::derive_generators("biggroup table offset generator", 1)[0];

    return offset_generator;
}

/**
 * @brief Given two lists of points that need to be multiplied by scalars, create a new list of length +1 with original
 * points masked, but the same scalar product sum
 * @details Add +1G, +2G, +4G etc to the original points and adds a new point 2ⁿ⋅G and scalar x to the lists. By
 * doubling the point every time, we ensure that no +-1 combination of 6 sequential elements run into edgecases, unless
 * the points are deliberately constructed to trigger it.
 */
template <typename C, class Fq, class Fr, class G>
std::pair<std::vector<element<C, Fq, Fr, G>>, std::vector<Fr>> element<C, Fq, Fr, G>::mask_points(
    const std::vector<element>& _points, const std::vector<Fr>& _scalars)
{
    std::vector<element> points;
    std::vector<Fr> scalars;
    ASSERT(_points.size() == _scalars.size());
    using NativeFr = typename Fr::native;
    auto running_scalar = NativeFr::one();
    // Get the offset generator G_offset in native and in-circuit form
    auto native_offset_generator = element::compute_table_offset_generator();
    Fr last_scalar = Fr(0);
    NativeFr generator_coefficient = NativeFr(2).pow(_points.size());
    auto generator_coefficient_inverse = generator_coefficient.invert();
    // For each point and scalar
    for (size_t i = 0; i < _points.size(); i++) {
        scalars.push_back(_scalars[i]);
        // Convert point into point + 2ⁱ⋅G_offset
        points.push_back(_points[i] + (native_offset_generator * running_scalar));
        // Add \frac{2ⁱ⋅scalar}{2ⁿ} to the last scalar
        last_scalar += _scalars[i] * (running_scalar * generator_coefficient_inverse);
        // Double the running scalar
        running_scalar += running_scalar;
    }

    // Add a scalar -(<(1,2,4,...,2ⁿ⁻¹ ),(scalar₀,...,scalarₙ₋₁)> / 2ⁿ)
    scalars.push_back(-last_scalar);
    if constexpr (Fr::is_composite) {
        scalars.back().self_reduce();
    }
    // Add in-circuit G_offset to points
    points.push_back(element(native_offset_generator * generator_coefficient));

    return { points, scalars };
}

/**
 * @brief Replace all pairs (∞, scalar) by the pair (one, 0) where one is a fixed generator of the curve
 * @details This is a step in enabling our our multiscalar multiplication algorithms to hande points at infinity.
 */
template <typename C, class Fq, class Fr, class G>
std::pair<std::vector<element<C, Fq, Fr, G>>, std::vector<Fr>> element<C, Fq, Fr, G>::handle_points_at_infinity(
    const std::vector<element>& _points, const std::vector<Fr>& _scalars)
{
    auto builder = _points[0].get_context();
    std::vector<element> points;
    std::vector<Fr> scalars;
    element one = element::one(builder);

    for (auto [_point, _scalar] : zip_view(_points, _scalars)) {
        bool_ct is_point_at_infinity = _point.is_point_at_infinity();
        if (is_point_at_infinity.get_value() && static_cast<bool>(is_point_at_infinity.is_constant())) {
            // if point is at infinity and a circuit constant we can just skip.
            continue;
        }
        if (_scalar.get_value() == 0 && _scalar.is_constant()) {
            // if scalar multiplier is 0 and also a constant, we can skip
            continue;
        }
        Fq updated_x = Fq::conditional_assign(is_point_at_infinity, one.x, _point.x);
        Fq updated_y = Fq::conditional_assign(is_point_at_infinity, one.y, _point.y);
        element point(updated_x, updated_y);
        Fr scalar = Fr::conditional_assign(is_point_at_infinity, 0, _scalar);

        points.push_back(point);
        scalars.push_back(scalar);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/1002): if both point and scalar are constant,
        // don't bother adding constraints
    }

    return { points, scalars };
}
} // namespace bb::stdlib
