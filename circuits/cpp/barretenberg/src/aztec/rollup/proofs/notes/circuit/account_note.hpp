#pragma once
#include <stdlib/types/turbo.hpp>
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

struct account_note {
    field_ct account_alias_id;
    point_ct account_public_key;
    point_ct signing_pub_key;
};

inline point_ct encrypt_note(account_note const& account_note)
{
    std::vector<field_ct> leaf_elements{
        account_note.account_alias_id,
        account_note.account_public_key.x,
        account_note.signing_pub_key.x,
    };
    return pedersen::encrypt(leaf_elements, GeneratorIndex::ACCOUNT_NOTE_HASH_INPUTS, true);
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup