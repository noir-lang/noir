use crate::kernel_circuit_public_inputs_composer::KernelCircuitPublicInputsComposer;
use dep::reset_kernel_lib::{NullifierReadRequestHints, PrivateValidationRequestProcessor};
use dep::types::{
    abis::{
    kernel_data::PrivateKernelData, kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey, utils::arrays::array_length
};

struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    sorted_new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    read_commitment_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    sorted_new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    nullifier_read_request_hints: NullifierReadRequestHints,
    nullifier_commitment_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    master_nullifier_secret_keys: [GrumpkinPrivateKey; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
}

impl PrivateKernelTailCircuitPrivateInputs {
    pub fn native_private_kernel_circuit_tail(self) -> KernelCircuitPublicInputs {
        let previous_public_inputs = self.previous_kernel.public_inputs;
        assert_eq(
            array_length(previous_public_inputs.end.public_call_stack), 0, "Public call stack must be empty when executing the tail circuit"
        );

        let nullifier_tree_root = previous_public_inputs.constants.historical_header.state.partial.nullifier_tree.root;
        let request_processor = PrivateValidationRequestProcessor {
            validation_requests: previous_public_inputs.validation_requests,
            note_hash_read_request_hints: self.read_commitment_hints,
            pending_note_hashes: previous_public_inputs.end.new_note_hashes,
            nullifier_read_request_hints: self.nullifier_read_request_hints,
            pending_nullifiers: previous_public_inputs.end.new_nullifiers,
            master_nullifier_secret_keys: self.master_nullifier_secret_keys,
            nullifier_tree_root
        };
        request_processor.validate();

        let mut composer = KernelCircuitPublicInputsComposer::new(
            self.previous_kernel,
            self.sorted_new_note_hashes,
            self.sorted_new_note_hashes_indexes,
            self.sorted_new_nullifiers,
            self.sorted_new_nullifiers_indexes,
            self.nullifier_commitment_hints
        );
        composer.compose().finish()
    }
}

mod tests {
    use dep::std::unsafe;
    use crate::private_kernel_tail::PrivateKernelTailCircuitPrivateInputs;
    use dep::reset_kernel_lib::{
        tests::nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder,
        read_request_reset::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::constants::{
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::KernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered}
    },
        hash::compute_unique_siloed_note_hashes,
        tests::{fixture_builder::FixtureBuilder, sort::sort_get_sorted_hints},
        utils::{arrays::{array_eq, array_length}}, traits::{Empty, is_empty, is_empty_array}
    };

    struct PrivateKernelTailInputsBuilder {
        previous_kernel: FixtureBuilder,
        read_commitment_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
        nullifier_commitment_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            previous_kernel.append_new_nullifiers(1);

            PrivateKernelTailInputsBuilder {
                previous_kernel,
                read_commitment_hints: [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
                nullifier_commitment_hints: [0; MAX_NEW_NULLIFIERS_PER_TX],
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX)
            }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_unique_siloed_note_hashes<N>(
            self,
            note_hashes: [SideEffect; N]
        ) -> [SideEffect; N] {
            let first_nullifier = self.previous_kernel.new_nullifiers.get_unchecked(0);
            compute_unique_siloed_note_hashes(first_nullifier.value, note_hashes)
        }

        pub fn add_pending_note_hash_read_request(&mut self, note_hash_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_note_hash(note_hash_index);
            self.read_commitment_hints[read_request_index] = note_hash_index;
        }

        pub fn add_pending_nullifier_read_request(&mut self, nullifier_index_offset_one: u64) {
            let nullifier_index = nullifier_index_offset_one + 1; // + 1 is for the first nullifier
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn nullify_pending_note_hash(&mut self, nullifier_index: u64, note_hash_index: u64) {
            self.previous_kernel.new_nullifiers.storage[nullifier_index].note_hash = self.previous_kernel.new_note_hashes.get(note_hash_index).value;
            self.nullifier_commitment_hints[nullifier_index] = note_hash_index;
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_note_hashes.storage,
                |a: SideEffect, b: SideEffect| a.counter < b.counter
            );
            let sorted_new_note_hashes = sorted.sorted_array;
            let sorted_new_note_hashes_indexes = sorted.sorted_index_hints;

            let mut sorted_read_commitment_hints = [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX];
            for i in 0..sorted_read_commitment_hints.len() {
                sorted_read_commitment_hints[i] = sorted_new_note_hashes_indexes[self.read_commitment_hints[i]];
            }

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_nullifiers.storage,
                |a: SideEffectLinkedToNoteHash, b: SideEffectLinkedToNoteHash| a.counter < b.counter
            );
            let sorted_new_nullifiers = sorted.sorted_array;
            let sorted_new_nullifiers_indexes = sorted.sorted_index_hints;

            let mut sorted_nullifier_commitment_hints = [0; MAX_NEW_NULLIFIERS_PER_TX];
            for i in 0..sorted_nullifier_commitment_hints.len() {
                sorted_nullifier_commitment_hints[i] = sorted_new_nullifiers_indexes[self.nullifier_commitment_hints[i]];
            }

            let kernel = PrivateKernelTailCircuitPrivateInputs {
                previous_kernel: self.previous_kernel.to_private_kernel_data(),
                sorted_new_note_hashes,
                sorted_new_note_hashes_indexes,
                read_commitment_hints: sorted_read_commitment_hints,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                nullifier_commitment_hints: sorted_nullifier_commitment_hints,
                master_nullifier_secret_keys: unsafe::zeroed()
            };
            kernel.native_private_kernel_circuit_tail()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    unconstrained fn native_matching_one_read_request_to_commitment_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_note_hashes(1);
        builder.add_pending_note_hash_read_request(0);

        let unique_siloed_note_hashes = builder.compute_unique_siloed_note_hashes(builder.previous_kernel.new_note_hashes.storage);

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_note_hashes) == 1);
        assert(public_inputs.end.new_note_hashes[0].eq(unique_siloed_note_hashes[0]));
    }

    #[test(should_fail_with="Hinted note hash does not match read request")]
    unconstrained fn native_read_request_unknown_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(1);
        builder.add_pending_note_hash_read_request(0);
        // Tweak the read request so that it does not match the hash at index 0;
        let read_request = builder.previous_kernel.note_hash_read_requests.pop();
        builder.previous_kernel.note_hash_read_requests.push(SideEffect { value: read_request.value + 1, counter: 0 });
        builder.failed();
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.rollup_validation_requests.max_block_number.unwrap(), 13);
    }

    #[test]
    unconstrained fn one_pending_nullifier_read_request() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);

        builder.succeeded();
    }

    #[test]
    unconstrained fn two_pending_nullifier_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        builder.add_pending_nullifier_read_request(0);

        builder.succeeded();
    }

    #[test(should_fail_with="Hinted value does not match read request")]
    unconstrained fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        let mut hint = builder.nullifier_read_request_hints_builder.pending_read_hints.pop();
        assert(hint.pending_value_index == 2);
        hint.pending_value_index = 1;
        builder.nullifier_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than counter of the value being read")]
    unconstrained fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        let nullifier_being_read = builder.previous_kernel.new_nullifiers.storage[2];
        let mut read_request = builder.previous_kernel.nullifier_read_requests.pop();
        read_request.counter = nullifier_being_read.counter - 1;
        builder.previous_kernel.nullifier_read_requests.push(read_request);

        builder.failed();
    }

    #[test]
    unconstrained fn native_squash_one_of_one_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(1);
        builder.previous_kernel.append_new_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        let new_nullifiers = builder.previous_kernel.new_nullifiers.storage;
        let public_inputs = builder.execute();
        assert(is_empty_array(public_inputs.end.new_note_hashes));

        // The nullifier at index 1 is chopped.
        assert(
            array_eq(
                public_inputs.end.new_nullifiers,
                [new_nullifiers[0], new_nullifiers[2]]
            )
        );
    }

    #[test]
    unconstrained fn native_squash_one_of_two_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        let new_note_hashes = builder.previous_kernel.new_note_hashes.storage;
        // The 0th hash will be chopped.
        let unique_siloed_note_hashes = builder.compute_unique_siloed_note_hashes([new_note_hashes[1]]);
        let new_nullifiers = builder.previous_kernel.new_nullifiers.storage;
        let public_inputs = builder.execute();
        assert(
            array_eq(
                public_inputs.end.new_note_hashes,
                [unique_siloed_note_hashes[0]]
            )
        );
        // The nullifier at index 1 is chopped.
        assert(
            array_eq(
                public_inputs.end.new_nullifiers,
                [new_nullifiers[0], new_nullifiers[2]]
            )
        );
    }

    #[test]
    unconstrained fn native_squash_two_of_two_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(2, 0);
        let new_nullifiers = builder.previous_kernel.new_nullifiers.storage;
        let public_inputs = builder.execute();

        // Only the first nullifier is left after squashing.
        assert(is_empty_array(public_inputs.end.new_note_hashes));
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[0]]));
    }

    #[test]
    unconstrained fn ordering_of_commitments_and_nullifiers() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_note_hashes(10);
        builder.previous_kernel.append_new_nullifiers(10);

        let sorted_note_hashes = builder.previous_kernel.new_note_hashes.storage;
        let sorted_nullifiers = builder.previous_kernel.new_nullifiers.storage;

        let mut reversed_note_hashes = [SideEffect::empty(); 10];
        let mut reversed_nullifiers = [SideEffectLinkedToNoteHash::empty(); 10];

        for i in 0..10 {
            reversed_note_hashes[9 - i] = builder.previous_kernel.new_note_hashes.pop();
            reversed_nullifiers[9 - i] = builder.previous_kernel.new_nullifiers.pop();
        }

        builder.previous_kernel.new_note_hashes.extend_from_array(reversed_note_hashes);
        builder.previous_kernel.new_nullifiers.extend_from_array(reversed_nullifiers);

        let sorted_unique_note_hashes = compute_unique_siloed_note_hashes(
            // tx nullifier is part of non revertible accumulated data
            builder.previous_kernel.new_nullifiers.get_unchecked(0).value,
            sorted_note_hashes
        );

        let public_inputs = builder.execute();

        for i in 0..10 {
            assert(public_inputs.end.new_note_hashes[i].eq(sorted_unique_note_hashes[i]));
            assert(public_inputs.end.new_nullifiers[i].eq(sorted_nullifiers[i]));
        }
    }

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_0() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_note_hashes), 2);
        assert_eq(array_length(public_inputs.end.new_nullifiers), 3);
    }
    // same as previous test, but this time there are 0 commitments!
    // (Do we really need this test?)

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_1() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_nullifiers(2);
        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_note_hashes) == 0);
        assert(array_length(public_inputs.end.new_nullifiers) == 3);
    }

    #[test(should_fail)]
    unconstrained fn invalid_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(1);
        builder.previous_kernel.append_new_nullifiers(1);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        // Change the hint to be out of bounds.
        builder.nullifier_commitment_hints[1] = MAX_NEW_NOTE_HASHES_PER_TX;
        builder.failed();
    }

    #[test(should_fail_with="Hinted hash does not match")]
    unconstrained fn wrong_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(2, 0);
        // Tweak the hint to be for the hash at index 1.
        builder.nullifier_commitment_hints[2] = 1;
        builder.failed();
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.push_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Public call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_public_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.push_public_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    unconstrained fn zero_0th_nullifier_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.new_nullifiers = BoundedVec::new();
        builder.failed();
    }
}
