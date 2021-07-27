#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

inline auto complete_partial_commitment(grumpkin::fq const& claim_note_partial_commitment,
                                        uint32_t interaction_nonce,
                                        uint256_t fee)
{
    return crypto::pedersen::compress_native({ claim_note_partial_commitment, interaction_nonce, fee },
                                             GeneratorIndex::CLAIM_NOTE_COMMITMENT);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup