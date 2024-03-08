#pragma once
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "join_split_tx.hpp"

namespace bb::join_split_example::proofs::join_split {

Builder new_join_split_circuit(join_split_tx const& tx);

} // namespace bb::join_split_example::proofs::join_split
