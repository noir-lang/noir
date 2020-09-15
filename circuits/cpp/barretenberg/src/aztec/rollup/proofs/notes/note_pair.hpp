#pragma once
#include "pedersen_note.hpp"
#include "tx_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {

using namespace plonk::stdlib::types::turbo;

typedef std::pair<private_note, public_note> note_pair;

note_pair create_note_pair(Composer& composer, tx_note const& note);

void set_note_public(Composer& composer, public_note const& note);

} // namespace notes
} // namespace proofs
} // namespace rollup