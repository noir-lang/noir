#pragma once
#include <stdlib/types/types.hpp>
#include "commit.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

using namespace plonk::stdlib::types;

struct account_note {
    field_ct account_alias_hash;
    point_ct account_public_key;
    point_ct signing_pub_key;
    field_ct commitment;

    account_note(field_ct const& account_alias_hash,
                 point_ct const& account_public_key,
                 point_ct const& signing_pub_key)
        : account_alias_hash(account_alias_hash)
        , account_public_key(account_public_key)
        , signing_pub_key(signing_pub_key)
        , commitment(account::commit(account_alias_hash, account_public_key, signing_pub_key))
    {}

    operator byte_array_ct() const { return byte_array_ct(commitment); }
};

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example