#pragma once
#include "account_note.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/examples/join_split/notes/constants.hpp"

namespace bb::join_split_example::proofs::notes::native::account {

using fr = bb::fr;

inline fr compute_account_alias_hash_nullifier(fr const& alias_hash)
{
    return crypto::pedersen_hash::hash(
        std::vector<fr>{ alias_hash },
        crypto::GeneratorContext<curve::Grumpkin>(notes::GeneratorIndex::ACCOUNT_ALIAS_HASH_NULLIFIER));
}

} // namespace bb::join_split_example::proofs::notes::native::account
