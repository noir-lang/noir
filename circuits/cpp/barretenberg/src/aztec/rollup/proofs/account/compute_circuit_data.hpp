#pragma once
#include "account_tx.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace account {

struct circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
};

circuit_data compute_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs);

circuit_data compute_or_load_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                          std::string const& key_path);

} // namespace account
} // namespace proofs
} // namespace rollup
