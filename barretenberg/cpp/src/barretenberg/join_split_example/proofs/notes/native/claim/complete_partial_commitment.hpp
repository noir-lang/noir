#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example::proofs::notes::native::claim {

inline auto complete_partial_commitment(grumpkin::fq const& claim_note_partial_commitment,
                                        uint32_t interaction_nonce,
                                        uint256_t fee)
{
    return crypto::pedersen_hash::hash({ claim_note_partial_commitment, interaction_nonce, fee },
                                       GeneratorIndex::CLAIM_NOTE_COMMITMENT);
}

} // namespace join_split_example::proofs::notes::native::claim
