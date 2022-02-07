#pragma once
#include "compute_circuit_data.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <plonk/proof_system/types/plonk_proof.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

std::vector<uint8_t> create_noop_join_split_proof(circuit_data const& circuit_data,
                                                  barretenberg::fr const& merkle_root,
                                                  bool valid = true,
                                                  bool mock = false);

} // namespace join_split
} // namespace proofs
} // namespace rollup
