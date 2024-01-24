#include "bigint_constraint.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"

namespace acir_format {

template <typename Builder> void create_bigint_operations_constraint(Builder& builder, const BigIntOperation& input)
{
    // TODO
    (void)builder;
    info(input);
}

template void create_bigint_operations_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                       const BigIntOperation& input);
template void create_bigint_operations_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                             const BigIntOperation& input);

template <typename Builder>
void create_bigint_from_le_bytes_constraint(Builder& builder, const BigIntFromLeBytes& input)
{
    // TODO
    (void)builder;
    info(input);
}

template void create_bigint_from_le_bytes_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                          const BigIntFromLeBytes& input);
template void create_bigint_from_le_bytes_constraint<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                                const BigIntFromLeBytes& input);

} // namespace acir_format
