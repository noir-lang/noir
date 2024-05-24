use dep::types::{
    abis::{
    call_request::CallRequest, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder},
    max_block_number::MaxBlockNumber, nullifier::Nullifier, note_hash::ScopedNoteHash,
    log_hash::NoteLogHash,
    private_circuit_public_inputs::{PrivateCircuitPublicInputs, PrivateCircuitPublicInputsArrayLengths}
},
    address::AztecAddress,
    constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL
},
    traits::is_empty, transaction::tx_request::TxRequest,
    utils::arrays::{array_to_bounded_vec, find_index}
};

struct DataSource {
    private_call_public_inputs: PrivateCircuitPublicInputs,
    array_lengths: PrivateCircuitPublicInputsArrayLengths,
    contract_address: AztecAddress,
    storage_contract_address: AztecAddress,
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
    public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    public_teardown_call_request: CallRequest,
}

struct PrivateKernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub fn new_from_tx_request(tx_request: TxRequest, private_call_public_inputs: PrivateCircuitPublicInputs) -> Self {
        let mut public_inputs = PrivateKernelCircuitPublicInputsBuilder::empty();

        public_inputs.constants = CombinedConstantData::private(
            private_call_public_inputs.historical_header,
            tx_request.tx_context,
        );

        public_inputs.min_revertible_side_effect_counter = private_call_public_inputs.min_revertible_side_effect_counter;

