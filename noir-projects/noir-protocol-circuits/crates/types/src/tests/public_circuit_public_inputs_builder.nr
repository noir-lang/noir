use crate::{
    abis::{
    gas::Gas, call_context::CallContext, note_hash::NoteHash, nullifier::Nullifier,
    public_circuit_public_inputs::PublicCircuitPublicInputs, read_request::ReadRequest,
    log_hash::LogHash, global_variables::GlobalVariables
},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest}, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, tests::fixtures
};
use crate::{
    constants::{
    MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_NOTE_HASHES_PER_CALL, MAX_L2_TO_L1_MSGS_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL
},
    traits::Empty
};

struct PublicCircuitPublicInputsBuilder {
    call_context: CallContext,
    args_hash: Field,
    returns_hash: Field,
    note_hash_read_requests: BoundedVec<ReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    nullifier_non_existent_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL>,
    l1_to_l2_msg_read_requests: BoundedVec<ReadRequest, MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL>,
    contract_storage_update_requests: BoundedVec<StorageUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>,
    contract_storage_reads: BoundedVec<StorageRead, MAX_PUBLIC_DATA_READS_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    note_hashes: BoundedVec<NoteHash, MAX_NOTE_HASHES_PER_CALL>,
    nullifiers: BoundedVec<Nullifier, MAX_NULLIFIERS_PER_CALL>,
    l2_to_l1_msgs: BoundedVec<L2ToL1Message, MAX_L2_TO_L1_MSGS_PER_CALL>,
    start_side_effect_counter: u32,
    end_side_effect_counter: u32,
    unencrypted_logs_hashes: BoundedVec<LogHash, MAX_UNENCRYPTED_LOGS_PER_CALL>,
    historical_header: Header,
    global_variables: GlobalVariables,
    prover_address: AztecAddress,
    revert_code: u8,
    start_gas_left: Gas,
    end_gas_left: Gas,
    transaction_fee: Field,
}

impl PublicCircuitPublicInputsBuilder {
    pub fn new() -> Self {
        let mut public_inputs = PublicCircuitPublicInputsBuilder::empty();
        public_inputs.call_context.msg_sender = fixtures::MSG_SENDER;
        public_inputs
    }

    pub fn finish(self) -> PublicCircuitPublicInputs {
        PublicCircuitPublicInputs {
            call_context: self.call_context,
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            l1_to_l2_msg_read_requests: self.l1_to_l2_msg_read_requests.storage,
            contract_storage_update_requests: self.contract_storage_update_requests.storage,
            contract_storage_reads: self.contract_storage_reads.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            note_hashes: self.note_hashes.storage,
            nullifiers: self.nullifiers.storage,
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage,
            start_side_effect_counter: self.start_side_effect_counter,
            end_side_effect_counter: self.end_side_effect_counter,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            historical_header: self.historical_header,
            global_variables: self.global_variables,
            prover_address: self.prover_address,
            revert_code: self.revert_code,
            start_gas_left: self.start_gas_left,
            end_gas_left: self.end_gas_left,
            transaction_fee: self.transaction_fee
        }
    }
}

impl Empty for PublicCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PublicCircuitPublicInputsBuilder {
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            l1_to_l2_msg_read_requests: BoundedVec::new(),
            contract_storage_update_requests: BoundedVec::new(),
            contract_storage_reads: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            l2_to_l1_msgs: BoundedVec::new(),
            start_side_effect_counter: 0 as u32,
            end_side_effect_counter: 0 as u32,
            unencrypted_logs_hashes: BoundedVec::new(),
            historical_header: Header::empty(),
            global_variables: GlobalVariables::empty(),
            prover_address: AztecAddress::zero(),
            revert_code: 0 as u8,
            start_gas_left: Gas::empty(),
            end_gas_left: Gas::empty(),
            transaction_fee: 0
        }
    }
}
