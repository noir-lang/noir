use crate::{
    abis::{
    call_context::CallContext, call_request::CallRequest, caller_context::CallerContext,
    gas_settings::GasSettings, gas::Gas, max_block_number::MaxBlockNumber, note_hash::NoteHash,
    nullifier::Nullifier,
    validation_requests::key_validation_request_and_generator::KeyValidationRequestAndGenerator,
    private_call_request::PrivateCallRequest, private_circuit_public_inputs::PrivateCircuitPublicInputs,
    read_request::ReadRequest, log_hash::{LogHash, NoteLogHash, EncryptedLogHash}
},
    address::{AztecAddress, compute_initialization_hash}, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, tests::fixtures, transaction::tx_context::TxContext
};
use crate::{
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_ENCRYPTED_LOGS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL,
    MAX_NOTE_ENCRYPTED_LOGS_PER_CALL
},
    traits::Empty
};

struct PrivateCircuitPublicInputsBuilder {
    contract_address: AztecAddress,
    call_context: CallContext,

    args_hash: Field,
    returns_hash: Field,

    min_revertible_side_effect_counter: u32,
    is_fee_payer: bool,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: BoundedVec<ReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    key_validation_requests_and_generators: BoundedVec<KeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_note_hashes: BoundedVec<NoteHash, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<Nullifier, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_requests: BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    public_teardown_function_hash: Field,
    new_l2_to_l1_msgs: BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    note_encrypted_logs_hashes: BoundedVec<NoteLogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_CALL>,
    encrypted_logs_hashes: BoundedVec<EncryptedLogHash, MAX_ENCRYPTED_LOGS_PER_CALL>,
    unencrypted_logs_hashes: BoundedVec<LogHash, MAX_UNENCRYPTED_LOGS_PER_CALL>,

    historical_header: Header,

    chain_id: Field,
    version: Field,

    gas_settings: GasSettings,

    counter_start: u32,
    counter_end: u32,
}

impl PrivateCircuitPublicInputsBuilder {
    pub fn new() -> Self {
        PrivateCircuitPublicInputsBuilder::new_with_counter(0)
    }

    pub fn new_with_counter(counter: u32) -> Self {
        let mut public_inputs = PrivateCircuitPublicInputsBuilder::empty();

        let contract_data = fixtures::contracts::default_contract;
        let contract_function = fixtures::contract_functions::default_private_function;
        let function_data = contract_function.data;
        let contract_address = contract_data.address;

        public_inputs.contract_address = contract_address;

        public_inputs.call_context = CallContext {
            msg_sender: fixtures::contracts::parent_contract.address,
            storage_contract_address: contract_address,
            function_selector: function_data.selector,
            is_delegate_call: false,
            is_static_call: false,
            side_effect_counter: counter
        };

        public_inputs.chain_id = fixtures::CHAIN_ID;
        public_inputs.version = fixtures::VERSION;
        public_inputs.gas_settings = GasSettings::empty();
        public_inputs.counter_start = counter;
        public_inputs.counter_end = counter + 1;

        public_inputs
    }

    pub fn is_delegate_call(&mut self) -> Self {
        self.call_context.is_delegate_call = true;
        self.call_context.storage_contract_address = fixtures::contracts::parent_contract.address;
        self.call_context.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        self.call_context.is_static_call = true;
        *self
    }

    pub fn build_tx_context(self) -> TxContext {
        TxContext::new(self.chain_id, self.version, self.gas_settings)
    }

    pub fn end_setup(&mut self) {
        self.min_revertible_side_effect_counter = self.counter_end;
    }

    pub fn set_tx_max_block_number(&mut self, max_block_number: u32) {
        self.max_block_number = MaxBlockNumber::new(max_block_number);
    }

    pub fn append_note_hash_read_requests(&mut self, num_reads: u64) {
        let value_offset = self.note_hash_read_requests.len();
        for i in 0..self.note_hash_read_requests.max_len() {
            if i < num_reads {
                let read_request = ReadRequest { value: (value_offset + i + 987) as Field, counter: self.next_counter() };
                self.note_hash_read_requests.push(read_request);
            }
        }
    }

