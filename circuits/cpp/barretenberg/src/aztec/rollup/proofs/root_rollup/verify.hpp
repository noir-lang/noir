#pragma once
#include "compute_circuit_data.hpp"
#include "root_rollup_tx.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

struct verify_result {
    bool verified;
    bool logic_verified;
    std::vector<fr> public_inputs;
    std::vector<uint8_t> proof_data;
    std::vector<fr> broadcast_data;
    recursion_output<bn254> recursion_output_data;
};

verify_result verify_logic(root_rollup_tx& rollup, circuit_data const& circuit_data);

verify_result verify(root_rollup_tx& rollup, circuit_data const& circuit_data);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