        // Since it's the first iteration, we need to push the the tx hash nullifier into the `new_nullifiers` array
        public_inputs.end.new_nullifiers.push(Nullifier { value: tx_request.hash(), note_hash: 0, counter: 0 }.scope(AztecAddress::zero()));
        // Note that we do not need to nullify the transaction request nonce anymore.
        // Should an account want to additionally use nonces for replay protection or handling cancellations,
        // they will be able to do so in the account contract logic:
        // https://github.com/AztecProtocol/aztec-packages/issues/660

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub fn new_from_previous_kernel(previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs) -> Self {
        let mut public_inputs = PrivateKernelCircuitPublicInputsBuilder::empty();

        public_inputs.constants = previous_kernel_public_inputs.constants;
        public_inputs.min_revertible_side_effect_counter = previous_kernel_public_inputs.min_revertible_side_effect_counter;
        public_inputs.fee_payer = previous_kernel_public_inputs.fee_payer;
        public_inputs.public_teardown_call_request = previous_kernel_public_inputs.public_teardown_call_request;

        let start = previous_kernel_public_inputs.validation_requests;
        public_inputs.validation_requests.max_block_number = start.for_rollup.max_block_number;
        public_inputs.validation_requests.note_hash_read_requests = array_to_bounded_vec(start.note_hash_read_requests);
        public_inputs.validation_requests.nullifier_read_requests = array_to_bounded_vec(start.nullifier_read_requests);
        public_inputs.validation_requests.scoped_key_validation_requests_and_generators = array_to_bounded_vec(start.scoped_key_validation_requests_and_generators);

        let start = previous_kernel_public_inputs.end;
        public_inputs.end.new_note_hashes = array_to_bounded_vec(start.new_note_hashes);
        public_inputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);
        public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);
        public_inputs.end.note_encrypted_logs_hashes = array_to_bounded_vec(start.note_encrypted_logs_hashes);
        public_inputs.end.encrypted_logs_hashes = array_to_bounded_vec(start.encrypted_logs_hashes);
        public_inputs.end.unencrypted_logs_hashes = array_to_bounded_vec(start.unencrypted_logs_hashes);
        public_inputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);
        let _call_request = public_inputs.end.private_call_stack.pop();
        public_inputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub fn compose(
        &mut self,
        private_call_public_inputs: PrivateCircuitPublicInputs,
        array_lengths: PrivateCircuitPublicInputsArrayLengths,
        contract_address: AztecAddress,
        note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
        public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
        public_teardown_call_request: CallRequest
    ) -> Self {
        let storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
        let source = DataSource {
            private_call_public_inputs,
            array_lengths,
            contract_address,
            storage_contract_address,
            note_hash_nullifier_counters,
            public_call_requests,
            public_teardown_call_request
        };

        self.propagate_max_block_number(source);
        self.propagate_note_hash_read_requests(source);
        self.propagate_nullifier_read_requests(source);
        self.propagate_key_validation_requests(source);
        self.propagate_note_hashes(source);
        self.propagate_nullifiers(source);
        self.propagate_l2_to_l1_messages(source);
        self.propagate_logs(source);
        self.propagate_private_call_requests(source);
        self.propagate_public_call_requests(source);
        self.propagate_public_teardown_call_request(source);
        self.propagate_fee_payer(source);

        *self
    }

    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        self.public_inputs.finish()
    }

    fn propagate_max_block_number(&mut self, source: DataSource) {
        // Update the max block number if the private call requested a lower one.
        self.public_inputs.validation_requests.max_block_number = MaxBlockNumber::min(self.public_inputs.validation_requests.max_block_number, source.private_call_public_inputs.max_block_number);
    }

    fn propagate_note_hash_read_requests(&mut self, source: DataSource) {
        let read_requests = source.private_call_public_inputs.note_hash_read_requests;
        for i in 0..read_requests.len() {
            let request = read_requests[i];
            if !is_empty(request) {
                self.public_inputs.validation_requests.note_hash_read_requests.push(request.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_nullifier_read_requests(&mut self, source: DataSource) {
        let nullifier_read_requests = source.private_call_public_inputs.nullifier_read_requests;
        for i in 0..nullifier_read_requests.len() {
            let request = nullifier_read_requests[i];
            if !is_empty(request) {
                self.public_inputs.validation_requests.nullifier_read_requests.push(request.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_key_validation_requests(&mut self, source: DataSource) {
        let key_validation_requests_and_generators = source.private_call_public_inputs.key_validation_requests_and_generators;
        for i in 0..key_validation_requests_and_generators.len() {
            let request = key_validation_requests_and_generators[i];
            if !is_empty(request) {
                self.public_inputs.validation_requests.scoped_key_validation_requests_and_generators.push(request.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_note_hashes(&mut self, source: DataSource) {
        let note_hashes = source.private_call_public_inputs.new_note_hashes;
        for i in 0..note_hashes.len() {
            let mut note_hash = note_hashes[i];
            if note_hash.value != 0 {
                let nullifier_counter = source.note_hash_nullifier_counters[i];
                assert(
                    (nullifier_counter == 0) | (nullifier_counter > note_hash.counter), "Invalid nullifier counter"
                );
                self.public_inputs.end.new_note_hashes.push(note_hash.scope(nullifier_counter, source.storage_contract_address));
            }
        }
    }

    fn propagate_nullifiers(&mut self, source: DataSource) {
        let nullifiers = source.private_call_public_inputs.new_nullifiers;
        for i in 0..nullifiers.len() {
            let nullifier = nullifiers[i];
            if nullifier.value != 0 {
                self.public_inputs.end.new_nullifiers.push(nullifier.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_l2_to_l1_messages(&mut self, source: DataSource) {
        let l2_to_l1_msgs = source.private_call_public_inputs.new_l2_to_l1_msgs;
        for i in 0..l2_to_l1_msgs.len() {
            let msg = l2_to_l1_msgs[i];
            if !is_empty(msg) {
                self.public_inputs.end.new_l2_to_l1_msgs.push(msg.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_logs(&mut self, source: DataSource) {
        let encrypted_logs = source.private_call_public_inputs.encrypted_logs_hashes;
        for i in 0..encrypted_logs.len() {
            let log = encrypted_logs[i];
            if !is_empty(log) {
                self.public_inputs.end.encrypted_logs_hashes.push(log.scope(source.storage_contract_address));
            }
        }
        let unencrypted_logs = source.private_call_public_inputs.unencrypted_logs_hashes;
        for i in 0..unencrypted_logs.len() {
            let log = unencrypted_logs[i];
            if !is_empty(log) {
                self.public_inputs.end.unencrypted_logs_hashes.push(log.scope(source.storage_contract_address));
            }
        }

        let note_logs = source.private_call_public_inputs.note_encrypted_logs_hashes;
        for i in 0..note_logs.len() {
            if !is_empty(note_logs[i]) {
                let note_index = self.match_log_to_note(note_logs[i]);
                assert(
                    note_index < MAX_NEW_NOTE_HASHES_PER_TX, "Could not find note hash linked to note log."
                );
                assert(
                    note_logs[i].note_hash_counter
                    == self.public_inputs.end.new_note_hashes.get_unchecked(note_index).counter(), "Could not find note hash linked to note log."
                );
                self.public_inputs.end.note_encrypted_logs_hashes.push(note_logs[i]);
            }
        }
    }

    unconstrained fn match_log_to_note(self, note_log: NoteLogHash) -> u64 {
        find_index(
            self.public_inputs.end.new_note_hashes.storage,
            |n: ScopedNoteHash| n.counter() == note_log.note_hash_counter
        )
    }

    fn propagate_private_call_requests(&mut self, source: DataSource) {
        let call_requests = source.private_call_public_inputs.private_call_requests;
        let num_requests = source.array_lengths.private_call_requests;
        let mut proceed = true;
        for i in 0..call_requests.len() {
            proceed &= i != num_requests;
            if proceed {
                // Push the call requests to the stack in reverse order.
                let call_request = call_requests[num_requests - i - 1];
                self.public_inputs.end.private_call_stack.push(call_request.scope(source.contract_address));
            }
        }
    }

    fn propagate_public_call_requests(&mut self, source: DataSource) {
        let call_requests = source.public_call_requests;
        for i in 0..call_requests.len() {
            let call_request = call_requests[i];
            if !is_empty(call_request) {
                self.public_inputs.end.public_call_stack.push(call_request);
            }
        }
    }

    fn propagate_public_teardown_call_request(&mut self, source: DataSource) {
        let call_request = source.public_teardown_call_request;
        if !is_empty(call_request) {
            assert(
                is_empty(self.public_inputs.public_teardown_call_request), "Public teardown call request already set"
            );
            self.public_inputs.public_teardown_call_request = call_request;
        }
    }

    fn propagate_fee_payer(&mut self, source: DataSource) {
        if (source.private_call_public_inputs.is_fee_payer) {
            assert(self.public_inputs.fee_payer.is_zero(), "Cannot overwrite non-empty fee_payer");
            self.public_inputs.fee_payer = source.storage_contract_address;
        }
    }
}
