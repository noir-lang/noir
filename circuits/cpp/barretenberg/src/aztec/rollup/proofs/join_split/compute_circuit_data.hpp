#pragma once
#include "join_split_tx.hpp"
#include "../compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

join_split_tx noop_tx();

using circuit_data = proofs::circuit_data;

circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs);

} // namespace join_split
} // namespace proofs
} // namespace rollup