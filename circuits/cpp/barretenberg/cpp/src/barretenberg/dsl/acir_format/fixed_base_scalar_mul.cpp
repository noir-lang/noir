#include "fixed_base_scalar_mul.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/proof_system/arithmetization/gate_data.hpp"

namespace acir_format {

void create_fixed_base_constraint(Builder& builder, const FixedBaseScalarMul& input)
{

    // Computes low * G + high * 2^128 * G
    //
    // Low and high need to be less than 2^128
    field_ct low_as_field = field_ct::from_witness_index(&builder, input.low);
    field_ct high_as_field = field_ct::from_witness_index(&builder, input.high);

    low_as_field.create_range_constraint(128);
    high_as_field.create_range_constraint(128);

    auto low_value = grumpkin::fr(low_as_field.get_value());
    auto high_value = grumpkin::fr(high_as_field.get_value());
    auto pow_128 = grumpkin::fr(2).pow(128);

    grumpkin::g1::element result = grumpkin::g1::one * low_value + grumpkin::g1::one * (high_value * pow_128);
    grumpkin::g1::affine_element result_affine = result.normalize();

    auto x_var = builder.add_variable(result_affine.x);
    auto y_var = builder.add_variable(result_affine.y);
    builder.create_add_gate({ x_var,
                              y_var,
                              x_var,
                              barretenberg::fr::zero(),
                              barretenberg::fr::zero(),
                              barretenberg::fr::zero(),
                              barretenberg::fr::zero() });

    builder.assert_equal(x_var, input.pub_key_x);
    builder.assert_equal(y_var, input.pub_key_y);
}

} // namespace acir_format
