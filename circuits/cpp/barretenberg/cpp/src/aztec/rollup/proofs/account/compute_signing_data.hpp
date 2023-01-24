#pragma once
#include "account_tx.hpp"

namespace rollup {
namespace proofs {
namespace account {

barretenberg::fr compute_signing_data(account_tx const& tx);

} // namespace account
} // namespace proofs
} // namespace rollup
