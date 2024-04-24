use crate::{
    abis::{
    call_context::CallContext, gas_settings::GasSettings, gas::Gas, max_block_number::MaxBlockNumber,
    nullifier_key_validation_request::NullifierKeyValidationRequest,
    private_circuit_public_inputs::PrivateCircuitPublicInputs, read_request::ReadRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    address::{AztecAddress, compute_initialization_hash}, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, tests::fixtures, transaction::tx_context::TxContext
};
use crate::{
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_ENCRYPTED_LOGS_PER_CALL,
    MAX_UNENCRYPTED_LOGS_PER_CALL
},
    traits::Empty
};

struct PrivateCircuitPublicInputsBuilder {
    call_context: CallContext,

    args_hash: Field,
    returns_hash: Field,

    min_revertible_side_effect_counter: u32,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_note_hashes: BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack_hashes: BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs: BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    encrypted_logs_hashes: BoundedVec<SideEffect, MAX_ENCRYPTED_LOGS_PER_CALL>,
    unencrypted_logs_hashes: BoundedVec<SideEffect, MAX_UNENCRYPTED_LOGS_PER_CALL>,

    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    historical_header: Header,

    chain_id: Field,
    version: Field,

    gas_settings: GasSettings,
}

impl PrivateCircuitPublicInputsBuilder {
    pub fn new() -> Self {
        let mut public_inputs = PrivateCircuitPublicInputsBuilder::empty();

        let args_hash = 0;

        let contract_data = fixtures::contracts::default_contract;

        let contract_function = fixtures::contract_functions::default_private_function;
        let function_data = contract_function.data;

        let contract_address = contract_data.address;

        let call_context = CallContext {
            msg_sender: fixtures::contracts::parent_contract.address,
            storage_contract_address: contract_address,
            function_selector: function_data.selector,
            is_delegate_call: false,
            is_static_call: false,
            side_effect_counter: 0
        };
        public_inputs.call_context = call_context;
        public_inputs.args_hash = args_hash;
        public_inputs.historical_header = fixtures::HEADER;
        public_inputs.chain_id = 0;
        public_inputs.version = 1;
        public_inputs.gas_settings = GasSettings::default();

        public_inputs
    }

    pub fn build_tx_context(self) -> TxContext {
        TxContext::new(self.chain_id, self.version, self.gas_settings)
    }

    pub fn finish(self) -> PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.call_context,
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            max_block_number: self.max_block_number,
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack_hashes: self.private_call_stack_hashes.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.call_context.side_effect_counter,
            end_side_effect_counter: 10,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            historical_header: self.historical_header,
            tx_context: self.build_tx_context()
        }
    }
}

impl Empty for PrivateCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateCircuitPublicInputsBuilder {
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            min_revertible_side_effect_counter: 0 as u32,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_key_validation_requests: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            private_call_stack_hashes: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            new_l2_to_l1_msgs: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            historical_header: Header::empty(),
            chain_id: 0,
            version: 0,
            gas_settings: GasSettings::empty()
        }
    }
}
