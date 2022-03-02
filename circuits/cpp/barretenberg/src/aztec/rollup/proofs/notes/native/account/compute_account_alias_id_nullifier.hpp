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

inline fr compute_account_alias_id_nullifier(fr const& account_alias_id)
{
    return crypto::pedersen::compress_native(std::vector<fr>{ account_alias_id },
                                             notes::GeneratorIndex::ACCOUNT_ALIAS_ID_NULLIFIER);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup