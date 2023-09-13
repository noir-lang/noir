#pragma once
#include "../../fixtures/user_context.hpp"
#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "sign_join_split_tx.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

inline std::vector<uint8_t> create_proof(join_split_tx const& tx, circuit_data const& cd)
{
    Builder builder(cd.num_gates);
    join_split_circuit(builder, tx);

    if (builder.failed()) {
        info("Join-split circuit logic failed: ", builder.err());
    }

    Composer composer = Composer(cd.proving_key, cd.verification_key);
    auto prover = composer.create_prover(builder);
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
