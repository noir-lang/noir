#pragma once
#include "compute_join_split_circuit_data.hpp"
#include "compute_account_circuit_data.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace rollup_proofs {

struct rollup_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t rollup_size;
    size_t num_gates;
    size_t proof_lengths;
    std::vector<std::shared_ptr<waffle::verification_key>> verification_keys;
};

rollup_circuit_data compute_rollup_circuit_data(size_t rollup_size,
                                                join_split_circuit_data const& join_split_circuit_data,
                                                account_circuit_data const& account_circuit_data,
                                                bool create_keys,
                                                std::string const& srs_path);

rollup_circuit_data compute_or_load_rollup_circuit_data(size_t rollup_size,
                                                        join_split_circuit_data const& join_split_circuit_data,
                                                        account_circuit_data const& account_circuit_data,
                                                        std::string const& srs_path,
                                                        std::string const& key_path);

} // namespace rollup_proofs
} // namespace rollup
