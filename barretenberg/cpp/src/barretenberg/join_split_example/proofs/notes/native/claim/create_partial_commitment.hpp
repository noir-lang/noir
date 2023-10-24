#pragma once
#include "../../constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "claim_note.hpp"

namespace join_split_example::proofs::notes::native::claim {

inline auto create_partial_commitment(uint256_t const& deposit_value,
                                      uint256_t const& bridge_call_data,
                                      grumpkin::fq const& value_note_partial_commitment,
                                      grumpkin::fq const& input_nullifier)
{
    return crypto::pedersen_hash::hash(
        { deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier },
        GeneratorIndex::CLAIM_NOTE_PARTIAL_COMMITMENT);
}

} // namespace join_split_example::proofs::notes::native::claim
