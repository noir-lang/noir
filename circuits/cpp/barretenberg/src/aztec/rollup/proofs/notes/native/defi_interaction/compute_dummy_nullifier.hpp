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
    // TODO: check if this is ok with Ariel!
    const auto result =
        crypto::pedersen::commit_native(std::vector<barretenberg::fr>{ defi_interaction_note_commitment, nonce },
                                        GeneratorIndex::DEFI_INTERACTION_NOTE_DUMMY_NULLIFIER);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen compression.
    auto blake_result = blake2::blake2s(to_buffer(result));

    return from_buffer<fr>(blake_result);
}

} // namespace defi_interaction
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup