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
    // note value must be 252 bits or smaller - we assume this is checked elsewhere
    field_ct value;
    // this secret must be 250 bits or smaller - it cannot be taken from the entire field_ct range
    field_ct secret;
    // this asset_id value must be 32 bits or smaller
    field_ct asset_id;
    field_ct nonce;

    witness_data(Composer& composer, native::value::value_note const& note)
    {
        secret = witness_ct(&composer, note.secret);
        owner.x = witness_ct(&composer, note.owner.x);
        owner.y = witness_ct(&composer, note.owner.y);
        value = witness_ct(&composer, note.value);
        asset_id = witness_ct(&composer, note.asset_id);
        nonce = witness_ct(&composer, note.nonce);

        composer.create_range_constraint(asset_id.witness_index, 32);
        composer.create_range_constraint(value.witness_index, NOTE_VALUE_BIT_LENGTH);
    }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup