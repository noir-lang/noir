#pragma once
#include <stdlib/types/turbo.hpp>
#include "../bridge_id.hpp"
#include "witness_data.hpp"
#include "../value/create_partial_commitment.hpp"
#include "create_partial_commitment.hpp"
#include "complete_partial_commitment.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct partial_claim_note {
    field_ct deposit_value;
    field_ct bridge_id;
    field_ct value_note_partial_commitment;
    field_ct partial_commitment;

    partial_claim_note(claim_note_tx_witness_data const& data, point_ct const& owner, field_ct const& owner_nonce)
    {
        deposit_value = data.deposit_value;
        bridge_id = data.bridge_id_data.to_field();
        value_note_partial_commitment = value::create_partial_commitment(data.note_secret, owner, owner_nonce);
        partial_commitment = create_partial_commitment(deposit_value, bridge_id, value_note_partial_commitment);
    }
};

struct claim_note {
    field_ct deposit_value;
    field_ct bridge_id;
    field_ct defi_interaction_nonce;
    field_ct fee;
    field_ct value_note_partial_commitment;
    field_ct commitment;

    claim_note(claim_note_witness_data const& data)
        : deposit_value(data.deposit_value)
        , bridge_id(data.bridge_id_data.to_field())
        , defi_interaction_nonce(data.defi_interaction_nonce)
        , fee(data.fee)
        , value_note_partial_commitment(data.value_note_partial_commitment)
        , commitment(complete_partial_commitment(
              create_partial_commitment(deposit_value, bridge_id, value_note_partial_commitment),
              defi_interaction_nonce,
              fee))
    {}

    operator byte_array_ct() const { return byte_array_ct(commitment); }
};

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup