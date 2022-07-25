#pragma once
#include "claim_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

inline auto create_partial_commitment(uint256_t const& deposit_value,
                                      uint256_t const& bridge_call_data,
                                      grumpkin::fq const& value_note_partial_commitment,
                                      grumpkin::fq const& input_nullifier)
{
    return crypto::pedersen::compress_native(
        { deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier },
        GeneratorIndex::CLAIM_NOTE_PARTIAL_COMMITMENT);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup