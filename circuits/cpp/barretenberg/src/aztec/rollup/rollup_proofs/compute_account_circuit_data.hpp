#pragma once
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <rollup/client_proofs/account/account_tx.hpp>

namespace rollup {
namespace rollup_proofs {

struct account_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
};

account_circuit_data compute_account_circuit_data(std::string const& srs_path);

account_circuit_data compute_or_load_account_circuit_data(std::string const& srs_path, std::string const& key_path);

} // namespace rollup_proofs
} // namespace rollup
