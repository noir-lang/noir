#pragma once
#include "barretenberg/examples/join_split/types.hpp"

#include "barretenberg/examples/join_split/notes/circuit/bridge_call_data.hpp"
#include "barretenberg/examples/join_split/notes/circuit/value/create_partial_commitment.hpp"
#include "complete_partial_commitment.hpp"
#include "create_partial_commitment.hpp"
#include "witness_data.hpp"

namespace bb::join_split_example::proofs::notes::circuit::claim {

using namespace bb::stdlib;

struct partial_claim_note {
    suint_ct deposit_value;
    suint_ct bridge_call_data;
    field_ct value_note_partial_commitment;
    field_ct input_nullifier;
    field_ct partial_commitment;

    partial_claim_note(partial_claim_note_witness_data const& data,
                       group_ct const& owner,
                       bool_ct const& owner_account_required)
    {
        deposit_value = data.deposit_value;
        bridge_call_data = data.bridge_call_data_local.to_safe_uint();
        value_note_partial_commitment =
            value::create_partial_commitment(data.note_secret, owner, owner_account_required, 0);
        input_nullifier = data.input_nullifier;
        partial_commitment =
            create_partial_commitment(deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier);
    }
};

struct claim_note {
    suint_ct deposit_value;
    suint_ct bridge_call_data;
    field_ct value_note_partial_commitment;
    field_ct input_nullifier;
    suint_ct defi_interaction_nonce;
    suint_ct fee;
    field_ct commitment;

    claim_note(claim_note_witness_data const& data)
        : deposit_value(data.deposit_value)
        , bridge_call_data(data.bridge_call_data_local.to_safe_uint())
        , value_note_partial_commitment(data.value_note_partial_commitment)
        , input_nullifier(data.input_nullifier)
        , defi_interaction_nonce(data.defi_interaction_nonce)
        , fee(data.fee)
        , commitment(complete_partial_commitment(
              create_partial_commitment(
                  deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier),
              defi_interaction_nonce,
              fee))
    {}

    operator byte_array_ct() const { return byte_array_ct(commitment); }
};

} // namespace bb::join_split_example::proofs::notes::circuit::claim
