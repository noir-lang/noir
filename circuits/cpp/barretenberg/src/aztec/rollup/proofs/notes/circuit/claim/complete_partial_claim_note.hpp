#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"
#include "encrypt.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

inline point_ct complete_partial_claim_note(point_ct const& claim_note, field_ct const& interaction_nonce)
{
    return notes::circuit::conditionally_hash_and_accumulate<32>(
        claim_note, interaction_nonce, notes::GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup