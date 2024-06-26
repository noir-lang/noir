use dep::types::{
    abis::{
    private_kernel_data::PrivateKernelData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder, PublicKernelCircuitPublicInputs},
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier, side_effect::Ordered,
    log_hash::{NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}, gas::Gas, call_request::CallRequest
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX,
    MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX
},
    hash::{
    silo_encrypted_log_hash, silo_l2_to_l1_message, silo_note_hash, silo_nullifier,
    silo_unencrypted_log_hash
},
    utils::arrays::{array_to_bounded_vec, assert_sorted_array}
};

fn asc_sort_by_counters<T>(a: T, b: T) -> bool where T: Ordered {
    a.counter() < b.counter()
}

fn desc_sort_by_counters<T>(a: T, b: T) -> bool where T: Ordered {
    a.counter() > b.counter()
}

// Builds:
// .finish -> KernelCircuitPublicInputs (from PrivateKernelTailCircuitPrivateInputs)
// .finish_to_public -> PublicKernelCircuitPublicInputs (from PrivateKernelTailToPublicCircuitPrivateInputs)
struct KernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
    previous_kernel: PrivateKernelData,
    // Hints
    sorted_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_note_hashes_indexes: [u32; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_nullifiers_indexes: [u32; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hashes_indexes: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes_indexes: [u32; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes_indexes: [u32; MAX_UNENCRYPTED_LOGS_PER_TX],
}

impl KernelCircuitPublicInputsComposer {
    pub fn new(
        previous_kernel: PrivateKernelData,
        sorted_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_note_hashes_indexes: [u32; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_nullifiers_indexes: [u32; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
        sorted_note_encrypted_log_hashes_indexes: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
        sorted_encrypted_log_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
        sorted_encrypted_log_hashes_indexes: [u32; MAX_ENCRYPTED_LOGS_PER_TX],
        sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
        sorted_unencrypted_log_hashes_indexes: [u32; MAX_UNENCRYPTED_LOGS_PER_TX]
    ) -> Self {
        let public_inputs = PrivateKernelCircuitPublicInputsBuilder::empty();

        KernelCircuitPublicInputsComposer {
            public_inputs,
            previous_kernel,
            sorted_note_hashes,
            sorted_note_hashes_indexes,
            sorted_nullifiers,
            sorted_nullifiers_indexes,
            sorted_note_encrypted_log_hashes,
            sorted_note_encrypted_log_hashes_indexes,
            sorted_encrypted_log_hashes,
            sorted_encrypted_log_hashes_indexes,
            sorted_unencrypted_log_hashes,
            sorted_unencrypted_log_hashes_indexes
        }
    }

    pub fn compose(&mut self) -> Self {
        self.propagate_rollup_validation_requests();

        self.propagate_constant_data();

        self.propagate_sorted_arrays();

        self.propagate_fee_payer();

        self.silo_values();

        *self
    }

    pub fn compose_public(
        &mut self,
        sorted_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
        sorted_call_requests_indexes: [u32; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]
    ) -> Self {
        let _ = self.compose();

        self.propagate_sorted_public_call_requests(sorted_call_requests, sorted_call_requests_indexes);
        self.propagate_public_teardown_call_request();

        *self
    }

    pub fn finish(self) -> KernelCircuitPublicInputs {
        let teardown_gas = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
        let inputs = self.public_inputs.finish_tail(teardown_gas);
        let limits = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.gas_limits;
        assert(inputs.end.gas_used.within(limits), "The gas used exceeds the gas limits");
        inputs
    }

    pub fn finish_to_public(self) -> PublicKernelCircuitPublicInputs {
        let min_revertible_side_effect_counter = self.previous_kernel.public_inputs.min_revertible_side_effect_counter;
        let teardown_gas = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
        let inputs = self.public_inputs.finish_to_public(teardown_gas, min_revertible_side_effect_counter);
        let limits = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.gas_limits;
        let total_gas_used = inputs.end.gas_used + inputs.end_non_revertible.gas_used;
        assert(total_gas_used.within(limits), "The gas used exceeds the gas limits");
        inputs
    }

    fn silo_values(&mut self) {
        self.silo_note_hashes();
        self.silo_nullifiers();
        self.silo_l2_to_l1_messages();
        self.silo_encrypted_logs();
        self.silo_unencrypted_logs();
    }

    fn silo_note_hashes(&mut self) {
        // First nullifier is tx hash.
        let tx_hash = self.public_inputs.end.new_nullifiers.get_unchecked(0).value();

        let note_hashes = self.public_inputs.end.new_note_hashes.storage;
        for i in 0..note_hashes.len() {
            self.public_inputs.end.new_note_hashes.storage[i].note_hash.value = silo_note_hash(
                note_hashes[i],
                tx_hash,
                i
            );
        }
    }

    fn silo_nullifiers(&mut self) {
        let nullifiers = self.public_inputs.end.new_nullifiers.storage;
        for i in 1..nullifiers.len() { // i starts from 1 to skip the first nullifier.
            self.public_inputs.end.new_nullifiers.storage[i].nullifier.value = silo_nullifier(nullifiers[i]);
        }
    }

    fn silo_l2_to_l1_messages(&mut self) {
        let l2_to_l1_msgs = self.public_inputs.end.new_l2_to_l1_msgs.storage;
        let tx_context = self.previous_kernel.public_inputs.constants.tx_context;
        for i in 0..l2_to_l1_msgs.len() {
            self.public_inputs.end.new_l2_to_l1_msgs.storage[i].message.content = silo_l2_to_l1_message(
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

    fn propagate_rollup_validation_requests(&mut self) {
        self.public_inputs.validation_requests.max_block_number = self.previous_kernel.public_inputs.validation_requests.for_rollup.max_block_number;
    }

    fn propagate_constant_data(&mut self) {
        self.public_inputs.constants = self.previous_kernel.public_inputs.constants;
    }

    fn propagate_sorted_arrays(&mut self) {
        let accumulated_data = self.previous_kernel.public_inputs.end;

        assert_sorted_array(
            accumulated_data.new_note_hashes,
            self.sorted_note_hashes,
            self.sorted_note_hashes_indexes,
            asc_sort_by_counters
        );
        self.public_inputs.end.new_note_hashes = array_to_bounded_vec(self.sorted_note_hashes);

        assert_sorted_array(
            accumulated_data.new_nullifiers,
            self.sorted_nullifiers,
            self.sorted_nullifiers_indexes,
            asc_sort_by_counters
        );
        self.public_inputs.end.new_nullifiers = array_to_bounded_vec(self.sorted_nullifiers);

        assert_sorted_array(
            accumulated_data.note_encrypted_logs_hashes,
            self.sorted_note_encrypted_log_hashes,
            self.sorted_note_encrypted_log_hashes_indexes,
            asc_sort_by_counters
        );
        self.public_inputs.end.note_encrypted_logs_hashes = array_to_bounded_vec(self.sorted_note_encrypted_log_hashes);

        assert_sorted_array(
            accumulated_data.encrypted_logs_hashes,
            self.sorted_encrypted_log_hashes,
            self.sorted_encrypted_log_hashes_indexes,
            asc_sort_by_counters
        );
        self.public_inputs.end.encrypted_logs_hashes = array_to_bounded_vec(self.sorted_encrypted_log_hashes);

        assert_sorted_array(
            accumulated_data.unencrypted_logs_hashes,
            self.sorted_unencrypted_log_hashes,
            self.sorted_unencrypted_log_hashes_indexes,
            asc_sort_by_counters
        );
        self.public_inputs.end.unencrypted_logs_hashes = array_to_bounded_vec(self.sorted_unencrypted_log_hashes);
        // TODO: Sort all the side effects below.
        self.public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(accumulated_data.new_l2_to_l1_msgs);
    }

    fn propagate_sorted_public_call_requests(
        &mut self,
        sorted_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
        sorted_call_requests_indexes: [u32; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]
    ) {
        let accumulated_data = self.previous_kernel.public_inputs.end;
        assert_sorted_array(
            accumulated_data.public_call_stack,
            sorted_call_requests,
            sorted_call_requests_indexes,
            desc_sort_by_counters
        );
        self.public_inputs.end.public_call_stack = array_to_bounded_vec(sorted_call_requests);
    }

    fn propagate_public_teardown_call_request(&mut self) {
        self.public_inputs.public_teardown_call_request = self.previous_kernel.public_inputs.public_teardown_call_request;
    }

    fn propagate_fee_payer(&mut self) {
        self.public_inputs.fee_payer = self.previous_kernel.public_inputs.fee_payer;
    }
}
