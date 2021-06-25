#pragma once
#include "../pedersen_note.hpp"
#include "witness_data.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

inline point_ct commit(const witness_data& plaintext)
{
    point_ct accumulator =
        group_ct::fixed_base_scalar_mul<254>(plaintext.secret, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);

    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, plaintext.value, GeneratorIndex::JOIN_SPLIT_NOTE_VALUE);
    accumulator = conditionally_hash_and_accumulate<32>(
        accumulator, plaintext.asset_id, GeneratorIndex::JOIN_SPLIT_NOTE_ASSET_ID);
    accumulator = accumulate(
        accumulator,
        pedersen::compress_to_point(plaintext.owner.x, plaintext.owner.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER));
    accumulator =
        conditionally_hash_and_accumulate<32>(accumulator, plaintext.nonce, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);
    return accumulator;
}

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup