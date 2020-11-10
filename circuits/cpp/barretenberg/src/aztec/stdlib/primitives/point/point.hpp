#pragma once
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

template <typename Composer, typename E> point<Composer> create_point_witness(Composer& composer, E const& p)
{
    return { witness_t<Composer>(&composer, p.x), witness_t<Composer>(&composer, p.y) };
}

template <typename Composer> std::ostream& operator<<(std::ostream& os, point<Composer> const& p)
{
    return os << "{ " << p.x << ", " << p.y << " }";
}

} // namespace stdlib
} // namespace plonk