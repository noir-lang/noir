#pragma once
#include "../pedersen_note/pedersen_note.hpp"
#include "../tx/tx_note.hpp"

namespace rollup {
namespace prover {

using namespace plonk::stdlib::types::turbo;
using namespace rollup;

typedef std::pair<pedersen_note::private_note, pedersen_note::public_note> note_pair;

note_pair create_note_pair(Composer& composer, tx::tx_note const& note);

void set_note_public(Composer& composer, pedersen_note::public_note const& note);

byte_array_ct create_note_leaf(Composer& composer, pedersen_note::public_note const& note);

std::string create_note_db_element(pedersen_note::public_note const& note);

}
}