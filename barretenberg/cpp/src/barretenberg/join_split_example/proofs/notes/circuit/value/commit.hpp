#pragma once
#include "complete_partial_commitment.hpp"
#include "create_partial_commitment.hpp"
#include "witness_data.hpp"

namespace join_split_example::proofs::notes::circuit::value {

inline auto commit(const witness_data& plaintext)
{
    auto partial_commitment = create_partial_commitment(
        plaintext.secret, plaintext.owner, plaintext.account_required, plaintext.creator_pubkey);
    return complete_partial_commitment(
        partial_commitment, plaintext.value, plaintext.asset_id, plaintext.input_nullifier);
}

} // namespace join_split_example::proofs::notes::circuit::value