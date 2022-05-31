#pragma once
#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace account {

using namespace barretenberg;

inline fr compute_account_alias_hash_nullifier(fr const& alias_hash)
{
    return crypto::pedersen::compress_native(std::vector<fr>{ alias_hash },
                                             notes::GeneratorIndex::ACCOUNT_ALIAS_HASH_NULLIFIER);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup