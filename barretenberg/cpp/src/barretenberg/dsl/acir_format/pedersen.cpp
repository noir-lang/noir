#include "pedersen.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace acir_format {

using namespace bb;

template <typename Builder> void create_pedersen_constraint(Builder& builder, const PedersenConstraint& input)
{
    using field_ct = stdlib::field_t<Builder>;

    std::vector<field_ct> scalars;

    for (const auto& scalar : input.scalars) {
        // convert input indices to field_ct
        field_ct scalar_as_field = field_ct::from_witness_index(&builder, scalar);
        scalars.push_back(scalar_as_field);
    }

    auto point = stdlib::pedersen_commitment<Builder>::commit(scalars, input.hash_index);

    builder.assert_equal(point.x.witness_index, input.result_x);
    builder.assert_equal(point.y.witness_index, input.result_y);
}

template <typename Builder> void create_pedersen_hash_constraint(Builder& builder, const PedersenHashConstraint& input)
{
    using field_ct = stdlib::field_t<Builder>;

    std::vector<field_ct> scalars;

    for (const auto& scalar : input.scalars) {
        // convert input indices to field_ct
        field_ct scalar_as_field = field_ct::from_witness_index(&builder, scalar);
        scalars.push_back(scalar_as_field);
    }

    auto result = stdlib::pedersen_hash<Builder>::hash(scalars, input.hash_index);

    builder.assert_equal(result.witness_index, input.result);
}

template void create_pedersen_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                              const PedersenConstraint& input);
template void create_pedersen_hash_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                   const PedersenHashConstraint& input);
template void create_pedersen_constraint<MegaCircuitBuilder>(MegaCircuitBuilder& builder,
                                                             const PedersenConstraint& input);
template void create_pedersen_hash_constraint<MegaCircuitBuilder>(MegaCircuitBuilder& builder,
                                                                  const PedersenHashConstraint& input);

} // namespace acir_format
