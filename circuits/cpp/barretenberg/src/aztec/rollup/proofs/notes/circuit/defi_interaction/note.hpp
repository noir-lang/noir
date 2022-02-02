#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/defi_interaction/note.hpp"
#include "witness_data.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace defi_interaction {

using namespace plonk::stdlib::types::turbo;

struct note {

    // compress bridge_id to field
    suint_ct bridge_id;

    // 32 bits
    suint_ct interaction_nonce;

    // 252 bits
    suint_ct total_input_value;

    // 252 bits
    suint_ct total_output_value_a;

    // 252 bits. Force this to be 0 if bridge_id only uses 1 output note
    suint_ct total_output_value_b;

    // if interaction failed, re-create original deposit note
    bool_ct interaction_result;

    // commitment to the defi_interaction_note
    field_ct commitment;

    note(witness_data const& note)
        : bridge_id(note.bridge_id_data.to_safe_uint())
        , interaction_nonce(note.interaction_nonce)
        , total_input_value(note.total_input_value)
        , total_output_value_a(note.total_output_value_a)
        , total_output_value_b(note.total_output_value_b)
        , interaction_result(note.interaction_result)
        , commitment(compute_commitment())
    {}

    operator byte_array_ct() const { return byte_array_ct(commitment); }

    byte_array_ct to_byte_array(Composer& composer, bool_ct is_real = 1) const
    {
        byte_array_ct arr(&composer);

        arr.write(bridge_id * is_real);
        arr.write(interaction_nonce * is_real);
        arr.write(total_input_value * is_real);
        arr.write(total_output_value_a * is_real);
        arr.write(total_output_value_b * is_real);
        arr.write((field_ct(interaction_result) * is_real));

        return arr;
    }

  private:
    field_ct compute_commitment()
    {
        return pedersen::compress({ bridge_id,
                                    total_input_value,
                                    total_output_value_a,
                                    total_output_value_b,
                                    interaction_nonce,
                                    interaction_result },
                                  true,
                                  GeneratorIndex::DEFI_INTERACTION_NOTE_COMMITMENT);
    }
};

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup