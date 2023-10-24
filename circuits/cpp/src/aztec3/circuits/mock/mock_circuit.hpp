#pragma once

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::mock {

using namespace plonk::stdlib;

template <typename Builder> void mock_circuit(Builder& builder, std::vector<fr> const& public_inputs_)
{
    const auto public_inputs = map(public_inputs_, [&](auto& i) { return field_t(witness_t(&builder, i)); });
    for (auto& p : public_inputs) {
        p.set_public();
    }
    plonk::stdlib::pedersen<Builder>::hash({ field_t(witness_t(&builder, 1)), field_t(witness_t(&builder, 1)) });
}

}  // namespace aztec3::circuits::mock
