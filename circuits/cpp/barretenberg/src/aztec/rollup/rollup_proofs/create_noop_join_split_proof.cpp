#include "create_noop_join_split_proof.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include <rollup/client_proofs/join_split/join_split.hpp>
#include <rollup/client_proofs/join_split/sign_notes.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>
#include <sys/stat.h>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

std::vector<uint8_t> create_noop_join_split_proof(join_split_circuit_data const& circuit_data, fr const& merkle_root)
{
    join_split_tx tx = noop_tx();
    tx.merkle_root = merkle_root;

    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
    join_split_circuit(composer, tx);

    if (composer.failed) {
        error("join split logic failed: ", composer.err);
    }

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace rollup_proofs
} // namespace rollup
