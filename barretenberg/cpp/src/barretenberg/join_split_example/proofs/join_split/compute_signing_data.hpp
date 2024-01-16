#pragma once
#include "join_split_tx.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

bb::fr compute_signing_data(join_split_tx const& tx);

} // namespace join_split
} // namespace proofs
} // namespace join_split_example