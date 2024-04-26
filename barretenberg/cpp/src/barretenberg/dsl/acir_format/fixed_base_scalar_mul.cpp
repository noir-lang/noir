#include "fixed_base_scalar_mul.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"

namespace acir_format {

template <typename Builder> void create_fixed_base_constraint(Builder& builder, const FixedBaseScalarMul& input)
{
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    using cycle_scalar_ct = typename bb::stdlib::cycle_group<Builder>::cycle_scalar;
    using field_ct = bb::stdlib::field_t<Builder>;

    // We reconstruct the scalar from the low and high limbs
    field_ct low_as_field = field_ct::from_witness_index(&builder, input.low);
    field_ct high_as_field = field_ct::from_witness_index(&builder, input.high);
    cycle_scalar_ct scalar(low_as_field, high_as_field);

    // We multiply the scalar with G1 to get the result
    auto result = cycle_group_ct(grumpkin::g1::affine_one) * scalar;

    // Finally we add the constraints
    builder.assert_equal(result.x.get_witness_index(), input.pub_key_x);
    builder.assert_equal(result.y.get_witness_index(), input.pub_key_y);
}

template void create_fixed_base_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                const FixedBaseScalarMul& input);
template void create_fixed_base_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                      const FixedBaseScalarMul& input);

} // namespace acir_format
