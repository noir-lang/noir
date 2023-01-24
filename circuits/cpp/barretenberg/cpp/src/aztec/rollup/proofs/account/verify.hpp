#pragma once
#include "../verify.hpp"
#include "./compute_circuit_data.hpp"
#include "./account.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace plonk::stdlib::types::turbo;

verify_result<Composer> verify_logic(account_tx& tx, circuit_data const& cd);

verify_result<Composer> verify(account_tx& tx, circuit_data const& cd);

} // namespace account
} // namespace proofs
} // namespace rollup
