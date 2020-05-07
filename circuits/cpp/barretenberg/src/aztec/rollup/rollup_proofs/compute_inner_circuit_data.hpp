#pragma once
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace rollup_proofs {

struct inner_circuit_data {
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t proof_size;
};

inner_circuit_data compute_inner_circuit_data();

} // namespace rollup_proofs
} // namespace rollup