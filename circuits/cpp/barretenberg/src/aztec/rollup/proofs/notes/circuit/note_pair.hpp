#pragma once
#include "../native/claim_note.hpp"
#include "../native/value_note.hpp"
#include "claim_note.hpp"
#include "value_note.hpp"
#include "encrypt_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

typedef std::pair<value_note, point_ct> note_pair;
typedef std::pair<claim_note, point_ct> claim_note_pair;

inline note_pair create_note_pair(Composer& composer, native::value_note const& note)
{
    auto note_witness = create_value_note_witness(composer, note);
    auto enc_note = encrypt_note(note_witness);
    return { note_witness, enc_note };
}

inline claim_note_pair create_note_pair(Composer& composer, native::claim_note const& note) {}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup