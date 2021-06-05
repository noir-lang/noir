#pragma once
#include "compute_circuit_data.hpp"
#include "account.hpp"
#include "../../fixtures/user_context.hpp"

namespace rollup {
namespace proofs {
namespace account {

inline std::vector<uint8_t> create_proof(account_tx& tx,
                                         fixtures::grumpkin_key_pair const& signer,
                                         circuit_data const& cd,
                                         numeric::random::Engine* rand_engine = nullptr)
{
    tx.sign(signer);

    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    composer.rand_engine = rand_engine;

    account_circuit(composer, tx);

    if (composer.failed) {
        error("Account circuit logic failed: ", composer.err);
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace account
} // namespace proofs
} // namespace rollup