    pub fn append_nullifier_read_requests(&mut self, num_reads: u64) {
        let value_offset = self.nullifier_read_requests.len();
        for i in 0..self.nullifier_read_requests.max_len() {
            if i < num_reads {
                let read_request = ReadRequest { value: (value_offset + i + 3344) as Field, counter: self.next_counter() };
                self.nullifier_read_requests.push(read_request);
            }
        }
    }

    pub fn add_new_note_hash(&mut self, value: Field) {
        self.new_note_hashes.push(NoteHash { value, counter: self.next_counter() });
    }

    pub fn append_new_note_hashes(&mut self, count: u64) {
        let value_offset = self.new_note_hashes.len();
        for i in 0..self.new_note_hashes.max_len() {
            if i < count {
                let mocked_value = 123123 + value_offset as Field;
                self.add_new_note_hash(mocked_value);
            }
        }
    }

    pub fn add_encrypted_log(&mut self, hash: Field, preimages_length: Field) {
        let side_effect = EncryptedLogHash { value: hash, counter: self.next_counter(), length: preimages_length, randomness: 2 };
        self.encrypted_logs_hashes.push(side_effect);
    }

    pub fn add_unencrypted_log(&mut self, hash: Field, preimages_length: Field) {
        let side_effect = LogHash { value: hash, counter: self.next_counter(), length: preimages_length };
        self.unencrypted_logs_hashes.push(side_effect);
    }

    pub fn generate_call_request(self, hash: Field, is_delegate_call: bool) -> CallRequest {
        let mut caller_context = CallerContext::empty();
        if is_delegate_call {
            caller_context.msg_sender = self.call_context.msg_sender;
            caller_context.storage_contract_address = self.call_context.storage_contract_address;
        }
        CallRequest {
            hash,
            caller_contract_address: self.contract_address,
            caller_context,
            start_side_effect_counter: self.counter_end,
            end_side_effect_counter: self.counter_end + 1
        }
    }

    pub fn add_private_call_request(&mut self, hash: Field, is_delegate_call: bool) {
        let mut caller_context = CallerContext::empty();
        caller_context.is_static_call = self.call_context.is_static_call;
        if is_delegate_call {
            caller_context.msg_sender = self.call_context.msg_sender;
            caller_context.storage_contract_address = self.call_context.storage_contract_address;
        }
        self.private_call_requests.push(
            PrivateCallRequest {
            hash,
            caller_context,
            start_side_effect_counter: self.next_counter(),
            end_side_effect_counter: self.next_counter()
        }
        );
    }

    pub fn append_private_call_requests(&mut self, num_requests: u64, is_delegate_call: bool) {
        let hash_offset = 7070 + self.private_call_requests.len();
        for i in 0..self.private_call_requests.max_len() {
            if i < num_requests {
                let hash = (hash_offset + i) as Field;
                self.add_private_call_request(hash, is_delegate_call);
            }
        }
    }

    pub fn add_public_call_request(&mut self, hash: Field) {
        let _ = self.next_counter(); // Increment for creating the call
        self.public_call_stack_hashes.push(hash);
        let _ = self.next_counter(); // Increment for ending the call.
    }

    pub fn add_teardown_call_request(&mut self, hash: Field) {
        let _ = self.next_counter(); // Increment for creating the call
        self.public_teardown_function_hash = hash;
        let _ = self.next_counter(); // Increment for ending the call.
    }

    pub fn finish(self) -> PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.call_context,
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            is_fee_payer: self.is_fee_payer,
            max_block_number: self.max_block_number,
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            key_validation_requests_and_generators: self.key_validation_requests_and_generators.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_requests: self.private_call_requests.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            public_teardown_function_hash: self.public_teardown_function_hash,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.counter_start,
            end_side_effect_counter: self.counter_end,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            historical_header: self.historical_header,
            tx_context: self.build_tx_context()
        }
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.counter_end;
        self.counter_end += 1;
        counter
    }
}

impl Empty for PrivateCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateCircuitPublicInputsBuilder {
            contract_address: AztecAddress::empty(),
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            min_revertible_side_effect_counter: 0 as u32,
            is_fee_payer: false,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            key_validation_requests_and_generators: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            private_call_requests: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            new_l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            historical_header: Header::empty(),
            chain_id: 0,
            version: 0,
            gas_settings: GasSettings::empty(),
            counter_start: 0,
            counter_end: 0
        }
    }
}
