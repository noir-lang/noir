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

inline point_ct complete_partial_value_note(point_ct const& partial_note,
                                            field_ct const& value,
                                            field_ct const& asset_id)
{
    auto accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        partial_note, value, GeneratorIndex::JOIN_SPLIT_NOTE_VALUE);
    accumulator =
        conditionally_hash_and_accumulate<32>(accumulator, asset_id, GeneratorIndex::JOIN_SPLIT_NOTE_ASSET_ID);
    return accumulator;
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup