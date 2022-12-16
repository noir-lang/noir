#pragma once
#include "compute_circuit_data.hpp"
#include "rollup_tx.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types;

verify_result<Composer> verify_logic(rollup_tx& tx, circuit_data const& cd);

verify_result<Composer> verify(rollup_tx& tx, circuit_data const& cd);

} // namespace rollup
} // namespace proofs
} // namespace rollup
