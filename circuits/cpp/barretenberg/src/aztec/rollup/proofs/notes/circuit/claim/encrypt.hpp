#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"
#include "../pedersen_note.hpp"
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

inline point_ct encrypt(field_ct const& deposit_value,
                        field_ct const& bridge_id,
                        field_ct const& defi_interaction_nonce,
                        point_ct const& partial_state)
{
    point_ct accumulator =
        group_ct::fixed_base_scalar_mul<254>(bridge_id, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_BRIDGE_ID);

    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, deposit_value, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_VALUE);

    accumulator =
        accumulate(accumulator,
                   pedersen::compress_to_point(
                       partial_state.x, partial_state.y, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_PARTIAL_STATE));
    accumulator = conditionally_hash_and_accumulate<32>(
        accumulator, defi_interaction_nonce, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
    return accumulator;
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup