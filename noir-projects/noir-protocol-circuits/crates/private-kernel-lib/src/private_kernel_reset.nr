use crate::components::reset_output_composer::{PrivateKernelResetOutputs, ResetOutputComposer};
use dep::reset_kernel_lib::{
    get_transient_or_propagated_note_hash_indexes_for_logs, KeyValidationHint, NoteHashReadRequestHints,
    NullifierReadRequestHints, PrivateValidationRequestProcessor, verify_squashed_transient_data
};
use dep::types::{
    abis::private_kernel_data::PrivateKernelData,
    constants::{MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX},
    PrivateKernelCircuitPublicInputs
};

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_INIT_INDEX,
    PRIVATE_KERNEL_INNER_INDEX,
];

struct PrivateKernelResetHints<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    transient_nullifier_indexes_for_note_hashes: [u32; MAX_NOTE_HASHES_PER_TX],
    transient_note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX],
    note_hash_read_request_hints: NoteHashReadRequestHints<NH_RR_PENDING, NH_RR_SETTLED>,
    nullifier_read_request_hints: NullifierReadRequestHints<NLL_RR_PENDING, NLL_RR_SETTLED>,
    key_validation_hints: [KeyValidationHint; KEY_VALIDATION_REQUESTS],
}

struct PrivateKernelResetCircuitPrivateInputs<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    previous_kernel: PrivateKernelData,
    hints: PrivateKernelResetHints<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
}

impl<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> PrivateKernelResetCircuitPrivateInputs<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    unconstrained fn generate_output(self) -> PrivateKernelResetOutputs {
        ResetOutputComposer::new(
            self.previous_kernel.public_inputs,
            self.hints.transient_nullifier_indexes_for_note_hashes,
            self.hints.transient_note_hash_indexes_for_nullifiers
        ).finish()
    }

    pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        if !std::runtime::is_unconstrained() {
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        let previous_public_inputs = self.previous_kernel.public_inputs;
        let updated_validation_requests = PrivateValidationRequestProcessor {
            validation_requests: previous_public_inputs.validation_requests,
            note_hash_read_request_hints: self.hints.note_hash_read_request_hints,
            pending_note_hashes: previous_public_inputs.end.note_hashes,
            note_hash_tree_root: previous_public_inputs.constants.historical_header.state.partial.note_hash_tree.root,
            nullifier_read_request_hints: self.hints.nullifier_read_request_hints,
            pending_nullifiers: previous_public_inputs.end.nullifiers,
            nullifier_tree_root: previous_public_inputs.constants.historical_header.state.partial.nullifier_tree.root,
            key_validation_hints: self.hints.key_validation_hints
        }.validate();

        // Validate output.
        if !dep::std::runtime::is_unconstrained() {
            let transient_or_propagated_note_hash_indexes_for_logs = get_transient_or_propagated_note_hash_indexes_for_logs(
                previous_public_inputs.end.note_encrypted_logs_hashes,
                previous_public_inputs.end.note_hashes,
                output.note_hashes
            );
            verify_squashed_transient_data(
                previous_public_inputs.end.note_hashes,
                previous_public_inputs.end.nullifiers,
                previous_public_inputs.end.note_encrypted_logs_hashes,
                output.note_hashes,
                output.nullifiers,
                output.note_encrypted_log_hashes,
                self.hints.transient_nullifier_indexes_for_note_hashes,
                self.hints.transient_note_hash_indexes_for_nullifiers,
                transient_or_propagated_note_hash_indexes_for_logs
            );
        }

        // Generate output.
        let mut updated_output = self.previous_kernel.public_inputs;
        updated_output.validation_requests = updated_validation_requests;
        updated_output.end.note_hashes = output.note_hashes;
        updated_output.end.nullifiers = output.nullifiers;
        updated_output.end.note_encrypted_logs_hashes = output.note_encrypted_log_hashes;

        updated_output
    }
}

