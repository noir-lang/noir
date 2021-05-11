#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"
#include "../pedersen_note.hpp"
#include "bridge_id.hpp"
#include "witness_data.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace defi_interaction {

using namespace plonk::stdlib::types::turbo;

inline point_ct encrypt(witness_data const& plaintext)
{
    point_ct accumulator =
        group_ct::fixed_base_scalar_mul<254>(plaintext.bridge_id_data, GeneratorIndex::DEFI_INTERACTION_NOTE_BRIDGE_ID);

    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, plaintext.total_input_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_INPUT_VALUE);
    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, plaintext.total_output_a_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_A_VALUE);
    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, plaintext.total_output_b_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_B_VALUE);
    accumulator = conditionally_hash_and_accumulate<32>(
        accumulator, plaintext.interaction_nonce, GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_NONCE);
    accumulator = conditionally_hash_and_accumulate<1>(
        accumulator, field_ct(plaintext.interaction_result), GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_RESULT);

    return accumulator;
}

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup