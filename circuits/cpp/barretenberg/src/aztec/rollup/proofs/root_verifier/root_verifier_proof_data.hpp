#pragma once
#include "root_verifier_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

struct root_verifier_proof_data {
    barretenberg::fr broadcasted_inputs_hash_reduced;
    g1::affine_element recursion_output[2];

    root_verifier_proof_data() {}
    root_verifier_proof_data(std::vector<uint8_t> const& proof_data);

    bool operator==(const root_verifier_proof_data& other) const = default;
};

namespace RootVerifierProofFields {
enum {
    BROADCASTED_INPUTS_HASH_REDUCED,
    NUM_FIELDS,
};
} // namespace RootVerifierProofFields

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