mod tests {
    use crate::private_kernel_reset::{PrivateKernelResetCircuitPrivateInputs, PrivateKernelResetHints, ALLOWED_PREVIOUS_CIRCUITS};
    use dep::reset_kernel_lib::{
        tests::{
        note_hash_read_request_hints_builder::NoteHashReadRequestHintsBuilder,
        nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder
    },
        reset::{
        read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus},
        key_validation_hint::KeyValidationHint
    }
    };
    use dep::types::constants::{
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX, GENERATOR_INDEX__OVSK_M,
        PRIVATE_KERNEL_INNER_INDEX, BASE_ROLLUP_INDEX
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        max_block_number::MaxBlockNumber, note_hash::{NoteHash, ScopedNoteHash},
        nullifier::{Nullifier, ScopedNullifier}, log_hash::NoteLogHash, read_request::ScopedReadRequest
    },
        address::AztecAddress, scalar::Scalar, tests::{fixture_builder::FixtureBuilder},
        utils::{arrays::{array_eq, array_length}}, traits::{Empty, is_empty, is_empty_array},
        point::Point
    };

    struct PrivateKernelResetInputsBuilder {
        previous_kernel: FixtureBuilder,
        transient_nullifier_indexes_for_note_hashes: [u32; MAX_NOTE_HASHES_PER_TX],
        transient_note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX],
        note_hash_read_request_hints_builder: NoteHashReadRequestHintsBuilder,
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
    }

    impl PrivateKernelResetInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_INNER_INDEX);
            previous_kernel.append_nullifiers(1);

            PrivateKernelResetInputsBuilder {
                previous_kernel,
                transient_nullifier_indexes_for_note_hashes: [MAX_NULLIFIERS_PER_TX; MAX_NOTE_HASHES_PER_TX],
                transient_note_hash_indexes_for_nullifiers: [MAX_NOTE_HASHES_PER_TX; MAX_NULLIFIERS_PER_TX],
                note_hash_read_request_hints_builder: NoteHashReadRequestHintsBuilder::new(MAX_NOTE_HASH_READ_REQUESTS_PER_TX),
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX)
            }
        }

        pub fn add_pending_note_hash_read_request(&mut self, note_hash_index: u32) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_note_hash(note_hash_index);
            let hint_index = self.note_hash_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: note_hash_index };
            self.note_hash_read_request_hints_builder.pending_read_hints.push(hint);
            self.note_hash_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn add_pending_nullifier_read_request(&mut self, nullifier_index_offset_one: u32) {
            let nullifier_index = nullifier_index_offset_one + 1; // + 1 is for the first nullifier
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn nullify_pending_note_hash(&mut self, nullifier_index: u32, note_hash_index: u32) {
            let note_hash = self.previous_kernel.note_hashes.get(note_hash_index).note_hash;
            self.previous_kernel.nullifiers.storage[nullifier_index].nullifier.note_hash = note_hash.value;
            self.transient_nullifier_indexes_for_note_hashes[note_hash_index] = nullifier_index;
            self.transient_note_hash_indexes_for_nullifiers[nullifier_index] = note_hash_index;
        }

        pub fn execute(&mut self) -> PrivateKernelCircuitPublicInputs {
            let hints = PrivateKernelResetHints {
                transient_nullifier_indexes_for_note_hashes: self.transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers: self.transient_note_hash_indexes_for_nullifiers,
                note_hash_read_request_hints: self.note_hash_read_request_hints_builder.to_hints(),
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                key_validation_hints: [KeyValidationHint::empty(); MAX_KEY_VALIDATION_REQUESTS_PER_TX]
            };

            let kernel = PrivateKernelResetCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data(), hints };
            kernel.execute()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn execution_succeeded() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        let _public_inputs = builder.execute();
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 13);
    }

    #[test]
    fn two_pending_note_hash_read_request() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_note_hashes(3);
        builder.add_pending_note_hash_read_request(1);
        builder.add_pending_note_hash_read_request(0);

        builder.succeeded();
    }

    #[test(should_fail_with="Value of the note hash does not match read request")]
    fn pending_note_hash_read_request_wrong_hint_fails() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_note_hashes(3);
        builder.add_pending_note_hash_read_request(1);
        let mut hint = builder.note_hash_read_request_hints_builder.pending_read_hints.pop();
        hint.pending_value_index = 2;
        builder.note_hash_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test]
    fn one_pending_nullifier_read_request() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);

        builder.succeeded();
    }

    #[test]
    fn two_pending_nullifier_read_requests() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        builder.add_pending_nullifier_read_request(0);

        builder.succeeded();
    }

    #[test(should_fail_with="Value of the nullifier does not match read request")]
    fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        let mut hint = builder.nullifier_read_request_hints_builder.pending_read_hints.pop();
        assert(hint.pending_value_index == 2);
        hint.pending_value_index = 1;
        builder.nullifier_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than the counter of the nullifier")]
    fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_nullifiers(3);
        builder.add_pending_nullifier_read_request(1);
        let nullifier_being_read = builder.previous_kernel.nullifiers.storage[2];
        let mut read_request = builder.previous_kernel.nullifier_read_requests.pop();
        read_request.read_request.counter = nullifier_being_read.counter() - 1;
        builder.previous_kernel.nullifier_read_requests.push(read_request);

        builder.failed();
    }

    #[test]
    fn propagates_unverified_requests() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_note_hashes(3);
        builder.previous_kernel.append_nullifiers(3);

        builder.add_pending_note_hash_read_request(0);
        builder.add_pending_nullifier_read_request(1);

        // Now add some read requests that will be propagated
        let remaining_note_hash_rr_index = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        let note_hash_rr = builder.previous_kernel.note_hash_read_requests.storage[remaining_note_hash_rr_index];

        let remaining_nullifier_rr_index = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        let nullifier_rr = builder.previous_kernel.nullifier_read_requests.storage[remaining_nullifier_rr_index];

        let key_validation_index = builder.previous_kernel.add_request_for_key_validation(Point::new(1, 2, false), 27, GENERATOR_INDEX__OVSK_M);
        let key_validation = builder.previous_kernel.scoped_key_validation_requests_and_generators.storage[key_validation_index];

        // Check that they have been propagated to the next kernel
        let result = builder.execute();

        assert_eq(result.validation_requests.note_hash_read_requests[0], note_hash_rr);
        assert_eq(result.validation_requests.nullifier_read_requests[0], nullifier_rr);
        assert_eq(result.validation_requests.scoped_key_validation_requests_and_generators[0], key_validation);
    }

    #[test]
    fn native_squash_one_of_one_transient_matches_works() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.append_note_hashes_with_logs(1);
        builder.previous_kernel.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let public_inputs = builder.execute();

        assert(is_empty_array(public_inputs.end.note_hashes));

        // The nullifier at index 1 is chopped.
        assert(array_eq(public_inputs.end.nullifiers, [nullifiers[0], nullifiers[2]]));
        assert(is_empty_array(public_inputs.end.note_encrypted_logs_hashes));
    }

    #[test]
    fn native_squash_one_of_two_transient_matches_works() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.append_note_hashes_with_logs(2);
        builder.previous_kernel.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        let note_hashes = builder.previous_kernel.note_hashes.storage;
        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let note_logs = builder.previous_kernel.note_encrypted_logs_hashes.storage;
        let public_inputs = builder.execute();

        // The 0th hash is chopped.
        assert(array_eq(public_inputs.end.note_hashes, [note_hashes[1]]));

        // The nullifier at index 1 is chopped.
        assert(array_eq(public_inputs.end.nullifiers, [nullifiers[0], nullifiers[2]]));

        // The 0th note log is chopped.
        assert(array_eq(public_inputs.end.note_encrypted_logs_hashes, [note_logs[1]]));
    }

    #[test]
    fn native_squash_two_of_two_transient_matches_works() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.append_note_hashes_with_logs(2);
        builder.previous_kernel.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(2, 0);
        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let public_inputs = builder.execute();

        assert(is_empty_array(public_inputs.end.note_hashes));

        // Only the first nullifier is left after squashing.
        assert(array_eq(public_inputs.end.nullifiers, [nullifiers[0]]));
        assert(is_empty_array(public_inputs.end.note_encrypted_logs_hashes));
    }

    #[test]
    fn squash_unordered_transient_notes_works() {
        let mut builder = PrivateKernelResetInputsBuilder::new();

        builder.previous_kernel.append_note_hashes_with_logs(3);
        // Shuffle the note hashes so they will have to be re-ordered.
        let tmp = builder.previous_kernel.note_hashes.storage[0];
        builder.previous_kernel.note_hashes.storage[0] = builder.previous_kernel.note_hashes.storage[1];
        builder.previous_kernel.note_hashes.storage[1] = builder.previous_kernel.note_hashes.storage[2];
        builder.previous_kernel.note_hashes.storage[2] = tmp;

        builder.previous_kernel.append_nullifiers(3);
        // Shuffle the nullifers so they will have to be re-ordered.
        let tmp = builder.previous_kernel.nullifiers.storage[1];
        builder.previous_kernel.nullifiers.storage[1] = builder.previous_kernel.nullifiers.storage[3];
        builder.previous_kernel.nullifiers.storage[3] = builder.previous_kernel.nullifiers.storage[2];
        builder.previous_kernel.nullifiers.storage[2] = tmp;

        // The nullifier at index 1 is nullifying the note hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the note hash at index 2;
        builder.nullify_pending_note_hash(2, 2);
        // The nullifier at index 3 is nullifying the note hash at index 0;
        builder.nullify_pending_note_hash(3, 0);

        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let public_inputs = builder.execute();

        assert(is_empty_array(public_inputs.end.note_hashes));

        // Only the first nullifier is left after squashing.
        assert(array_eq(public_inputs.end.nullifiers, [nullifiers[0]]));
        assert(is_empty_array(public_inputs.end.note_encrypted_logs_hashes));
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match")]
    fn wrong_transient_nullifier_index_for_note_hash_fails() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(1);
        builder.previous_kernel.append_nullifiers(1);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(1, 0);
        // Change the hint to be out of bounds.
        builder.transient_nullifier_indexes_for_note_hashes[0] = 0;
        builder.failed();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn wrong_transient_nullifier_index_hint_fails() {
        let mut builder = PrivateKernelResetInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(2);
        builder.previous_kernel.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_pending_note_hash(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_pending_note_hash(2, 0);
        // Tweak the hint to be for the hash at index 1.
        builder.transient_note_hash_indexes_for_nullifiers[2] = 1;
        builder.failed();
    }

    #[test]
    fn propagate_fee_payer() {
        // Check that we carry forward if the fee payer is already set
        let mut builder = PrivateKernelResetInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.fee_payer = fee_payer;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer remains empty if unset
        let mut builder = PrivateKernelResetInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelResetInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PrivateKernelResetInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
