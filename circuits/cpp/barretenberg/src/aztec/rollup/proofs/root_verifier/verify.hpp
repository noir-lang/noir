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
    bool logic_verified;
    std::vector<uint8_t> proof_data;
    std::vector<fr> public_inputs;
};

verify_result verify_logic(root_verifier_tx& tx,
                           circuit_data const& circuit_data,
                           root_rollup::circuit_data const& root_rollup_cd);

verify_result verify_proverless(root_verifier_tx& tx,
                                circuit_data const& circuit_data,
                                root_rollup::circuit_data const& root_rollup_cd);

verify_result verify(root_verifier_tx& tx,
                     circuit_data const& circuit_data,
                     root_rollup::circuit_data const& root_rollup_cd);

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
