#pragma once
#include "claim_tx.hpp"
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::types;

void claim_circuit(Composer& composer, claim_tx const& tx);

} // namespace claim
} // namespace proofs
} // namespace rollup
