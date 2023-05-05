#pragma once
#include "barretenberg/common/map.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
namespace join_split_example {
namespace proofs {
namespace mock {

using namespace proof_system::plonk::stdlib;

template <typename Composer> void mock_circuit(Composer& composer, std::vector<fr> const& public_inputs_)
{
    const auto public_inputs = map(public_inputs_, [&](auto& i) { return field_t(witness_t(&composer, i)); });
    for (auto& p : public_inputs) {
        p.set_public();
    }
    plonk::stdlib::pedersen_commitment<Composer>::compress(field_t(witness_t(&composer, 1)),
                                                           field_t(witness_t(&composer, 1)));
}

} // namespace mock
} // namespace proofs
} // namespace join_split_example
