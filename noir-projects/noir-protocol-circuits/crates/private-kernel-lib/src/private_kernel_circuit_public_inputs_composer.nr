use dep::types::{
    abis::{
    call_request::CallRequest, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder},
    max_block_number::MaxBlockNumber, nullifier::Nullifier,
    private_circuit_public_inputs::PrivateCircuitPublicInputs
},
    address::AztecAddress,
    constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL
},
    hash::{compute_l2_to_l1_hash, silo_note_hash, silo_nullifier}, traits::is_empty,
    transaction::tx_request::TxRequest, utils::arrays::array_to_bounded_vec
};

struct DataSource {
    private_call_public_inputs: PrivateCircuitPublicInputs,
    storage_contract_address: AztecAddress,
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
    private_call_requests: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
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
        public_inputs.end.new_nullifiers.push(Nullifier { value: tx_request.hash(), note_hash: 0, counter: 0 });
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

        let start = previous_kernel_public_inputs.validation_requests;
        public_inputs.validation_requests.max_block_number = start.for_rollup.max_block_number;
        public_inputs.validation_requests.note_hash_read_requests = array_to_bounded_vec(start.note_hash_read_requests);
        public_inputs.validation_requests.nullifier_read_requests = array_to_bounded_vec(start.nullifier_read_requests);
        public_inputs.validation_requests.nullifier_key_validation_requests = array_to_bounded_vec(start.nullifier_key_validation_requests);

        let start = previous_kernel_public_inputs.end;
        public_inputs.end.new_note_hashes = array_to_bounded_vec(start.new_note_hashes);
        public_inputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);
        public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);
        public_inputs.end.encrypted_logs_hashes = array_to_bounded_vec(start.encrypted_logs_hashes);
        public_inputs.end.unencrypted_logs_hashes = array_to_bounded_vec(start.unencrypted_logs_hashes);
        public_inputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
        public_inputs.end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;
        public_inputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);
        let _call_request = public_inputs.end.private_call_stack.pop();
        public_inputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub fn compose(
        &mut self,
        private_call_public_inputs: PrivateCircuitPublicInputs,
        note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
        private_call_requests: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
        public_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]
    ) -> Self {
        let storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
        let source = DataSource {
            storage_contract_address,
            private_call_public_inputs,
            note_hash_nullifier_counters,
            private_call_requests,
            public_call_requests
        };

        self.propagate_max_block_number(source);
        self.propagate_note_hash_read_requests(source);
        self.propagate_nullifier_read_requests(source);
        self.propagate_nullifier_key_validation_requests(source);
        self.propagate_note_hashes(source);
        self.propagate_nullifiers(source);
        self.propagate_l2_to_l1_messages(source);
        self.propagate_logs(source);
        self.propagate_private_call_requests(source);
        self.propagate_public_call_requests(source);

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
                self.public_inputs.validation_requests.note_hash_read_requests.push(request.to_context(source.storage_contract_address));
            }
        }
    }

    fn propagate_nullifier_read_requests(&mut self, source: DataSource) {
        let nullifier_read_requests = source.private_call_public_inputs.nullifier_read_requests;
        for i in 0..nullifier_read_requests.len() {
            let request = nullifier_read_requests[i];
            if !is_empty(request) {
                self.public_inputs.validation_requests.nullifier_read_requests.push(request.to_context(source.storage_contract_address));
            }
        }
    }

    fn propagate_nullifier_key_validation_requests(&mut self, source: DataSource) {
        let nullifier_key_validation_requests = source.private_call_public_inputs.nullifier_key_validation_requests;
        for i in 0..nullifier_key_validation_requests.len() {
            let request = nullifier_key_validation_requests[i];
            if !is_empty(request) {
                self.public_inputs.validation_requests.nullifier_key_validation_requests.push(request.to_context(source.storage_contract_address));
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
                    (nullifier_counter == 0) | (nullifier_counter > note_hash.counter), "invalid nullifier counter"
                );

                // TODO: Silo values in the tail circuit.
                note_hash.value = silo_note_hash(source.storage_contract_address, note_hash.value);
                self.public_inputs.end.new_note_hashes.push(note_hash.to_context(nullifier_counter));
            }
        }
    }

    fn propagate_nullifiers(&mut self, source: DataSource) {
        let nullifiers = source.private_call_public_inputs.new_nullifiers;
        for i in 0..nullifiers.len() {
            let nullifier = nullifiers[i];
            if nullifier.value != 0 {
                let siloed_note_hash = if nullifier.note_hash == 0 {
                    0
                } else {
                    silo_note_hash(source.storage_contract_address, nullifier.note_hash)
                };
                self.public_inputs.end.new_nullifiers.push(
                    Nullifier {
                    value: silo_nullifier(source.storage_contract_address, nullifier.value),
                    counter: nullifier.counter,
                    note_hash: siloed_note_hash
                }
                );
            }
        }
    }

    fn propagate_l2_to_l1_messages(&mut self, source: DataSource) {
        let l2_to_l1_msgs = source.private_call_public_inputs.new_l2_to_l1_msgs;
        for i in 0..l2_to_l1_msgs.len() {
            let msg = l2_to_l1_msgs[i];
            if !is_empty(msg) {
                let hash = compute_l2_to_l1_hash(
                    source.storage_contract_address,
                    source.private_call_public_inputs.tx_context.version,
                    source.private_call_public_inputs.tx_context.chain_id,
                    msg
                );
                self.public_inputs.end.new_l2_to_l1_msgs.push(hash);
            }
        }
    }

    fn propagate_logs(&mut self, source: DataSource) {
        self.public_inputs.end.encrypted_logs_hashes.extend_from_bounded_vec(array_to_bounded_vec(source.private_call_public_inputs.encrypted_logs_hashes));
        self.public_inputs.end.unencrypted_logs_hashes.extend_from_bounded_vec(array_to_bounded_vec(source.private_call_public_inputs.unencrypted_logs_hashes));

        // Add log preimages lengths from current iteration to accumulated lengths.
        self.public_inputs.end.encrypted_log_preimages_length += source.private_call_public_inputs.encrypted_log_preimages_length;
        self.public_inputs.end.unencrypted_log_preimages_length += source.private_call_public_inputs.unencrypted_log_preimages_length;
    }

    fn propagate_private_call_requests(&mut self, source: DataSource) {
        let call_requests = source.private_call_requests;
        for i in 0..call_requests.len() {
            let call_request = call_requests[i];
            if !is_empty(call_request) {
                self.public_inputs.end.private_call_stack.push(call_request);
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
}
