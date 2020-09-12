#pragma once
#include "account_tx.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace account {

struct account_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
};

account_circuit_data compute_account_circuit_data(std::string const& srs_path);

account_circuit_data compute_or_load_account_circuit_data(std::string const& srs_path, std::string const& key_path);

} // namespace account
} // namespace proofs
} // namespace rollup
