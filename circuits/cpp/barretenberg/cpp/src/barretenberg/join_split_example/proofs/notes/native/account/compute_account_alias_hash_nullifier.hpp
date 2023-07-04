#pragma once
#include "../../constants.hpp"
#include "account_note.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace account {

using fr = barretenberg::fr;

inline fr compute_account_alias_hash_nullifier(fr const& alias_hash)
{
    return crypto::pedersen_commitment::compress_native(std::vector<fr>{ alias_hash },
                                                        notes::GeneratorIndex::ACCOUNT_ALIAS_HASH_NULLIFIER);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example
