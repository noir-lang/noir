#pragma once
#include "compute_circuit_data.hpp"
#include "rollup_tx.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;

struct verify_result {
    bool verified;
    bool logic_verified;
    std::string err;
    std::vector<uint8_t> proof_data;
    std::vector<fr> public_inputs;
};

verify_result verify_logic(rollup_tx& tx, circuit_data const& circuit_data);

verify_result verify(rollup_tx& tx, circuit_data const& circuit_data);

} // namespace rollup
} // namespace proofs
} // namespace rollup
