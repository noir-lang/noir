#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/claim_note.hpp"
#include "../../constants.hpp"
#include "encrypt.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

point_ct create_partial_value_note(field_ct const& secret, field_ct const& nonce, point_ct const& owner)
{
    point_ct accumulator = group_ct::fixed_base_scalar_mul<254>(secret, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);
    accumulator =
        accumulate(accumulator, pedersen::compress_to_point(owner.x, owner.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER));
    accumulator = conditionally_hash_and_accumulate<32>(accumulator, nonce, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);
    return accumulator;
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup