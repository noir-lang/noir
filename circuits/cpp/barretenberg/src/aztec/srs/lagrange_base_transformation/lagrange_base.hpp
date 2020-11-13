#pragma once 

#include "../../ecc/curves/bn254/g1.hpp"
#include "../../polynomials/evaluation_domain.hpp"
#include "../../ecc/groups/element.hpp"

namespace barretenberg
{
namespace lagrange_base
{
    void transform_srs(g1::affine_element*, g1::affine_element*, const size_t);
}
}