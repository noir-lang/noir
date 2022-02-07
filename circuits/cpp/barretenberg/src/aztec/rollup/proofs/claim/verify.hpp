#pragma once
#include "../verify.hpp"
#include "./get_circuit_data.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::types::turbo;

verify_result<Composer> verify_logic(claim_tx& tx, circuit_data const& cd);

verify_result<Composer> verify(claim_tx& tx, circuit_data const& cd);

} // namespace claim
} // namespace proofs
} // namespace rollup
