#pragma once
#include "join_split_tx.hpp"
#include "../notes/circuit/value/witness_data.hpp"
#include "../notes/circuit/claim/witness_data.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct join_split_inputs {
    field_ct proof_id;
    suint_ct public_value;
    field_ct public_owner;
    suint_ct asset_id;
    field_ct num_input_notes;
    suint_ct input_note1_index;
    suint_ct input_note2_index;
    notes::circuit::value::witness_data input_note1;
    notes::circuit::value::witness_data input_note2;
    notes::circuit::value::witness_data output_note1;
    notes::circuit::value::witness_data output_note2;
    notes::circuit::claim::partial_claim_note_witness_data partial_claim_note;
    point_ct signing_pub_key;
    stdlib::schnorr::signature_bits<Composer> signature;
    field_ct merkle_root;
    merkle_tree::hash_path input_path1;
    merkle_tree::hash_path input_path2;
    suint_ct account_index;
    merkle_tree::hash_path account_path;
    field_ct account_private_key;
    suint_ct alias_hash;
    suint_ct nonce;
    field_ct backward_link;
    field_ct allow_chain;
};

struct join_split_outputs {
    field_ct nullifier1;
    field_ct nullifier2;
    field_ct output_note1;
    field_ct output_note2;
    field_ct public_asset_id;
    field_ct tx_fee;
    field_ct bridge_id;
    field_ct defi_deposit_value;
};

join_split_outputs join_split_circuit_component(join_split_inputs const& inputs);

void join_split_circuit(Composer& composer, join_split_tx const& tx);

} // namespace join_split
} // namespace proofs
} // namespace rollup
