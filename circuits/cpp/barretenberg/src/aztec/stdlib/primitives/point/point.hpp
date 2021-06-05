#pragma once

#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer> struct point {
    field_t<Composer> x;
    field_t<Composer> y;

    void set_public()
    {
        auto composer = x.context;
        composer->set_public_input(x.witness_index);
        composer->set_public_input(y.witness_index);
    }
};

template <typename Composer, typename E>
point<Composer> create_point_witness(Composer& composer, E const& p, const bool validate_on_curve = true)
{
    // validate point is on the grumpkin curve
    field_t<Composer> x(witness_t<Composer>(&composer, p.x));
    field_t<Composer> y(witness_t<Composer>(&composer, p.y));

    // we need to disable this for when we are conditionally creating a point (e.g. account output note spending keys)
    if (validate_on_curve) {
        auto on_curve = x * x;
        on_curve = on_curve * x + grumpkin::g1::curve_b; // x^3 - 17
        on_curve = y.madd(y, -on_curve);                 // on_curve = y^2 - (x^3 - 17) == 0
        on_curve.assert_is_zero("create_point_witness: point not on curve");
    }
    return { x, y };
}

template <typename Composer> std::ostream& operator<<(std::ostream& os, point<Composer> const& p)
{
    return os << "{ " << p.x << ", " << p.y << " }";
}

} // namespace stdlib
} // namespace plonk