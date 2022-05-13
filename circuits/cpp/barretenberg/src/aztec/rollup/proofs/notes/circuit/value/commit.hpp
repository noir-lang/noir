#pragma once
#include "witness_data.hpp"
#include "create_partial_commitment.hpp"
#include "complete_partial_commitment.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

inline auto commit(const witness_data& plaintext)
{
    auto partial_commitment =
        create_partial_commitment(plaintext.secret, plaintext.owner, plaintext.account_nonce, plaintext.creator_pubkey);
    return complete_partial_commitment(
        partial_commitment, plaintext.value, plaintext.asset_id, plaintext.input_nullifier);
}

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup