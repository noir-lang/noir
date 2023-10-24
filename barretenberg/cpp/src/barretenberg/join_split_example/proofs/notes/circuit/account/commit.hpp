#pragma once
#include "../../constants.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

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

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example
