#pragma once
#include <stdlib/types/turbo.hpp>
#include "../native/defi_interaction_result_note.hpp"
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

struct defi_interaction_result_note {

    // squish bridge_id to field
    field_ct bridge_id;

    // 32 bits
    field_ct interaction_nonce;

    // 252 bits
    field_ct total_input_value;

    // 252 bits
    field_ct total_output_a_value;

    // 252 bits. Force this to be 0 if bridge_id only uses 1 output note
    field_ct total_output_b_value;

    // if interaction failed, re-create original deposit note
    bool_ct interaction_result;
};

// inline claim_note create_value_note_witness(Composer& composer, native::claim_note const& /*input_id*/)
// {
//     // field_ct view_key = witness_ct(&composer, note.secret);
//     // field_ct note_owner_x = witness_ct(&composer, note.owner.x);
//     // field_ct note_owner_y = witness_ct(&composer, note.owner.y);
//     // field_ct witness_value = witness_ct(&composer, note.value);
//     // field_ct asset_id = witness_ct(&composer, note.asset_id);
//     // field_ct nonce = witness_ct(&composer, note.nonce);

//     // composer.create_range_constraint(asset_id.witness_index, 32);
//     // composer.create_range_constraint(witness_value.witness_index, NOTE_VALUE_BIT_LENGTH);

//     // return { { note_owner_x, note_owner_y }, witness_value, view_key, asset_id, nonce };
// }

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup