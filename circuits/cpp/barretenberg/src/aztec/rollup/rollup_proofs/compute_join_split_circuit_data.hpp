#pragma once
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace rollup_proofs {

struct join_split_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    size_t proof_size;
};

join_split_circuit_data compute_join_split_circuit_data();

} // namespace rollup_proofs
} // namespace rollup