#pragma once
#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "sign_join_split_tx.hpp"
#include "../../fixtures/user_context.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

inline std::vector<uint8_t> create_proof(join_split_tx const& tx,
                                         circuit_data const& cd,
                                         numeric::random::Engine* rand_engine = nullptr)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    composer.rand_engine = rand_engine;
    join_split_circuit(composer, tx);

    if (composer.failed()) {
        info("Join-split circuit logic failed: ", composer.err());
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
