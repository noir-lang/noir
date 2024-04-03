use crate::{abis::{call_request::CallRequest, side_effect::{SideEffect, SideEffectLinkedToNoteHash}}};
use crate::constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX
};

struct PrivateAccumulatedData {
    new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
}
