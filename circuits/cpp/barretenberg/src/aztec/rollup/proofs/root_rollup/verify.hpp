#pragma once
#include "../verify.hpp"
#include "compute_circuit_data.hpp"
#include "root_rollup_tx.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

struct verify_result : ::rollup::proofs::verify_result<Composer> {
    std::vector<fr> broadcast_data;
};

verify_result verify_logic(root_rollup_tx& tx, circuit_data const& cd);

verify_result verify(root_rollup_tx& tx, circuit_data const& cd);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
