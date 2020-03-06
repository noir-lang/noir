#include "note.hpp"

namespace rollup {

typedef std::pair<stdlib::pedersen_note::private_note, stdlib::pedersen_note::public_note> note_pair;

note_pair create_note_pair(Composer& composer, crypto::pedersen_note::private_note const& note)
{
    note_pair result;

    field_t view_key = witness_t(&composer, note.secret);
    field_t note_owner_x = witness_t(&composer, note.owner.x);
    field_t note_owner_y = witness_t(&composer, note.owner.y);
    uint32 witness_value = witness_t(&composer, note.value);
    result.first = { { note_owner_x, note_owner_y }, witness_value, view_key };
    result.second = plonk::stdlib::pedersen_note::encrypt_note(result.first);
    return result;
}

void set_note_public(Composer& composer, stdlib::pedersen_note::public_note const& note)
{
    composer.set_public_input(note.ciphertext.x.witness_index);
    composer.set_public_input(note.ciphertext.y.witness_index);
}

byte_array create_note_leaf(Composer& composer, stdlib::pedersen_note::public_note const& note)
{
    byte_array value_byte_array(&composer);
    value_byte_array.write(note.ciphertext.x).write(note.ciphertext.y);
    return value_byte_array;
}

std::string create_note_db_element(stdlib::pedersen_note::public_note const& note)
{
    // TODO: Compress point.
    std::string new_element = std::string(64, 0);
    fr::serialize_to_buffer(note.ciphertext.x.get_value(), (uint8_t*)(&new_element[0]));
    fr::serialize_to_buffer(note.ciphertext.y.get_value(), (uint8_t*)(&new_element[32]));
    return new_element;
}

}