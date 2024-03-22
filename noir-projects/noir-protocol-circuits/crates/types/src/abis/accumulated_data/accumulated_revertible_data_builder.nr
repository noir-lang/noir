use crate::{
    abis::{
    accumulated_data::{
    private_accumulated_revertible_data::PrivateAccumulatedRevertibleData,
    public_accumulated_revertible_data::PublicAccumulatedRevertibleData
},
    call_request::CallRequest, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

struct AccumulatedRevertibleDataBuilder {
    new_note_hashes: BoundedVec<SideEffect, MAX_REVERTIBLE_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_REVERTIBLE_NULLIFIERS_PER_TX>,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,
    
    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
}

impl AccumulatedRevertibleDataBuilder {
    pub fn to_private(self) -> PrivateAccumulatedRevertibleData {
        PrivateAccumulatedRevertibleData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length
        }
    }

    pub fn to_public(self) -> PublicAccumulatedRevertibleData {
        PublicAccumulatedRevertibleData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            public_data_update_requests: self.public_data_update_requests.storage
        }
    }
}
