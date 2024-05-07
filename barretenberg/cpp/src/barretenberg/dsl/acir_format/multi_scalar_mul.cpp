#include "multi_scalar_mul.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"

namespace acir_format {

template <typename Builder> void create_multi_scalar_mul_constraint(Builder& builder, const MultiScalarMul& input)
{
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    using cycle_scalar_ct = typename bb::stdlib::cycle_group<Builder>::cycle_scalar;
    using field_ct = bb::stdlib::field_t<Builder>;

    std::vector<cycle_group_ct> points;
    std::vector<cycle_scalar_ct> scalars;

    for (size_t i = 0; i < input.points.size(); i += 2) {
        // Instantiate the input point/variable base as `cycle_group_ct`
        auto point_x = field_ct::from_witness_index(&builder, input.points[i]);
        auto point_y = field_ct::from_witness_index(&builder, input.points[i + 1]);
        cycle_group_ct input_point(point_x, point_y, false);

        // Reconstruct the scalar from the low and high limbs
        field_ct scalar_low_as_field = field_ct::from_witness_index(&builder, input.scalars[i]);
        field_ct scalar_high_as_field = field_ct::from_witness_index(&builder, input.scalars[i + 1]);
        cycle_scalar_ct scalar(scalar_low_as_field, scalar_high_as_field);

        // Add the point and scalar to the vectors
        points.push_back(input_point);
        scalars.push_back(scalar);
    }

    // Call batch_mul to multiply the points and scalars and sum the results
    auto output_point = cycle_group_ct::batch_mul(scalars, points);

    // Add the constraints
    builder.assert_equal(output_point.x.get_witness_index(), input.out_point_x);
    builder.assert_equal(output_point.y.get_witness_index(), input.out_point_y);
}

template void create_multi_scalar_mul_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                      const MultiScalarMul& input);
template void create_multi_scalar_mul_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                            const MultiScalarMul& input);

} // namespace acir_format
