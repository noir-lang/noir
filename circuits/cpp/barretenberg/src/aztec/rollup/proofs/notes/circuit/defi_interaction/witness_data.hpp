#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/defi_interaction/note.hpp"
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace defi_interaction {

using namespace plonk::stdlib::types::turbo;

struct witness_data {
    bridge_id bridge_id_data;
    suint_ct interaction_nonce;
    suint_ct total_input_value;
    suint_ct total_output_a_value;
    suint_ct total_output_b_value;
    bool_ct interaction_result;

    witness_data(Composer& composer, native::defi_interaction::note const& note_data)
    {
        bridge_id_data = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        interaction_nonce = suint_ct(
            witness_ct(&composer, note_data.interaction_nonce), DEFI_INTERACTION_NONCE_BIT_LENGTH, "interaction_nonce");
        total_input_value =
            suint_ct(witness_ct(&composer, note_data.total_input_value), NOTE_VALUE_BIT_LENGTH, "total_input_value");
        total_output_a_value = suint_ct(
            witness_ct(&composer, note_data.total_output_a_value), NOTE_VALUE_BIT_LENGTH, "total_output_value_a");
        total_output_b_value = suint_ct(
            witness_ct(&composer, note_data.total_output_b_value), NOTE_VALUE_BIT_LENGTH, "total_output_value_b");
        interaction_result = witness_ct(&composer, note_data.interaction_result);
    }
};

} // namespace defi_interaction
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup