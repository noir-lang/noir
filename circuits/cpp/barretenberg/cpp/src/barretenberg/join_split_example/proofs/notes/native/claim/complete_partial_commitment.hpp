#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

inline auto complete_partial_commitment(grumpkin::fq const& claim_note_partial_commitment,
                                        uint32_t interaction_nonce,
                                        uint256_t fee)
{
    return crypto::pedersen_commitment::compress_native({ claim_note_partial_commitment, interaction_nonce, fee },
                                                        GeneratorIndex::CLAIM_NOTE_COMMITMENT);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example
