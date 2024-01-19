#pragma once
#include "barretenberg/common/map.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
namespace join_split_example {
namespace proofs {
namespace mock {

using namespace bb::plonk;

template <typename Builder> void mock_circuit(Builder& builder, std::vector<bb::fr> const& public_inputs_)
{
    const auto public_inputs =
        map(public_inputs_, [&](auto& i) { return stdlib::field_t(stdlib::witness_t(&builder, i)); });
    for (auto& p : public_inputs) {
        p.set_public();
    }
    stdlib::pedersen_hash<Builder>::hash(
        { stdlib::field_t(stdlib::witness_t(&builder, 1)), stdlib::field_t(stdlib::witness_t(&builder, 1)) });
}

} // namespace mock
} // namespace proofs
} // namespace join_split_example
