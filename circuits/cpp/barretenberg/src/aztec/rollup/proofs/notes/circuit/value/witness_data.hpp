#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/value_note.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types::turbo;

struct witness_data {
    point_ct owner;
    // note value must be 252 bits or smaller - we assume this is checked elsewhere
    field_ct value;
    // this secret must be 250 bits or smaller - it cannot be taken from the entire field_ct range
    field_ct secret;
    // this asset_id value must be 32 bits or smaller
    field_ct asset_id;
    field_ct nonce;

    static witness_data from_tx_data(Composer& composer, native::value_note const& note)
    {
        field_ct view_key = witness_ct(&composer, note.secret);
        field_ct note_owner_x = witness_ct(&composer, note.owner.x);
        field_ct note_owner_y = witness_ct(&composer, note.owner.y);
        field_ct witness_value = witness_ct(&composer, note.value);
        field_ct asset_id = witness_ct(&composer, note.asset_id);
        field_ct nonce = witness_ct(&composer, note.nonce);

        composer.create_range_constraint(asset_id.witness_index, 32);
        composer.create_range_constraint(witness_value.witness_index, NOTE_VALUE_BIT_LENGTH);

        return { { note_owner_x, note_owner_y }, witness_value, view_key, asset_id, nonce };
    }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup