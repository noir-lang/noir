#pragma once
#include <barretenberg/waffle/stdlib/crypto/commitment/pedersen_note.hpp>
#include "types.hpp"

namespace rollup {

typedef std::pair<stdlib::pedersen_note::private_note, stdlib::pedersen_note::public_note> note_pair;

note_pair create_note_pair(Composer& composer, crypto::pedersen_note::private_note const& note);

void set_note_public(Composer& composer, stdlib::pedersen_note::public_note const& note);

byte_array create_note_leaf(Composer& composer, stdlib::pedersen_note::public_note const& note);

std::string create_note_db_element(stdlib::pedersen_note::public_note const& note);

}