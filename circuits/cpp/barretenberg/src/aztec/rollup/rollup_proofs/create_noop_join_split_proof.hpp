#pragma once
#include "compute_join_split_circuit_data.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <plonk/proof_system/types/plonk_proof.hpp>

namespace rollup {
namespace rollup_proofs {

std::vector<uint8_t> create_noop_join_split_proof(join_split_circuit_data const& circuit_data);

std::vector<uint8_t> create_noop_join_split_proof(join_split_circuit_data const& circuit_data,
                                                  barretenberg::fr const& merkle_root);

} // namespace rollup_proofs
} // namespace rollup
