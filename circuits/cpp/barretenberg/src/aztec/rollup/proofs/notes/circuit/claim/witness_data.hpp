#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/claim/claim_note.hpp"
#include "../../native/claim/claim_note_tx_data.hpp"
#include "../../constants.hpp"
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct claim_note_witness_data {
    field_ct deposit_value;
    bridge_id bridge_id_data;
    field_ct defi_interaction_nonce;
    point_ct partial_state;

    claim_note_witness_data(Composer& composer, native::claim::claim_note const& note_data)
    {
        deposit_value = witness_ct(&composer, note_data.deposit_value);
        bridge_id_data = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        defi_interaction_nonce = witness_ct(&composer, note_data.defi_interaction_nonce);
        partial_state = { witness_ct(&composer, note_data.partial_state.x),
                          witness_ct(&composer, note_data.partial_state.y) };

        composer.create_range_constraint(deposit_value.witness_index, NOTE_VALUE_BIT_LENGTH);
        composer.create_range_constraint(defi_interaction_nonce.witness_index, 32);
    }
};

struct claim_note_tx_witness_data {
    field_ct deposit_value;
    bridge_id bridge_id_data;
    field_ct note_secret;
    field_ct defi_interaction_nonce;

    claim_note_tx_witness_data(Composer& composer, native::claim::claim_note_tx_data const& note_data)
    {
        deposit_value = witness_ct(&composer, note_data.deposit_value);
        bridge_id_data = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        note_secret = witness_ct(&composer, note_data.note_secret);
        defi_interaction_nonce = witness_ct(&composer, note_data.defi_interaction_nonce);

        composer.create_range_constraint(deposit_value.witness_index, NOTE_VALUE_BIT_LENGTH);
        composer.create_range_constraint(defi_interaction_nonce.witness_index, 32);
    }
};

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup