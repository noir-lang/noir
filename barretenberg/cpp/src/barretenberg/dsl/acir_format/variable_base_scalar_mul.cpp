#include "variable_base_scalar_mul.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"

namespace acir_format {

template <typename Builder> void create_variable_base_constraint(Builder& builder, const VariableBaseScalarMul& input)
{
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    using cycle_scalar_ct = typename bb::stdlib::cycle_group<Builder>::cycle_scalar;
    using field_ct = bb::stdlib::field_t<Builder>;

    // We instantiate the input point/variable base as `cycle_group_ct`
    auto point_x = field_ct::from_witness_index(&builder, input.point_x);
    auto point_y = field_ct::from_witness_index(&builder, input.point_y);
    cycle_group_ct input_point(point_x, point_y, false);

    // We reconstruct the scalar from the low and high limbs
    field_ct scalar_low_as_field = field_ct::from_witness_index(&builder, input.scalar_low);
    field_ct scalar_high_as_field = field_ct::from_witness_index(&builder, input.scalar_high);
    cycle_scalar_ct scalar(scalar_low_as_field, scalar_high_as_field);

    // We multiply the scalar with input point/variable base to get the result
    auto result = input_point * scalar;

    // Finally we add the constraints
    builder.assert_equal(result.x.get_witness_index(), input.out_point_x);
    builder.assert_equal(result.y.get_witness_index(), input.out_point_y);
}

template void create_variable_base_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                   const VariableBaseScalarMul& input);
template void create_variable_base_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                         const VariableBaseScalarMul& input);

} // namespace acir_format
