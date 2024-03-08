#pragma once
#include "account_note.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/examples/join_split/constants.hpp"

namespace bb::join_split_example::proofs::notes::native::account {

using namespace bb;

inline fr compute_account_public_key_nullifier(grumpkin::g1::affine_element const& public_key)
{
    return crypto::pedersen_hash::hash(std::vector<fr>{ public_key.x },
                                       notes::GeneratorIndex::ACCOUNT_PUBLIC_KEY_NULLIFIER);
}

} // namespace bb::join_split_example::proofs::notes::native::account
