#pragma once
#include "barretenberg/examples/join_split/constants.hpp"
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace bb::join_split_example::proofs::notes::circuit::account {

inline auto commit(field_ct const& account_alias_hash,
                   group_ct const& account_public_key,
                   group_ct const& signing_pub_key)
{
    return pedersen_hash::hash(
        {
            account_alias_hash,
            account_public_key.x,
            signing_pub_key.x,
        },
        GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

} // namespace bb::join_split_example::proofs::notes::circuit::account
