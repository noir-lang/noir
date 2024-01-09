#include "ec_operations.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/proof_system/arithmetization/gate_data.hpp"

namespace acir_format {

template <typename Builder> void create_ec_add_constraint(Builder& builder, const EcAdd& input)
{
    // TODO
    builder.assert_equal(input.input1_x, input.input1_x);
    ASSERT(false);
}

template void create_ec_add_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder, const EcAdd& input);
template void create_ec_add_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                  const EcAdd& input);

template <typename Builder> void create_ec_double_constraint(Builder& builder, const EcDouble& input)
{
    // TODO
    builder.assert_equal(input.input_x, input.input_x);
    ASSERT(false);
}

template void create_ec_double_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder, const EcDouble& input);
template void create_ec_double_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                     const EcDouble& input);

} // namespace acir_format
