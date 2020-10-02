#include "note_pair.hpp"

namespace rollup {
namespace proofs {
namespace notes {

using namespace barretenberg;

typedef std::pair<private_note, public_note> note_pair;

note_pair create_note_pair(Composer& composer, tx_note const& note)
{
    note_pair result;

    field_ct view_key = witness_ct(&composer, note.secret);
    field_ct note_owner_x = witness_ct(&composer, note.owner.x);
    field_ct note_owner_y = witness_ct(&composer, note.owner.y);
    field_ct witness_value = witness_ct(&composer, note.value);
    field_ct asset_id = witness_ct(&composer, note.asset_id);

    composer.create_range_constraint(asset_id.witness_index, 32);
    composer.create_range_constraint(witness_value.witness_index, NOTE_VALUE_BIT_LENGTH);

    result.first = { { note_owner_x, note_owner_y }, witness_value, view_key, asset_id };
    result.second = encrypt_note(result.first);
    return result;
}

void set_note_public(Composer& composer, public_note const& note)
{
    composer.set_public_input(note.ciphertext.x.witness_index);
    composer.set_public_input(note.ciphertext.y.witness_index);
}

} // namespace notes
} // namespace proofs
} // namespace rollup