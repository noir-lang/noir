use crate::{
    abis::{
    call_context::CallContext, public_circuit_public_inputs::PublicCircuitPublicInputs,
    read_request::ReadRequest, side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest}, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, tests::fixtures
};
use crate::constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL
};

struct PublicCircuitPublicInputsBuilder {
    call_context: CallContext,
    args_hash: Field,
    returns_hash: Field,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    nullifier_non_existent_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL>,
    contract_storage_update_requests: BoundedVec<StorageUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>,
    contract_storage_reads: BoundedVec<StorageRead, MAX_PUBLIC_DATA_READS_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_note_hashes: BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL>,
    new_l2_to_l1_msgs: BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    start_side_effect_counter: u32,
    end_side_effect_counter: u32,
    unencrypted_logs_hash: Field,
    unencrypted_log_preimages_length: Field,
    historical_header: Header,
    prover_address: AztecAddress,
    revert_code: u8,
}

impl PublicCircuitPublicInputsBuilder {
    pub fn new() -> Self {
        let mut public_inputs: PublicCircuitPublicInputsBuilder = dep::std::unsafe::zeroed();
        public_inputs.call_context.msg_sender = fixtures::MSG_SENDER;
        public_inputs.historical_header = fixtures::HEADER;
        public_inputs
    }

    pub fn finish(self) -> PublicCircuitPublicInputs {
        PublicCircuitPublicInputs {
            call_context: self.call_context,
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            contract_storage_update_requests: self.contract_storage_update_requests.storage,
            contract_storage_reads: self.contract_storage_reads.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.start_side_effect_counter,
            end_side_effect_counter: self.end_side_effect_counter,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            historical_header: self.historical_header,
            prover_address: self.prover_address,
            revert_code: self.revert_code
        }
    }
}
