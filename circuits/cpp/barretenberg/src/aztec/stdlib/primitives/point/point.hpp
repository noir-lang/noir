#pragma once
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer> struct point {
    field_t<Composer> x;
    field_t<Composer> y;
};

template <typename Composer, typename E> point<Composer> create_point_witness(Composer& composer, E const& p)
{
    return { witness_t<Composer>(&composer, p.x), witness_t<Composer>(&composer, p.y) };
}

} // namespace stdlib
} // namespace plonk