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

    void assert_equal(const point& rhs, std::string const& msg = "point::assert_equal") const
    {
        this->x.assert_equal(rhs.x, msg);
        this->y.assert_equal(rhs.y, msg);
    }

    static point conditional_assign(const bool_t<Composer>& predicate, const point& lhs, const point& rhs)
    {
        return { field_t<Composer>::conditional_assign(predicate, lhs.x, rhs.x),
                 field_t<Composer>::conditional_assign(predicate, lhs.y, rhs.y) };
    };

    bool_t<Composer> operator==(const point& other) const { return (this->x == other.x) && (this->y == other.y); }
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