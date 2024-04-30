use crate::common;
use dep::std::{cmp::Eq, option::Option};
use dep::reset_kernel_lib::{
    NullifierReadRequestHints, PrivateValidationRequestProcessor,
    verify_squashed_transient_note_hashes_and_nullifiers
};
use dep::types::{
    abis::{
    kernel_data::PrivateKernelData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder, PublicKernelCircuitPublicInputs},
    note_hash::NoteHashContext, nullifier::Nullifier, side_effect::{SideEffect, Ordered}, gas::Gas
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX, DA_BYTES_PER_FIELD, FIXED_DA_GAS,
    DA_GAS_PER_BYTE
},
    grumpkin_private_key::GrumpkinPrivateKey,
    hash::{compute_note_hash_nonce, compute_unique_siloed_note_hash},
    utils::arrays::{array_length, array_to_bounded_vec, assert_sorted_array}, traits::{Empty, is_empty}
};

fn asc_sort_by_counters<T>(a: T, b: T) -> bool where T: Ordered {
    a.counter() < b.counter()
}

// Builds:
// .finish -> KernelCircuitPublicInputs (from PrivateKernelTailCircuitPrivateInputs)
// .finish_to_public -> PublicKernelCircuitPublicInputs (from PrivateKernelTailToPublicCircuitPrivateInputs)
struct KernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
    previous_kernel: PrivateKernelData,
    // Final data
    note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
    nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
    // Hints
    transient_nullifier_indexes_for_note_hashes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    transient_note_hash_indexes_for_nullifiers: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_encrypted_log_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes_indexes: [u64; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes_indexes: [u64; MAX_UNENCRYPTED_LOGS_PER_TX],
}

impl KernelCircuitPublicInputsComposer {
    pub fn new(
        previous_kernel: PrivateKernelData,
        note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
        nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
        transient_nullifier_indexes_for_note_hashes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
        transient_note_hash_indexes_for_nullifiers: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_encrypted_log_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_TX],
        sorted_encrypted_log_hashes_indexes: [u64; MAX_ENCRYPTED_LOGS_PER_TX],
        sorted_unencrypted_log_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_TX],
        sorted_unencrypted_log_hashes_indexes: [u64; MAX_UNENCRYPTED_LOGS_PER_TX]
    ) -> Self {
        let public_inputs = PrivateKernelCircuitPublicInputsBuilder::empty();

        KernelCircuitPublicInputsComposer {
            public_inputs,
            previous_kernel,
            note_hashes,
            nullifiers,
            transient_nullifier_indexes_for_note_hashes,
            transient_note_hash_indexes_for_nullifiers,
            sorted_note_hashes,
            sorted_note_hashes_indexes,
            sorted_nullifiers,
            sorted_nullifiers_indexes,
            sorted_encrypted_log_hashes,
            sorted_encrypted_log_hashes_indexes,
            sorted_unencrypted_log_hashes,
            sorted_unencrypted_log_hashes_indexes
        }
    }

    pub fn compose(&mut self) -> Self {
        assert_eq(
            array_length(self.previous_kernel.public_inputs.end.private_call_stack), 0, "Private call stack must be empty when executing the tail circuit"
        );

        self.propagate_rollup_validation_requests();

        self.propagate_constant_data();

        self.propagate_sorted_arrays();

        // TODO: Should be done in a reset circuit.
        self.squash_transient_data();

        self.silo_values();

        *self
    }

    pub fn compose_public(&mut self) -> Self {
        let _ = self.compose();

        self.propagate_sorted_public_call_requests();

        *self
    }

    pub fn finish(self) -> KernelCircuitPublicInputs {
        let teardown_gas = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
        self.public_inputs.finish_tail(teardown_gas)
    }

    pub fn finish_to_public(self) -> PublicKernelCircuitPublicInputs {
        let min_revertible_side_effect_counter = self.previous_kernel.public_inputs.min_revertible_side_effect_counter;
        let teardown_gas = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
        self.public_inputs.finish_to_public(teardown_gas, min_revertible_side_effect_counter)
    }

    fn silo_values(&mut self) {
        self.silo_note_hashes();
        // TODO: Move siloing from init/inner circuits to here.
    }

    fn silo_note_hashes(&mut self) {
        let first_nullifier = self.public_inputs.end.new_nullifiers.get_unchecked(0);
        assert(first_nullifier.value != 0, "The 0th nullifier in the accumulated nullifier array is zero");

        let note_hashes = self.public_inputs.end.new_note_hashes.storage;
        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            let note_hash = note_hashes[i];
            if note_hash.value != 0 {
                let nonce = compute_note_hash_nonce(first_nullifier.value, i);
                let unique_note_hash = compute_unique_siloed_note_hash(nonce, note_hash.value);
                self.public_inputs.end.new_note_hashes.storage[i].value = unique_note_hash;
            }
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
        self.public_inputs.end.encrypted_log_preimages_length = accumulated_data.encrypted_log_preimages_length;
        self.public_inputs.end.unencrypted_log_preimages_length = accumulated_data.unencrypted_log_preimages_length;
    }

    fn propagate_sorted_public_call_requests(&mut self) {
        let accumulated_data = self.previous_kernel.public_inputs.end;
        self.public_inputs.end.public_call_stack = array_to_bounded_vec(accumulated_data.public_call_stack);
    }

    fn squash_transient_data(&mut self) {
        verify_squashed_transient_note_hashes_and_nullifiers(
            self.public_inputs.end.new_note_hashes.storage,
            self.public_inputs.end.new_nullifiers.storage,
            self.note_hashes,
            self.nullifiers,
            self.transient_nullifier_indexes_for_note_hashes,
            self.transient_note_hash_indexes_for_nullifiers
        );

        // Currently all the transient note hashes and nullifiers must be cleared in the tail circuits.
        // Check that the propagated note hashes don't link to a nullifier, and vice versa.
        for i in 0..self.note_hashes.len() {
            assert(self.note_hashes[i].nullifier_counter == 0, "Unresolved transient note hash");
        }
        for i in 0..self.nullifiers.len() {
            assert(self.nullifiers[i].note_hash == 0, "Unresolved transient nullifier");
        }

        self.public_inputs.end.new_note_hashes = array_to_bounded_vec(self.note_hashes);
        self.public_inputs.end.new_nullifiers = array_to_bounded_vec(self.nullifiers);
    }
}
