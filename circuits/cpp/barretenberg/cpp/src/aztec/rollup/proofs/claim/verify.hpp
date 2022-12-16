#pragma once
#include "../verify.hpp"
#include "./get_circuit_data.hpp"
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::types;

verify_result<Composer> verify_logic(claim_tx& tx, circuit_data const& cd);

verify_result<Composer> verify(claim_tx& tx, circuit_data const& cd);

std::shared_ptr<waffle::verification_key> get_verification_key();

size_t get_number_of_gates();

} // namespace claim
} // namespace proofs
} // namespace rollup
