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
    return pedersen::compress(std::vector<field_ct>{ defi_interaction_note_commitment, nonce },
                              true,
                              GeneratorIndex::DEFI_INTERACTION_NOTE_DUMMY_NULLIFIER);
}

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup