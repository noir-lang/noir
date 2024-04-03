use crate::common;
use dep::std::{cmp::Eq, option::Option, unsafe};
use dep::reset_kernel_lib::{NullifierReadRequestHints, PrivateValidationRequestProcessor};
use dep::types::{
    abis::{
    kernel_data::PrivateKernelData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsBuilder, PublicKernelCircuitPublicInputs},
    side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey,
    hash::{compute_note_hash_nonce, compute_unique_siloed_note_hash},
    utils::arrays::{array_length, array_to_bounded_vec, assert_sorted_array}, traits::{Empty, is_empty}
};

fn asc_sort_by_counters<T>(a: T, b: T) -> bool where T: Ordered {
    a.counter() < b.counter()
}

struct KernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
    previous_kernel: PrivateKernelData,
    sorted_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    transient_note_hash_index_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX],
}

impl KernelCircuitPublicInputsComposer {
    pub fn new(
        previous_kernel: PrivateKernelData,
        sorted_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
        sorted_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
        sorted_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        transient_note_hash_index_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX]
    ) -> Self {
        let public_inputs: PrivateKernelCircuitPublicInputsBuilder = unsafe::zeroed();

        KernelCircuitPublicInputsComposer {
            public_inputs,
            previous_kernel,
            sorted_note_hashes,
            sorted_note_hashes_indexes,
            sorted_nullifiers,
            sorted_nullifiers_indexes,
            transient_note_hash_index_hints
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
        self.squash_transient_note_hashes_and_nullifiers();

        self.silo_values();

        *self
    }

    pub fn compose_public(&mut self) -> Self {
        let _ = self.compose();

        self.propagate_sorted_public_call_requests();

        *self
    }

    pub fn finish(self) -> KernelCircuitPublicInputs {
        self.public_inputs.finish_tail()
    }

    pub fn finish_to_public(self) -> PublicKernelCircuitPublicInputs {
        let min_revertible_side_effect_counter = self.previous_kernel.public_inputs.min_revertible_side_effect_counter;
        self.public_inputs.finish_to_public(min_revertible_side_effect_counter)
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

        // TODO: Sort all the side effects below.
        self.public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(accumulated_data.new_l2_to_l1_msgs);
        self.public_inputs.end.encrypted_logs_hash = accumulated_data.encrypted_logs_hash;
        self.public_inputs.end.unencrypted_logs_hash = accumulated_data.unencrypted_logs_hash;
        self.public_inputs.end.encrypted_log_preimages_length = accumulated_data.encrypted_log_preimages_length;
        self.public_inputs.end.unencrypted_log_preimages_length = accumulated_data.unencrypted_log_preimages_length;
    }

    fn propagate_sorted_public_call_requests(&mut self) {
        let accumulated_data = self.previous_kernel.public_inputs.end;
        self.public_inputs.end.public_call_stack = array_to_bounded_vec(accumulated_data.public_call_stack);
    }

    fn squash_transient_note_hashes_and_nullifiers(&mut self) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // Match nullifiers/nullified_commitments to commitments from the previous call(s)
        let mut new_note_hashes = self.public_inputs.end.new_note_hashes.storage;
        let mut new_nullifiers = self.public_inputs.end.new_nullifiers.storage;

        // Should start from 1 to skip the first nullifier?
        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = new_nullifiers[n_idx];
            // TODO - should not be able to squash the first nullifier.
            let nullified_note_hash = nullifier.note_hash;
            let hint_pos = self.transient_note_hash_index_hints[n_idx];

            // Nullified_commitment of value `0` implies non-transient (persistable)
            // nullifier in which case no attempt will be made to match it to a hash.
            // Non-empty nullified_note_hash implies transient nullifier which MUST be matched to a hash below!
            // 0-valued nullified_note_hash is empty and will be ignored
            if nullified_note_hash != 0 {
                assert(
                    hint_pos < MAX_NEW_NOTE_HASHES_PER_TX, "New nullifier is transient but hint is invalid"
                );
                let hash = new_note_hashes[hint_pos];
                assert_eq(nullified_note_hash, hash.value, "Hinted hash does not match");
                assert(
                    nullifier.counter > hash.counter, "Nullifier counter must be greater than hash counter"
                );
                // match found!
                // squash both the nullifier and the hash
                // (set to 0 here and then rearrange array after loop)
                new_note_hashes[hint_pos] = SideEffect::empty();
                new_nullifiers[n_idx] = SideEffectLinkedToNoteHash::empty();
            }
            // non-transient (persistable) nullifiers are just kept in new_nullifiers array and forwarded
            // to public inputs (used later by base rollup circuit)
        }

        // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries

        let mut new_note_hashes_vec = BoundedVec::new();

        for c_idx in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            if new_note_hashes[c_idx].value != 0 {
                new_note_hashes_vec.push(new_note_hashes[c_idx]);
            }
        }

        self.public_inputs.end.new_note_hashes = new_note_hashes_vec;

        let mut new_nullifiers_vec = BoundedVec::new();

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if new_nullifiers[n_idx].value != 0 {
                new_nullifiers_vec.push(new_nullifiers[n_idx]);
            }
        }

        self.public_inputs.end.new_nullifiers = new_nullifiers_vec;
    }
}
