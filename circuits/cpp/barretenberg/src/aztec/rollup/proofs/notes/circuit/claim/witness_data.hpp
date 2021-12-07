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
    suint_ct deposit_value;
    bridge_id bridge_id_data;
    suint_ct defi_interaction_nonce;
    suint_ct fee;
    field_ct value_note_partial_commitment;
    field_ct input_nullifier;

    claim_note_witness_data(Composer& composer, native::claim::claim_note const& note_data)
    {
        deposit_value =
            suint_ct(witness_ct(&composer, note_data.deposit_value), DEFI_DEPOSIT_VALUE_BIT_LENGTH, "deposit_value");
        bridge_id_data = bridge_id(&composer, note_data.bridge_id);
        defi_interaction_nonce = suint_ct(witness_ct(&composer, note_data.defi_interaction_nonce),
                                          DEFI_INTERACTION_NONCE_BIT_LENGTH,
                                          "defi_interaction_nonce");
        fee = suint_ct(witness_ct(&composer, note_data.fee), TX_FEE_BIT_LENGTH, "fee");
        value_note_partial_commitment = witness_ct(&composer, note_data.value_note_partial_commitment);
        input_nullifier = witness_ct(&composer, note_data.input_nullifier);
    }
};

/**
 * Convert native claim note tx data into circuit witness data.
 * Used in the join split circuit to create a partial claim note commitment.
 */
struct claim_note_tx_witness_data {
    suint_ct deposit_value;
    bridge_id bridge_id_data;
    field_ct note_secret;
    field_ct input_nullifier;

    claim_note_tx_witness_data(){};
    claim_note_tx_witness_data(Composer& composer, native::claim::claim_note_tx_data const& note_data)
    {
        deposit_value =
            suint_ct(witness_ct(&composer, note_data.deposit_value), DEFI_DEPOSIT_VALUE_BIT_LENGTH, "deposit_value");
        bridge_id_data = bridge_id(&composer, note_data.bridge_id);
        note_secret = witness_ct(&composer, note_data.note_secret);
        input_nullifier = witness_ct(&composer, note_data.input_nullifier);
    }
};

inline std::ostream& operator<<(std::ostream& os, claim_note_tx_witness_data const& tx)
{
    return os << "{ deposit_value: " << tx.deposit_value << ", bridge_id: " << tx.bridge_id_data.to_safe_uint() << " }";
}
} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup