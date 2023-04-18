#include "fixed_base_scalar_mul.hpp"

namespace acir_format {

void create_fixed_base_constraint(Composer& composer, const FixedBaseScalarMul& input)
{

    field_ct scalar_as_field = field_ct::from_witness_index(&composer, input.scalar);
    auto public_key = group_ct::fixed_base_scalar_mul_g1<254>(scalar_as_field);

    composer.assert_equal(public_key.x.witness_index, input.pub_key_x);
    composer.assert_equal(public_key.y.witness_index, input.pub_key_y);
}

} // namespace acir_format
