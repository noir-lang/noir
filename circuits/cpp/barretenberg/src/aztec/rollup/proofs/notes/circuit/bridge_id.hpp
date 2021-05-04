#pragma once
#include <stdlib/types/turbo.hpp>
#include "../native/bridge_id.hpp"
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

struct bridge_id {

    // bridge_id(const native::bridge_id& )
    // {
    //     // TODO format witnesses and do range checks
    // }

    // TODO: range constrain to be 20 bytes (160 bits)
    field_ct bridge_contract_address;
    // TODO: range constrain to be 2 bits (1 or 2)
    field_ct num_output_notes;

    // TODO: 32 bit range check
    field_ct input_asset_id;

    // TODO: 32 bit range check
    field_ct output_asset_id_a;

    // TODO: 32 bit range check
    field_ct output_asset_id_b;

    field_ct to_field(Composer& /*composer*/) const { return field_ct(0); }
};

// inline bridge_id create_value_note_witness(Composer& composer, native::bridge_id const& /*input_id*/)
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