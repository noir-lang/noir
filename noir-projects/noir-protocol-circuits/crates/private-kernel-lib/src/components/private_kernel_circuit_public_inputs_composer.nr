use dep::types::{
    abis::{
    call_request::CallRequest, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder},
    max_block_number::MaxBlockNumber, nullifier::{Nullifier, ScopedNullifier},
    private_circuit_public_inputs::PrivateCircuitPublicInputs
},
    address::AztecAddress,
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NOTE_HASHES_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL
},
    hash::{
    silo_encrypted_log_hash, silo_l2_to_l1_message, silo_note_hash, silo_nullifier,
    silo_unencrypted_log_hash
},
    traits::is_empty, transaction::tx_request::TxRequest,
    utils::arrays::{array_length, array_to_bounded_vec, sort_by_counters_asc, sort_by_counters_desc}
};

struct DataSource {
    private_call_public_inputs: PrivateCircuitPublicInputs,
    contract_address: AztecAddress,
    storage_contract_address: AztecAddress,
    public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    public_teardown_call_request: CallRequest,
}

pub fn create_first_nullifier(tx_request: TxRequest) -> ScopedNullifier {
    Nullifier { value: tx_request.hash(), note_hash: 0, counter: 0 }.scope(AztecAddress::zero())
}

struct PrivateKernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub fn new_from_tx_request(
        tx_request: TxRequest,
        private_call_public_inputs: PrivateCircuitPublicInputs,
        vk_tree_root: Field
    ) -> Self {
        let mut public_inputs = PrivateKernelCircuitPublicInputsBuilder::empty();

        public_inputs.constants = CombinedConstantData::private(
            private_call_public_inputs.historical_header,
            tx_request.tx_context,
            vk_tree_root,
        );

        // Since it's the first iteration, we need to push the tx hash nullifier into the `nullifiers` array
        public_inputs.end.nullifiers.push(create_first_nullifier(tx_request));
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
        public_inputs.end.note_hashes = array_to_bounded_vec(start.note_hashes);
        public_inputs.end.nullifiers = array_to_bounded_vec(start.nullifiers);
        public_inputs.end.l2_to_l1_msgs = array_to_bounded_vec(start.l2_to_l1_msgs);
        public_inputs.end.note_encrypted_logs_hashes = array_to_bounded_vec(start.note_encrypted_logs_hashes);
        public_inputs.end.encrypted_logs_hashes = array_to_bounded_vec(start.encrypted_logs_hashes);
        public_inputs.end.unencrypted_logs_hashes = array_to_bounded_vec(start.unencrypted_logs_hashes);
        public_inputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);
        public_inputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub fn pop_top_call_request(&mut self) -> Self {
        // Pop the top item in the call stack, which is the caller of the current call, and shouldn't be propagated to the output.
        let _call_request = self.public_inputs.end.private_call_stack.pop();
        *self
    }

    pub fn with_private_call(
        &mut self,
        private_call_public_inputs: PrivateCircuitPublicInputs,
        contract_address: AztecAddress,
        public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
        public_teardown_call_request: CallRequest
    ) -> Self {
        let storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
        let source = DataSource {
            private_call_public_inputs,
            contract_address,
            storage_contract_address,
            public_call_requests,
            public_teardown_call_request
        };

        self.propagate_from_private_call(source);

        *self
    }

    pub fn sort_and_silo(&mut self) {
        self.sort_ordered_values();
        self.silo_scoped_values();
    }

    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        self.public_inputs.finish()
    }

    fn propagate_from_private_call(&mut self, source: DataSource) {
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
        self.propagate_min_revertible_side_effect_counter(source);
    }

    fn propagate_min_revertible_side_effect_counter(&mut self, source: DataSource) {
        if self.public_inputs.min_revertible_side_effect_counter != 0 {
            assert(
                source.private_call_public_inputs.min_revertible_side_effect_counter == 0, "cannot overwrite non-zero min_revertible_side_effect_counter"
            );
        } else {
            self.public_inputs.min_revertible_side_effect_counter = source.private_call_public_inputs.min_revertible_side_effect_counter;
        };
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
        let note_hashes = source.private_call_public_inputs.note_hashes;
        for i in 0..note_hashes.len() {
            let note_hash = note_hashes[i];
            if note_hash.value != 0 {
                self.public_inputs.end.note_hashes.push(note_hash.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_nullifiers(&mut self, source: DataSource) {
        let nullifiers = source.private_call_public_inputs.nullifiers;
        for i in 0..nullifiers.len() {
            let nullifier = nullifiers[i];
            if nullifier.value != 0 {
                self.public_inputs.end.nullifiers.push(nullifier.scope(source.storage_contract_address));
            }
        }
    }

    fn propagate_l2_to_l1_messages(&mut self, source: DataSource) {
        let l2_to_l1_msgs = source.private_call_public_inputs.l2_to_l1_msgs;
        for i in 0..l2_to_l1_msgs.len() {
            let msg = l2_to_l1_msgs[i];
            if !is_empty(msg) {
                self.public_inputs.end.l2_to_l1_msgs.push(msg.scope(source.storage_contract_address));
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
                self.public_inputs.end.note_encrypted_logs_hashes.push(note_logs[i]);
            }
        }
    }

    fn propagate_private_call_requests(&mut self, source: DataSource) {
        let call_requests = source.private_call_public_inputs.private_call_requests;
        let num_requests = array_length(call_requests);
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

    fn sort_ordered_values(&mut self) {
        self.public_inputs.end.note_hashes.storage = sort_by_counters_asc(self.public_inputs.end.note_hashes.storage);
        self.public_inputs.end.nullifiers.storage = sort_by_counters_asc(self.public_inputs.end.nullifiers.storage);
        self.public_inputs.end.l2_to_l1_msgs.storage = sort_by_counters_asc(self.public_inputs.end.l2_to_l1_msgs.storage);
        self.public_inputs.end.note_encrypted_logs_hashes.storage = sort_by_counters_asc(self.public_inputs.end.note_encrypted_logs_hashes.storage);
        self.public_inputs.end.encrypted_logs_hashes.storage = sort_by_counters_asc(self.public_inputs.end.encrypted_logs_hashes.storage);
        self.public_inputs.end.unencrypted_logs_hashes.storage = sort_by_counters_asc(self.public_inputs.end.unencrypted_logs_hashes.storage);
        self.public_inputs.end.public_call_stack.storage = sort_by_counters_desc(self.public_inputs.end.public_call_stack.storage);
    }

    fn silo_scoped_values(&mut self) {
        self.silo_note_hashes();
        self.silo_nullifiers();
        self.silo_l2_to_l1_messages();
        self.silo_encrypted_logs();
        self.silo_unencrypted_logs();
    }

    fn silo_note_hashes(&mut self) {
        let first_nullifier = self.public_inputs.end.nullifiers.get_unchecked(0).value();
        let note_hashes = self.public_inputs.end.note_hashes.storage;
        for i in 0..note_hashes.len() {
            self.public_inputs.end.note_hashes.storage[i].note_hash.value = silo_note_hash(
                note_hashes[i],
                first_nullifier,
                i
            );
        }
    }

    fn silo_nullifiers(&mut self) {
        let nullifiers = self.public_inputs.end.nullifiers.storage;
        for i in 0..nullifiers.len() {
            self.public_inputs.end.nullifiers.storage[i].nullifier.value = silo_nullifier(nullifiers[i]);
        }
    }

    fn silo_l2_to_l1_messages(&mut self) {
        let l2_to_l1_msgs = self.public_inputs.end.l2_to_l1_msgs.storage;
        let tx_context = self.public_inputs.constants.tx_context;
        for i in 0..l2_to_l1_msgs.len() {
            self.public_inputs.end.l2_to_l1_msgs.storage[i].message.content = silo_l2_to_l1_message(
                l2_to_l1_msgs[i],
                tx_context.version,
                tx_context.chain_id,
            );
        }
    }

    fn silo_encrypted_logs(&mut self) {
        let logs = self.public_inputs.end.encrypted_logs_hashes.storage;
        for i in 0..logs.len() {
            self.public_inputs.end.encrypted_logs_hashes.storage[i].log_hash.value = silo_encrypted_log_hash(logs[i]);
        }
    }

    fn silo_unencrypted_logs(&mut self) {
        let logs = self.public_inputs.end.unencrypted_logs_hashes.storage;
        for i in 0..logs.len() {
            self.public_inputs.end.unencrypted_logs_hashes.storage[i].log_hash.value = silo_unencrypted_log_hash(logs[i]);
        }
    }
}
