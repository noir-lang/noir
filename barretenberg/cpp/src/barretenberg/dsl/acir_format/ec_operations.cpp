#include "ec_operations.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/groups/affine_element.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

namespace acir_format {

template <typename Builder>
void create_ec_add_constraint(Builder& builder, const EcAdd& input, bool has_valid_witness_assignments)
{
    // Input to cycle_group points
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;

    auto input1_point = to_grumpkin_point(
        input.input1_x, input.input1_y, input.input1_infinite, has_valid_witness_assignments, builder);
    auto input2_point = to_grumpkin_point(
        input.input2_x, input.input2_y, input.input2_infinite, has_valid_witness_assignments, builder);

    // Addition
    cycle_group_ct result = input1_point + input2_point;
    cycle_group_ct standard_result = result.get_standard_form();
    auto x_normalized = standard_result.x.normalize();
    auto y_normalized = standard_result.y.normalize();
    auto infinite = standard_result.is_point_at_infinity().normalize();
    builder.assert_equal(x_normalized.witness_index, input.result_x);
    builder.assert_equal(y_normalized.witness_index, input.result_y);
    builder.assert_equal(infinite.witness_index, input.result_infinite);
}

template void create_ec_add_constraint<bb::UltraCircuitBuilder>(bb::UltraCircuitBuilder& builder,
                                                                const EcAdd& input,
                                                                bool has_valid_witness_assignments);
template void create_ec_add_constraint<bb::MegaCircuitBuilder>(bb::MegaCircuitBuilder& builder,
                                                               const EcAdd& input,
                                                               bool has_valid_witness_assignments);

} // namespace acir_format
