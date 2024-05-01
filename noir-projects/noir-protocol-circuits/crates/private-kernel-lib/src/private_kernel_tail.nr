use crate::kernel_circuit_public_inputs_composer::KernelCircuitPublicInputsComposer;
use dep::reset_kernel_lib::{NullifierReadRequestHints, PrivateValidationRequestProcessor};
use dep::types::{
    abis::{
    kernel_data::PrivateKernelData, kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    note_hash::NoteHashContext, nullifier::Nullifier, side_effect::SideEffect
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey, utils::arrays::array_length
};

// Can just be KernelCircuitPublicInputs.
struct PrivateKernelTailOutputs {
    note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
    nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
}

struct PrivateKernelTailHints {
    transient_nullifier_indexes_for_note_hashes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    transient_note_hash_indexes_for_nullifiers: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    note_hash_read_request_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    nullifier_read_request_hints: NullifierReadRequestHints,
    master_nullifier_secret_keys: [GrumpkinPrivateKey; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
    sorted_new_note_hashes: [NoteHashContext; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_encrypted_log_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes_indexes: [u64; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes_indexes: [u64; MAX_UNENCRYPTED_LOGS_PER_TX],
}

struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    outputs: PrivateKernelTailOutputs,
    hints: PrivateKernelTailHints,
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
            note_hash_read_request_hints: self.hints.note_hash_read_request_hints,
            pending_note_hashes: previous_public_inputs.end.new_note_hashes,
            nullifier_read_request_hints: self.hints.nullifier_read_request_hints,
            pending_nullifiers: previous_public_inputs.end.new_nullifiers,
            master_nullifier_secret_keys: self.hints.master_nullifier_secret_keys,
            nullifier_tree_root
        };
        request_processor.validate();

        let mut composer = KernelCircuitPublicInputsComposer::new(
            self.previous_kernel,
            self.outputs.note_hashes,
            self.outputs.nullifiers,
            self.hints.transient_nullifier_indexes_for_note_hashes,
            self.hints.transient_note_hash_indexes_for_nullifiers,
            self.hints.sorted_new_note_hashes,
            self.hints.sorted_new_note_hashes_indexes,
            self.hints.sorted_new_nullifiers,
            self.hints.sorted_new_nullifiers_indexes,
            self.hints.sorted_encrypted_log_hashes,
            self.hints.sorted_encrypted_log_hashes_indexes,
            self.hints.sorted_unencrypted_log_hashes,
            self.hints.sorted_unencrypted_log_hashes_indexes
        );
        composer.compose().finish()
    }
}

mod tests {
    use crate::private_kernel_tail::{PrivateKernelTailCircuitPrivateInputs, PrivateKernelTailHints, PrivateKernelTailOutputs};
    use dep::reset_kernel_lib::{
        tests::{
        nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder,
        squash_transient_data::{squash_transient_note_hashes, squash_transient_nullifiers}
    },
        reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::constants::{
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::KernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        note_hash::NoteHashContext, nullifier::Nullifier, side_effect::{SideEffect, Ordered}, gas::Gas
    },
        grumpkin_private_key::GrumpkinPrivateKey,
        hash::{compute_note_hash_nonce, compute_unique_siloed_note_hash, accumulate_sha256},
        tests::{fixture_builder::FixtureBuilder, sort::sort_get_sorted_hints},
        utils::{arrays::{array_eq, array_length}}, traits::{Empty, is_empty, is_empty_array}
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailInputs and PrivateKernelTailToPublicInputs.
    struct PrivateKernelTailInputsBuilder {
        previous_kernel: FixtureBuilder,
        note_hash_read_request_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
        transient_nullifier_indexes_for_note_hashes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
        transient_note_hash_indexes_for_nullifiers: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);

            previous_kernel.append_new_nullifiers(1);

            PrivateKernelTailInputsBuilder {
                previous_kernel,
                note_hash_read_request_hints: [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
                transient_nullifier_indexes_for_note_hashes: [MAX_NEW_NULLIFIERS_PER_TX; MAX_NEW_NOTE_HASHES_PER_TX],
                transient_note_hash_indexes_for_nullifiers: [MAX_NEW_NOTE_HASHES_PER_TX; MAX_NEW_NULLIFIERS_PER_TX],
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX)
            }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_unique_siloed_note_hashes<N>(
            self,
            note_hashes: [NoteHashContext; N]
        ) -> [Field; N] {
            let first_nullifier = self.previous_kernel.new_nullifiers.get_unchecked(0);
            let mut unique_siloed_note_hashes = [0; N];
            for i in 0..N {
                if note_hashes[i].value != 0 {
                    let nonce = compute_note_hash_nonce(first_nullifier.value, i);
                    unique_siloed_note_hashes[i] = compute_unique_siloed_note_hash(nonce, note_hashes[i].value);
                }
            }
            unique_siloed_note_hashes
        }

        pub fn add_pending_note_hash_read_request(&mut self, note_hash_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_note_hash(note_hash_index);
            self.note_hash_read_request_hints[read_request_index] = note_hash_index;
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
            self.previous_kernel.new_note_hashes.storage[note_hash_index].nullifier_counter = self.previous_kernel.new_nullifiers.get(nullifier_index).counter;
            self.previous_kernel.new_nullifiers.storage[nullifier_index].note_hash = self.previous_kernel.new_note_hashes.get(note_hash_index).value;
            self.transient_nullifier_indexes_for_note_hashes[note_hash_index] = nullifier_index;
            self.transient_note_hash_indexes_for_nullifiers[nullifier_index] = note_hash_index;
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_note_hashes.storage,
                |a: NoteHashContext, b: NoteHashContext| a.counter < b.counter
            );
            let sorted_new_note_hashes = sorted.sorted_array;
            let sorted_new_note_hashes_indexes = sorted.sorted_index_hints;

            let mut sorted_note_hash_read_request_hints = [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX];
            for i in 0..sorted_note_hash_read_request_hints.len() {
                sorted_note_hash_read_request_hints[i] = sorted_new_note_hashes_indexes[self.note_hash_read_request_hints[i]];
            }

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_nullifiers.storage,
                |a: Nullifier, b: Nullifier| a.counter < b.counter
            );
            let sorted_new_nullifiers = sorted.sorted_array;
            let sorted_new_nullifiers_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.encrypted_logs_hashes.storage,
                |a: SideEffect, b: SideEffect| a.counter < b.counter
            );
            let sorted_encrypted_log_hashes = sorted.sorted_array;
            let sorted_encrypted_log_hashes_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.unencrypted_logs_hashes.storage,
                |a: SideEffect, b: SideEffect| a.counter < b.counter
            );
            let sorted_unencrypted_log_hashes = sorted.sorted_array;
            let sorted_unencrypted_log_hashes_indexes = sorted.sorted_index_hints;

            let mut sorted_transient_nullifier_indexes_for_note_hashes = [MAX_NEW_NULLIFIERS_PER_TX; MAX_NEW_NOTE_HASHES_PER_TX];
            for i in 0..self.transient_nullifier_indexes_for_note_hashes.len() {
                let old_index = self.transient_nullifier_indexes_for_note_hashes[i];
                if old_index != MAX_NEW_NULLIFIERS_PER_TX {
                    let new_note_hash_index = sorted_new_note_hashes_indexes[i];
                    sorted_transient_nullifier_indexes_for_note_hashes[new_note_hash_index] = sorted_new_nullifiers_indexes[old_index];
                }
            }

            let mut sorted_transient_note_hash_indexes_for_nullifiers = [MAX_NEW_NOTE_HASHES_PER_TX; MAX_NEW_NULLIFIERS_PER_TX];
            for i in 0..self.transient_note_hash_indexes_for_nullifiers.len() {
                let old_index = self.transient_note_hash_indexes_for_nullifiers[i];
                if old_index != MAX_NEW_NOTE_HASHES_PER_TX {
                    let new_nullifier_index = sorted_new_nullifiers_indexes[i];
                    sorted_transient_note_hash_indexes_for_nullifiers[new_nullifier_index] = sorted_new_note_hashes_indexes[old_index];
                }
            }

            let outputs = PrivateKernelTailOutputs {
                note_hashes: squash_transient_note_hashes(sorted_new_note_hashes),
                nullifiers: squash_transient_nullifiers(sorted_new_nullifiers)
            };

            let hints = PrivateKernelTailHints {
                transient_nullifier_indexes_for_note_hashes: sorted_transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers: sorted_transient_note_hash_indexes_for_nullifiers,
                note_hash_read_request_hints: sorted_note_hash_read_request_hints,
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                master_nullifier_secret_keys: [GrumpkinPrivateKey::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
                sorted_new_note_hashes,
                sorted_new_note_hashes_indexes,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                sorted_encrypted_log_hashes,
                sorted_encrypted_log_hashes_indexes,
                sorted_unencrypted_log_hashes,
                sorted_unencrypted_log_hashes_indexes
            };

            let kernel = PrivateKernelTailCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data(), outputs, hints };
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
    unconstrained fn execution_succeeded() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let public_inputs = builder.execute();

        assert(is_empty(public_inputs.start_state));
    }

    #[test]
    unconstrained fn native_matching_one_read_request_to_note_hash_works() {
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
    fn logs_are_handled_as_expected() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.previous_kernel.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        let expected_encrypted_logs_hash = accumulate_sha256([0, prev_encrypted_logs_hash]);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);
        let mut expected_unencrypted_logs_hash = accumulate_sha256([0, prev_unencrypted_logs_hash]);
        expected_unencrypted_logs_hash = accumulate_sha256([expected_unencrypted_logs_hash, unencrypted_logs_hash]);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
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
                [new_nullifiers[0].value, new_nullifiers[2].value]
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
                [new_nullifiers[0].value, new_nullifiers[2].value]
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
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[0].value]));
    }

    #[test]
    unconstrained fn squash_unordered_transient_notes_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_note_hashes(3);
        // Shuffle the note hashes so they will have to be re-ordered.
        let tmp = builder.previous_kernel.new_note_hashes.storage[0];
        builder.previous_kernel.new_note_hashes.storage[0] = builder.previous_kernel.new_note_hashes.storage[1];
        builder.previous_kernel.new_note_hashes.storage[1] = builder.previous_kernel.new_note_hashes.storage[2];
        builder.previous_kernel.new_note_hashes.storage[2] = tmp;

        builder.previous_kernel.append_new_nullifiers(3);
        // Shuffle the nullifers so they will have to be re-ordered.
        let tmp = builder.previous_kernel.new_nullifiers.storage[1];
        builder.previous_kernel.new_nullifiers.storage[1] = builder.previous_kernel.new_nullifiers.storage[3];
        builder.previous_kernel.new_nullifiers.storage[3] = builder.previous_kernel.new_nullifiers.storage[2];
        builder.previous_kernel.new_nullifiers.storage[2] = tmp;

        // The nullifier at index 1 is nullifying the note hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the note hash at index 2;
        builder.nullify_pending_note_hash(2, 2);
        // The nullifier at index 3 is nullifying the note hash at index 0;
        builder.nullify_pending_note_hash(3, 0);

        let new_nullifiers = builder.previous_kernel.new_nullifiers.storage;
        let public_inputs = builder.execute();

        // Only the first nullifier is left after squashing.
        assert(is_empty_array(public_inputs.end.new_note_hashes));
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[0].value]));
    }

    #[test]
    unconstrained fn ordering_of_note_hashes_and_nullifiers() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_new_note_hashes(10);
        builder.previous_kernel.append_new_nullifiers(10);

        let sorted_note_hashes = builder.previous_kernel.new_note_hashes.storage;
        let sorted_nullifiers = builder.previous_kernel.new_nullifiers.storage;

        let mut reversed_note_hashes = [NoteHashContext::empty(); 10];
        let mut reversed_nullifiers = [Nullifier::empty(); 10];

        for i in 0..10 {
            reversed_note_hashes[9 - i] = builder.previous_kernel.new_note_hashes.pop();
            reversed_nullifiers[9 - i] = builder.previous_kernel.new_nullifiers.pop();
        }

        builder.previous_kernel.new_note_hashes.extend_from_array(reversed_note_hashes);
        builder.previous_kernel.new_nullifiers.extend_from_array(reversed_nullifiers);

        let sorted_unique_note_hashes = builder.compute_unique_siloed_note_hashes(sorted_note_hashes);

        let public_inputs = builder.execute();

        for i in 0..10 {
            assert(public_inputs.end.new_note_hashes[i].eq(sorted_unique_note_hashes[i]));
            assert(public_inputs.end.new_nullifiers[i].eq(sorted_nullifiers[i].value));
        }
    }

    #[test]
    unconstrained fn native_empty_nullified_note_hash_means_persistent_nullifier_0() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_note_hashes), 2);
        assert_eq(array_length(public_inputs.end.new_nullifiers), 3);
        let expected_gas = Gas::tx_overhead() + Gas::new(DA_GAS_PER_BYTE * DA_BYTES_PER_FIELD * 5, 0);
        assert_eq(public_inputs.end.gas_used, expected_gas);
    }

    #[test(should_fail_with="Hinted note hash does not match")]
    unconstrained fn wrong_transient_nullifier_index_for_note_hash_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(1);
        builder.previous_kernel.append_new_nullifiers(1);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        // Change the hint to be out of bounds.
        builder.transient_nullifier_indexes_for_note_hashes[0] = 0;
        builder.failed();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    unconstrained fn wrong_transient_nullifier_index_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.append_new_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(2, 0);
        // Tweak the hint to be for the hash at index 1.
        builder.transient_note_hash_indexes_for_nullifiers[2] = 1;
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

    #[test]
    unconstrained fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();


        let expected_gas_consumed = Gas::new(300, 300) // teardown gas
            + Gas::tx_overhead() // tx overhead
            + Gas::new(DA_GAS_PER_BYTE * DA_BYTES_PER_FIELD * 1, 0); // tx nullifier
        assert_eq(public_inputs.end.gas_used, expected_gas_consumed);
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    unconstrained fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }
}
