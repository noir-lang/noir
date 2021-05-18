#pragma once
#include "compute_circuit_data.hpp"
#include "root_rollup_tx.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

bool verify_logic(root_rollup_tx& rollup, circuit_data const& circuit_data);

struct verify_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

verify_result verify(root_rollup_tx& rollup, circuit_data const& circuit_data);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
