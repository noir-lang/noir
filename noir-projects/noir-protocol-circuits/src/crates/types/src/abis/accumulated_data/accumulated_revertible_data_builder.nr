use crate::{
    abis::{
    accumulated_data::{
    private_accumulated_revertible_data::PrivateAccumulatedRevertibleData,
    public_accumulated_revertible_data::PublicAccumulatedRevertibleData
},
    call_request::CallRequest, new_contract_data::NewContractData,
    nullifier_key_validation_request::NullifierKeyValidationRequestContext,
    public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX, MAX_NEW_CONTRACTS_PER_TX,
    NUM_FIELDS_PER_SHA256, MAX_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX,
    MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

struct AccumulatedRevertibleDataBuilder {
    read_requests: BoundedVec<SideEffect, MAX_READ_REQUESTS_PER_TX>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequestContext, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,

    new_note_hashes: BoundedVec<SideEffect, MAX_REVERTIBLE_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_REVERTIBLE_NULLIFIERS_PER_TX>,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: BoundedVec<NewContractData, MAX_NEW_CONTRACTS_PER_TX>,

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX>,
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
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage
        }
    }

    pub fn to_public(self) -> PublicAccumulatedRevertibleData {
        PublicAccumulatedRevertibleData {
            read_requests: self.read_requests.storage,
            new_note_hashes: self.new_note_hashes.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage,
            public_data_update_requests: self.public_data_update_requests.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }
}
