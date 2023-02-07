#pragma once
#include <stdlib/types/types.hpp>
#include "../../native/value/value_note.hpp"
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types;

struct witness_data {
    point_ct owner;
    suint_ct value;
    field_ct secret;
    suint_ct asset_id;
    bool_ct account_required;
    field_ct creator_pubkey;
    field_ct input_nullifier;

    witness_data(Composer& composer, native::value::value_note const& note)
    {
        secret = witness_ct(&composer, note.secret);
        owner.x = witness_ct(&composer, note.owner.x);
        owner.y = witness_ct(&composer, note.owner.y);
        value = suint_ct(witness_ct(&composer, note.value), NOTE_VALUE_BIT_LENGTH, "note_value");
        asset_id = suint_ct(witness_ct(&composer, note.asset_id), ASSET_ID_BIT_LENGTH, "asset_id");
        account_required = bool_ct(witness_ct(&composer, note.account_required));
        creator_pubkey = witness_ct(&composer, note.creator_pubkey);
        input_nullifier = witness_ct(&composer, note.input_nullifier);
    }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example