#pragma once
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"

namespace bb::stdlib {

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
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/1002): if both point and scalar are constant, don't
        // bother adding constraints
    }

    return { points, scalars };
}
} // namespace bb::stdlib
