#pragma once
#include "escape_hatch_tx.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace plonk::stdlib::types::turbo;

void escape_hatch_circuit(Composer& composer, escape_hatch_tx const& tx, bool can_throw);

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
