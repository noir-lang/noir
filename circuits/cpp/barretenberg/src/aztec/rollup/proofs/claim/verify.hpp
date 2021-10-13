#pragma once
#include "claim_circuit.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

inline bool verify_logic(claim_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    claim_circuit(composer, tx);

    if (composer.failed) {
        info("Circuit logic failed: " + composer.err);
        return false;
    }

    return true;
}

struct verify_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"
inline verify_result verify(claim_tx const& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    claim_circuit(composer, tx);

    if (composer.failed) {
        info("Circuit logic failed: " + composer.err);
        return { false, {} };
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    auto verifier = composer.create_unrolled_verifier();
    if (!verifier.verify_proof(proof)) {
        info("Proof validation failed.");
        return { false, {} };
    }

    return { true, proof.proof_data };
}

} // namespace claim
} // namespace proofs
} // namespace rollup
