#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/value/value_note.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types::turbo;

struct witness_data {
    point_ct owner;
    field_ct value;
    field_ct secret;
    field_ct asset_id;
    field_ct nonce;
    field_ct creator_pubkey;
    field_ct input_nullifier;

    witness_data(Composer& composer, native::value::value_note const& note)
    {
        secret = witness_ct(&composer, note.secret);
        owner.x = witness_ct(&composer, note.owner.x);
        owner.y = witness_ct(&composer, note.owner.y);
        value = witness_ct(&composer, note.value);
        asset_id = witness_ct(&composer, note.asset_id);
        nonce = witness_ct(&composer, note.nonce);
        creator_pubkey = witness_ct(&composer, note.creator_pubkey);
        input_nullifier = witness_ct(&composer, note.input_nullifier);

        asset_id.create_range_constraint(ASSET_ID_BIT_LENGTH);
        value.create_range_constraint(NOTE_VALUE_BIT_LENGTH);
    }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup