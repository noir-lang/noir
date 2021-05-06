#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/claim_note.hpp"
#include "../../constants.hpp"
#include "bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct witness_data {
    field_ct deposit_value;
    bridge_id bridge_id;
    field_ct note_secret;
    field_ct defi_interaction_nonce;

    static witness_data from_tx_data(Composer& composer, native::claim_note_tx_data const& note_data)
    {
        auto deposit_value = witness_ct(&composer, note_data.deposit_value);
        auto bridge_id = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        auto note_secret = witness_ct(&composer, note_data.note_secret);
        auto defi_interaction_nonce = witness_ct(&composer, note_data.defi_interaction_nonce);

        composer.create_range_constraint(deposit_value.witness_index, NOTE_VALUE_BIT_LENGTH);
        composer.create_range_constraint(defi_interaction_nonce.witness_index, 32);

        return { deposit_value, bridge_id, note_secret, defi_interaction_nonce };
    }
};

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup