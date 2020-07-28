#pragma once
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <rollup/client_proofs/escape_hatch/escape_hatch_tx.hpp>

namespace rollup {
namespace rollup_proofs {

rollup::client_proofs::escape_hatch::escape_hatch_tx dummy_tx();

struct escape_hatch_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    std::vector<uint8_t> padding_proof;
};

escape_hatch_circuit_data compute_escape_hatch_circuit_data(std::string const& srs_path);

escape_hatch_circuit_data compute_or_load_escape_hatch_circuit_data(std::string const& srs_path,
                                                                    std::string const& key_path);

} // namespace rollup_proofs
} // namespace rollup
