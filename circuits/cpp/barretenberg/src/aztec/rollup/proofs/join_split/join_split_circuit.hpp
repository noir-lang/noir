#pragma once
#include "join_split_tx.hpp"
#include "../notes/circuit/claim_note.hpp"
// #include "../notes/circuit/note_pair.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct join_split_inputs {
    field_ct public_input;
    field_ct public_output;
    field_ct asset_id;
    uint32_ct num_input_notes;
    field_ct input_note1_index;
    field_ct input_note2_index;
    notes::circuit::value_note input_note1;
    notes::circuit::value_note input_note2;
    notes::circuit::value_note output_note1;
    notes::circuit::value_note output_note2;
    notes::circuit::claim_note_tx_data_ct claim_note_tx_data;
    point_ct signing_pub_key;
    stdlib::schnorr::signature_bits<Composer> signature;
    field_ct merkle_root;
    merkle_tree::hash_path input_path1;
    merkle_tree::hash_path input_path2;
    field_ct account_index;
    merkle_tree::hash_path account_path;
    field_ct input_owner;
    field_ct output_owner;
    field_ct account_private_key;
    field_ct alias_hash;
    field_ct nonce;
};

struct join_split_outputs {
    field_ct proof_id;
    field_ct nullifier1;
    field_ct nullifier2;
    field_ct tx_fee;
    field_ct public_input;
    field_ct public_output;
    point_ct output_note1;
    point_ct output_note2;
    field_ct asset_id;
};

join_split_outputs join_split_circuit_component(Composer& composer, join_split_inputs const& inputs);

void join_split_circuit(Composer& composer, join_split_tx const& tx);

} // namespace join_split
} // namespace proofs
} // namespace rollup
