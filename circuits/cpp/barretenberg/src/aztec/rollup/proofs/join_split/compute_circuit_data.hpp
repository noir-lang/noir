#pragma once
#include "join_split_tx.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

join_split_tx noop_tx();

struct circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    std::vector<uint8_t> padding_proof;
};

circuit_data compute_circuit_data(std::string const& srs_path);

circuit_data compute_or_load_circuit_data(std::string const& srs_path, std::string const& key_path);

} // namespace join_split
} // namespace proofs
} // namespace rollup