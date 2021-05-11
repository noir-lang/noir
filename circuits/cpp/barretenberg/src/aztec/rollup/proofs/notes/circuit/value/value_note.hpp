#pragma once
#include <stdlib/types/turbo.hpp>
#include "witness_data.hpp"
#include "encrypt.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types::turbo;

struct value_note {
    point_ct owner;
    // note value must be 252 bits or smaller - we assume this is checked elsewhere
    field_ct value;
    // this secret must be 250 bits or smaller - it cannot be taken from the entire field_ct range
    field_ct secret;
    // this asset_id value must be 32 bits or smaller
    field_ct asset_id;
    field_ct nonce;
    point_ct encrypted;

    value_note(witness_data const& note)
        : owner(note.owner)
        , value(note.value)
        , secret(note.secret)
        , asset_id(note.asset_id)
        , nonce(note.nonce)
        , encrypted(encrypt(note))
    {}

    operator byte_array_ct() const { return byte_array_ct(encrypted.x).write(encrypted.y); }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup