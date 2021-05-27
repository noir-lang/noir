#pragma once
#include "get_circuit_data.hpp"
#include "claim_circuit.hpp"

namespace rollup {
namespace proofs {
namespace claim {

std::vector<uint8_t> create_proof(claim_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);

    claim_circuit(composer, tx);

    if (composer.failed) {
        error("Claim circuit logic failed: ", composer.err);
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace claim
} // namespace proofs
} // namespace rollup
