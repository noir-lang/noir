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
    const std::vector<fr> hash_elements{ fr(ProofIds::ACCOUNT), account_alias_id, fr(0) };
    return crypto::pedersen::compress_native(hash_elements, notes::GeneratorIndex::ACCOUNT_ALIAS_ID_NULLIFIER);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup