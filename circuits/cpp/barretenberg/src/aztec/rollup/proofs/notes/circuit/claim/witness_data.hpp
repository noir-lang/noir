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

/**
 * Convert native claim note data into circuit witness data.
 * Used in the claim circuit where the input is an actual, fully committed, claim note.
 */
struct claim_note_witness_data {
    field_ct deposit_value;
    bridge_id bridge_id_data;
    field_ct defi_interaction_nonce;
    field_ct fee;
    field_ct value_note_partial_commitment;
    field_ct input_nullifier;

    claim_note_witness_data(Composer& composer, native::claim::claim_note const& note_data)
    {
        deposit_value = witness_ct(&composer, note_data.deposit_value);
        bridge_id_data = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        defi_interaction_nonce = witness_ct(&composer, note_data.defi_interaction_nonce);
        fee = witness_ct(&composer, note_data.fee);
        value_note_partial_commitment = witness_ct(&composer, note_data.value_note_partial_commitment);
        input_nullifier = witness_ct(&composer, note_data.input_nullifier);

        deposit_value.create_range_constraint(DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi deposit too large");
        defi_interaction_nonce.create_range_constraint(DEFI_TREE_DEPTH, "defi interaction nonce too large");
        fee.create_range_constraint(TX_FEE_BIT_LENGTH, "claim fee too large");
    }
};

/**
 * Convert native claim note tx data into circuit witness data.
 * Used in the join split circuit to create a partial claim note commitment.
 */
struct claim_note_tx_witness_data {
    field_ct deposit_value;
    bridge_id bridge_id_data;
    field_ct note_secret;
    field_ct input_nullifier;

    claim_note_tx_witness_data(Composer& composer, native::claim::claim_note_tx_data const& note_data)
    {
        deposit_value = witness_ct(&composer, note_data.deposit_value);
        bridge_id_data = bridge_id::from_uint256_t(composer, note_data.bridge_id);
        note_secret = witness_ct(&composer, note_data.note_secret);
        input_nullifier = witness_ct(&composer, note_data.input_nullifier);

        deposit_value.create_range_constraint(DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi deposit too large");
    }
};

inline std::ostream& operator<<(std::ostream& os, claim_note_tx_witness_data const& tx)
{
    return os << "{ deposit_value: " << tx.deposit_value << ", bridge_id: " << tx.bridge_id_data.to_field() << " }";
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup