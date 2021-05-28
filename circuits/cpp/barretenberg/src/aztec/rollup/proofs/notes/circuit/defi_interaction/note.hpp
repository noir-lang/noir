#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/defi_interaction/note.hpp"
#include "../pedersen_note.hpp"
#include "witness_data.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace defi_interaction {

using namespace plonk::stdlib::types::turbo;

struct note {

    // compress bridge_id to field
    field_ct bridge_id;

    // 32 bits
    field_ct interaction_nonce;

    // 252 bits
    field_ct total_input_value;

    // 252 bits
    field_ct total_output_a_value;

    // 252 bits. Force this to be 0 if bridge_id only uses 1 output note
    field_ct total_output_b_value;

    // if interaction failed, re-create original deposit note
    bool_ct interaction_result;

    // encrypted defi_interaction_note
    point_ct encrypted;

    note(witness_data const& note)
        : bridge_id(note.bridge_id_data.to_field())
        , interaction_nonce(note.interaction_nonce)
        , total_input_value(note.total_input_value)
        , total_output_a_value(note.total_output_a_value)
        , total_output_b_value(note.total_output_b_value)
        , interaction_result(note.interaction_result)
        , encrypted(encrypt())
    {}

    operator byte_array_ct() const { return byte_array_ct(encrypted.x).write(encrypted.y); }

    byte_array_ct to_byte_array(Composer& composer, bool_ct is_real = 1) const
    {
        byte_array_ct arr(&composer);

        arr.write((bridge_id * is_real).normalize());
        arr.write((interaction_nonce * is_real).normalize());
        arr.write((total_input_value * is_real).normalize());
        arr.write((total_output_a_value * is_real).normalize());
        arr.write((total_output_b_value * is_real).normalize());
        arr.write((field_ct(interaction_result) * is_real).normalize());

        return arr;
    }

  private:
    point_ct encrypt()
    {
        point_ct accumulator =
            group_ct::fixed_base_scalar_mul<254>(bridge_id, GeneratorIndex::DEFI_INTERACTION_NOTE_BRIDGE_ID);

        accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
            accumulator, total_input_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_INPUT_VALUE);
        accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
            accumulator, total_output_a_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_A_VALUE);
        accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
            accumulator, total_output_b_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_B_VALUE);
        accumulator = conditionally_hash_and_accumulate<32>(
            accumulator, interaction_nonce, GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_NONCE);
        accumulator = conditionally_hash_and_accumulate<2>(
            accumulator, field_ct(interaction_result), GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_RESULT);

        return accumulator;
    }
};

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup