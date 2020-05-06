#pragma once
#include "../tx/user_context.hpp"
#include "../client_proofs/join_split/sign_notes.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace plonk::stdlib::types::turbo;

waffle::plonk_proof create_noop_join_split_proof()
{
    auto user = rollup::tx::create_user_context();

    tx_note gibberish_note = { user.public_key, 0, fr::random_element() };
    auto gibberish_path = plonk::stdlib::merkle_tree::fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element() ));

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.merkle_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.output_note = { gibberish_note, gibberish_note };

    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { user.private_key, user.public_key });

    Composer composer = Composer("../srs_db/ignition");
    join_split_circuit(composer, tx);

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return proof;
}

}
}