#pragma once
#include "../../native/value/value_note.hpp"
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example::proofs::notes::circuit::value {

using namespace bb::stdlib;

struct witness_data {
    group_ct owner;
    suint_ct value;
    field_ct secret;
    suint_ct asset_id;
    bool_ct account_required;
    field_ct creator_pubkey;
    field_ct input_nullifier;

    witness_data(Builder& builder, native::value::value_note const& note)
    {
        secret = witness_ct(&builder, note.secret);
        owner = group_ct(witness_ct(&builder, note.owner.x), witness_ct(&builder, note.owner.y), false);
        owner.x = witness_ct(&builder, note.owner.x);
        owner.y = witness_ct(&builder, note.owner.y);
        value = suint_ct(witness_ct(&builder, note.value), NOTE_VALUE_BIT_LENGTH, "note_value");
        asset_id = suint_ct(witness_ct(&builder, note.asset_id), ASSET_ID_BIT_LENGTH, "asset_id");
        account_required = bool_ct(witness_ct(&builder, note.account_required));
        creator_pubkey = witness_ct(&builder, note.creator_pubkey);
        input_nullifier = witness_ct(&builder, note.input_nullifier);
    }
};

} // namespace join_split_example::proofs::notes::circuit::value
