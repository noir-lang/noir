#include "note_pair.hpp"

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace barretenberg;
using namespace rollup::pedersen_note;

typedef std::pair<private_note, public_note> note_pair;

note_pair create_note_pair(Composer& composer, tx_note const& note)
{
    note_pair result;

    field_ct view_key = witness_ct(&composer, note.secret);
    field_ct note_owner_x = witness_ct(&composer, note.owner.x);
    field_ct note_owner_y = witness_ct(&composer, note.owner.y);
    field_ct witness_value = witness_ct(&composer, note.value);

    composer.create_range_constraint(witness_value.witness_index, pedersen_note::NOTE_VALUE_BIT_LENGTH);

    result.first = { { note_owner_x, note_owner_y }, witness_value, view_key };
    result.second = encrypt_note(result.first);
    return result;
}

void set_note_public(Composer& composer, public_note const& note)
{
    composer.set_public_input(note.ciphertext.x.witness_index);
    composer.set_public_input(note.ciphertext.y.witness_index);
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup