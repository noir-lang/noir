#include "create_noop_join_split_proof.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include <rollup/tx/user_context.hpp>
#include <rollup/client_proofs/join_split/sign_notes.hpp>
#include <rollup/client_proofs/join_split/join_split.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

std::vector<uint8_t> create_noop_join_split_proof(fr const& merkel_root, join_split_circuit_data const& circuit_data)
{
    auto user = rollup::tx::create_user_context();

    tx_note gibberish_note = { user.public_key, 0, fr::random_element() };
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.merkle_root = merkel_root;
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.output_note = { gibberish_note, gibberish_note };

    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { user.private_key, user.public_key });

    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
    join_split_circuit(composer, tx);

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof.proof_data;
}

} // namespace rollup_proofs
} // namespace rollup
