#pragma once
#include "compute_circuit_data.hpp"
#include "root_verifier_circuit.hpp"
#include "root_verifier_proof_data.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace barretenberg;

using namespace plonk;

struct verify_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

verify_result verify(root_verifier_tx& tx, circuit_data const& circuit_data);

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
