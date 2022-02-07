#pragma once
#include <common/map.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace mock {

using namespace plonk::stdlib;

template <typename Composer> void mock_circuit(Composer& composer, std::vector<fr> const& public_inputs_)
{
    const auto public_inputs = map(public_inputs_, [&](auto& i) { return field_t(witness_t(&composer, i)); });
    for (auto& p : public_inputs) {
        p.set_public();
    }
    plonk::stdlib::pedersen<Composer>::compress(field_t(witness_t(&composer, 1)), field_t(witness_t(&composer, 1)));
}

} // namespace mock
} // namespace proofs
} // namespace rollup