#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace defi_interaction {

using namespace plonk::stdlib::types::turbo;

/**
 * nonce - randomness provided by the user (sdk) to ensure uniqueness.
 */
inline field_ct compute_dummy_nullifier(field_ct const& defi_interaction_note_commitment, field_ct const& nonce)
{
    // TODO: check if this is ok with Ariel!
    const auto result = pedersen::commit(std::vector<field_ct>{ defi_interaction_note_commitment, nonce },
                                         GeneratorIndex::DEFI_INTERACTION_NOTE_DUMMY_NULLIFIER,
                                         true);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen compression.
    auto blake_input = byte_array_ct(result.x).write(byte_array_ct(result.y));
    auto blake_result = plonk::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup