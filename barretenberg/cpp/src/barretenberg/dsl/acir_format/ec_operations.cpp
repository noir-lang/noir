#include "ec_operations.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/groups/affine_element.hpp"
#include "barretenberg/proof_system/arithmetization/gate_data.hpp"

namespace acir_format {

template <typename Builder>
void create_ec_add_constraint(Builder& builder, const EcAdd& input, bool has_valid_witness_assignments)
{
    // Input to cycle_group points
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    using field_ct = bb::stdlib::field_t<Builder>;

    auto x1 = field_ct::from_witness_index(&builder, input.input1_x);
    auto y1 = field_ct::from_witness_index(&builder, input.input1_y);
    auto x2 = field_ct::from_witness_index(&builder, input.input2_x);
    auto y2 = field_ct::from_witness_index(&builder, input.input2_y);
    if (!has_valid_witness_assignments) {
        auto g1 = grumpkin::g1::affine_one;
        // We need to have correct values representing points on the curve
        builder.variables[input.input1_x] = g1.x;
        builder.variables[input.input1_y] = g1.y;
        builder.variables[input.input2_x] = g1.x;
        builder.variables[input.input2_y] = g1.y;
    }

    cycle_group_ct input1_point(x1, y1, false);
    cycle_group_ct input2_point(x2, y2, false);

    // Addition
    cycle_group_ct result = input1_point + input2_point;

    auto x_normalized = result.x.normalize();
    auto y_normalized = result.y.normalize();
    builder.assert_equal(x_normalized.witness_index, input.result_x);
    builder.assert_equal(y_normalized.witness_index, input.result_y);
}

template void create_ec_add_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                            const EcAdd& input,
                                                            bool has_valid_witness_assignments);
template void create_ec_add_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                  const EcAdd& input,
                                                                  bool has_valid_witness_assignments);

} // namespace acir_format
