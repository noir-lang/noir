#pragma once
#include <stdlib/types/turbo.hpp>
#include "commit.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

using namespace plonk::stdlib::types::turbo;

struct account_note {
    field_ct account_alias_id;
    point_ct account_public_key;
    point_ct signing_pub_key;
    point_ct commitment;

    account_note(field_ct const& account_alias_id, point_ct const& account_public_key, point_ct const& signing_pub_key)
        : account_alias_id(account_alias_id)
        , account_public_key(account_public_key)
        , signing_pub_key(signing_pub_key)
        , commitment(commit(account_alias_id, account_public_key, signing_pub_key))
    {}

    operator byte_array_ct() const { return byte_array_ct(commitment.x).write(commitment.y); }
};

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup