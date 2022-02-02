#pragma once
#include "../../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/blake2s/blake2s.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace defi_interaction {

using namespace barretenberg;

/**
 * nonce - randomness provided by the user (sdk) to ensure uniqueness
 */
inline auto compute_dummy_nullifier(grumpkin::fq const& defi_interaction_note_commitment, grumpkin::fq nonce)
{
    return crypto::pedersen::compress_native(std::vector<barretenberg::fr>{ defi_interaction_note_commitment, nonce },
                                             GeneratorIndex::DEFI_INTERACTION_NOTE_DUMMY_NULLIFIER);
}

} // namespace defi_interaction
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup