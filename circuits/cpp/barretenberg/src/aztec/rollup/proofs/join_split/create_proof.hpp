#pragma once
#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "sign_join_split_tx.hpp"
#include "../../fixtures/user_context.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

inline std::vector<uint8_t> create_proof(join_split_tx& tx,
                                         fixtures::grumpkin_key_pair const& signer,
                                         circuit_data const& cd,
                                         numeric::random::Engine* rand_engine = nullptr)
{
    tx.signature = sign_join_split_tx(tx, signer, rand_engine);

    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    composer.rand_engine = rand_engine;

    join_split_circuit(composer, tx);

    if (composer.failed) {
        error("Join-split circuit logic failed: ", composer.err);
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
