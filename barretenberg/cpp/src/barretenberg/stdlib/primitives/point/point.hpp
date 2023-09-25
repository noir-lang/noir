#pragma once

#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

#include "../field/field.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename Builder> struct point {
    field_t<Builder> x;
    field_t<Builder> y;

    void set_public()
    {
        auto builder = x.context;
        builder->set_public_input(x.witness_index);
        builder->set_public_input(y.witness_index);
    }

    void assert_equal(const point& rhs, std::string const& msg = "point::assert_equal") const
    {
        this->x.assert_equal(rhs.x, msg);
        this->y.assert_equal(rhs.y, msg);
    }

    void on_curve(std::string const& msg = "point::on_curve: point not on curve") const
    {
        auto on_curve = x * x;
        on_curve = on_curve * x + grumpkin::g1::curve_b; // x^3 - 17
        on_curve = y.madd(y, -on_curve);                 // on_curve = y^2 - (x^3 - 17) == 0
        on_curve.assert_is_zero(msg);
    }

    void assert_not_equal(const point& rhs, std::string const& msg = "point:assert_not_equal") const
    {
        const auto lhs_eq = this->x == rhs.x;
        const auto rhs_eq = this->y == rhs.y;
        field_t<Builder>(lhs_eq && rhs_eq).assert_is_zero(msg);
    }

    static point conditional_assign(const bool_t<Builder>& predicate, const point& lhs, const point& rhs)
    {
        return { field_t<Builder>::conditional_assign(predicate, lhs.x, rhs.x),
                 field_t<Builder>::conditional_assign(predicate, lhs.y, rhs.y) };
    };

    bool_t<Builder> operator==(const point& other) const { return (this->x == other.x) && (this->y == other.y); }

    point<Builder> operator+(const point& other) const
    {
        const field_t<Builder>& x1 = this->x;
        const field_t<Builder>& y1 = this->y;

        const field_t<Builder>& x2 = other.x;
        const field_t<Builder>& y2 = other.y;

        const field_t<Builder> lambda = (y2 - y1) / (x2 - x1);
        const field_t<Builder> x3 = lambda * lambda - x2 - x1;
        const field_t<Builder> y3 = lambda * (x1 - x3) - y1;

        return { x3, y3 };
    }
};

template <typename Builder, typename E>
point<Builder> create_point_witness(Builder& builder, E const& p, const bool validate_on_curve = true)
{
    // validate point is on the grumpkin curve
    field_t<Builder> x(witness_t<Builder>(&builder, p.x));
    field_t<Builder> y(witness_t<Builder>(&builder, p.y));
    point<Builder> result = { x, y };

    // we need to disable this for when we are conditionally creating a point (e.g. account output note spending keys)
    if (validate_on_curve) {
        result.on_curve("create_point_witness: point not on curve");
    }
    return { x, y };
}

template <typename Builder> std::ostream& operator<<(std::ostream& os, point<Builder> const& p)
{
    return os << "{ " << p.x << ", " << p.y << " }";
}

} // namespace stdlib
} // namespace proof_system::plonk
