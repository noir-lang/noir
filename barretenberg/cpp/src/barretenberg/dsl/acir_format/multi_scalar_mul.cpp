#include "multi_scalar_mul.hpp"
#include "barretenberg/dsl/acir_format/serde/acir.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

namespace acir_format {

using namespace bb;

template <typename Builder>
void create_multi_scalar_mul_constraint(Builder& builder,
                                        const MultiScalarMul& input,
                                        bool has_valid_witness_assignments)
{
    using cycle_group_ct = stdlib::cycle_group<Builder>;
    using cycle_scalar_ct = typename stdlib::cycle_group<Builder>::cycle_scalar;
    using field_ct = stdlib::field_t<Builder>;

    std::vector<cycle_group_ct> points;
    std::vector<cycle_scalar_ct> scalars;

    for (size_t i = 0; i < input.points.size(); i += 3) {
        // Instantiate the input point/variable base as `cycle_group_ct`
        cycle_group_ct input_point = to_grumpkin_point(
            input.points[i], input.points[i + 1], input.points[i + 2], has_valid_witness_assignments, builder);

        //  Reconstruct the scalar from the low and high limbs
        field_ct scalar_low_as_field = to_field_ct(input.scalars[2 * (i / 3)], builder);
        field_ct scalar_high_as_field = to_field_ct(input.scalars[2 * (i / 3) + 1], builder);
        cycle_scalar_ct scalar(scalar_low_as_field, scalar_high_as_field);

        // Add the point and scalar to the vectors
        points.push_back(input_point);
        scalars.push_back(scalar);
    }
    // Call batch_mul to multiply the points and scalars and sum the results
    auto output_point = cycle_group_ct::batch_mul(points, scalars).get_standard_form();

    // Add the constraints and handle constant values
    if (output_point.is_point_at_infinity().is_constant()) {
        builder.fix_witness(input.out_point_is_infinite, output_point.is_point_at_infinity().get_value());
    } else {
        builder.assert_equal(output_point.is_point_at_infinity().witness_index, input.out_point_is_infinite);
    }
    if (output_point.x.is_constant()) {
        builder.fix_witness(input.out_point_x, output_point.x.get_value());
    } else {
        builder.assert_equal(output_point.x.get_witness_index(), input.out_point_x);
    }
    if (output_point.y.is_constant()) {
        builder.fix_witness(input.out_point_y, output_point.y.get_value());
    } else {
        builder.assert_equal(output_point.y.get_witness_index(), input.out_point_y);
    }
}

template void create_multi_scalar_mul_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                      const MultiScalarMul& input,
                                                                      bool has_valid_witness_assignments);
template void create_multi_scalar_mul_constraint<MegaCircuitBuilder>(MegaCircuitBuilder& builder,
                                                                     const MultiScalarMul& input,
                                                                     bool has_valid_witness_assignments);

} // namespace acir_format
