#pragma once
#include "../../pedersen_note/pedersen_note.hpp"
#include "tx_note.hpp"

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;
using namespace rollup;

typedef std::pair<pedersen_note::private_note, pedersen_note::public_note> note_pair;

note_pair create_note_pair(Composer& composer, tx_note const& note);

void set_note_public(Composer& composer, pedersen_note::public_note const& note);

} // namespace join_split
} // namespace client_proofs
} // namespace rollup