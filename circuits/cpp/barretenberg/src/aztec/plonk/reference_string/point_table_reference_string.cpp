#include "point_table_reference_string.hpp"

namespace waffle {

PointTableReferenceString::PointTableReferenceString(barretenberg::g1::affine_element* monomials)
    : monomials_(monomials)
{}

PointTableReferenceString::~PointTableReferenceString()
{
}

} // namespace waffle