#pragma once
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
    std::shared_ptr<waffle::verification_key> inner_verification_key;
};

rollup_circuit_data compute_rollup_circuit_data(size_t rollup_size);

} // namespace rollup_proofs
} // namespace rollup