use crate::{
    abis::{
    call_request::CallRequest, new_contract_data::NewContractData,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_NEW_L2_TO_L1_MSGS_PER_TX, NUM_FIELDS_PER_SHA256, MAX_NEW_CONTRACTS_PER_TX
};

struct PrivateAccumulatedRevertibleData {
    new_note_hashes: [SideEffect; MAX_REVERTIBLE_NOTE_HASHES_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_REVERTIBLE_NULLIFIERS_PER_TX],

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: [NewContractData; MAX_NEW_CONTRACTS_PER_TX],
}

impl PrivateAccumulatedRevertibleData {
    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.public_call_stack[0].is_empty()
    }
}
