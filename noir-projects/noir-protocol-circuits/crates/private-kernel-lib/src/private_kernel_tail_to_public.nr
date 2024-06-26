use crate::components::{
    kernel_circuit_public_inputs_composer::KernelCircuitPublicInputsComposer,
    previous_kernel_validator::PreviousKernelValidator
};
use dep::types::{
    abis::{
    private_kernel_data::PrivateKernelData,
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier, log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash},
    call_request::CallRequest
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_KEY_VALIDATION_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX
}
};

struct PrivateKernelTailToPublicHints {
    sorted_new_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_note_hashes_indexes: [u32; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u32; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hashes_indexes: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes_indexes: [u32; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes_indexes: [u32; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_call_requests: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    sorted_call_requests_indexes: [u32; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
}

struct PrivateKernelTailToPublicCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    hints: PrivateKernelTailToPublicHints,
}

impl PrivateKernelTailToPublicCircuitPrivateInputs {
    pub fn execute(self) -> PublicKernelCircuitPublicInputs {
        // Validate inputs.
        PreviousKernelValidator::new(self.previous_kernel.public_inputs).validate_for_private_tail_to_public();
        if !std::runtime::is_unconstrained() {
            // verify/aggregate the previous kernel
            self.previous_kernel.verify();
        }

        KernelCircuitPublicInputsComposer::new(
            self.previous_kernel,
            self.hints.sorted_new_note_hashes,
            self.hints.sorted_new_note_hashes_indexes,
            self.hints.sorted_new_nullifiers,
            self.hints.sorted_new_nullifiers_indexes,
            self.hints.sorted_note_encrypted_log_hashes,
            self.hints.sorted_note_encrypted_log_hashes_indexes,
            self.hints.sorted_encrypted_log_hashes,
            self.hints.sorted_encrypted_log_hashes_indexes,
            self.hints.sorted_unencrypted_log_hashes,
            self.hints.sorted_unencrypted_log_hashes_indexes
        ).compose_public(
            self.hints.sorted_call_requests,
            self.hints.sorted_call_requests_indexes
        ).finish_to_public()
    }
}

mod tests {
    use crate::private_kernel_tail_to_public::{PrivateKernelTailToPublicCircuitPrivateInputs, PrivateKernelTailToPublicHints};
    use dep::reset_kernel_lib::{
        tests::{
        note_hash_read_request_hints_builder::NoteHashReadRequestHintsBuilder,
        nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder,
        squash_transient_data::{squash_transient_note_hashes, squash_transient_nullifiers, squash_transient_logs}
    },
        reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::constants::{
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX, DA_BYTES_PER_FIELD,
        DA_GAS_PER_BYTE, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, GENERATOR_INDEX__TSK_M
    };
    use dep::types::{
        abis::{
        call_request::CallRequest, side_effect::Ordered,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, gas::Gas,
        note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier},
        log_hash::{LogHash, ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}
    },
        address::AztecAddress, hash::{silo_note_hash, silo_nullifier},
        tests::{fixture_builder::FixtureBuilder, sort::sort_get_sorted_hints},
        utils::{arrays::{array_eq, array_length}}, traits::is_empty_array, grumpkin_point::GrumpkinPoint
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailToPublicInputs and PrivateKernelTailInputs.
    struct PrivateKernelTailToPublicInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailToPublicInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.append_new_nullifiers(1);
            previous_kernel.push_public_call_request(1, false);

            PrivateKernelTailToPublicInputsBuilder { previous_kernel }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_output_note_hashes<N>(self, note_hashes: [ScopedNoteHash; N]) -> [NoteHash; N] {
            // First nullifier is tx hash.
            let tx_hash = self.previous_kernel.new_nullifiers.get_unchecked(0).value();
            let mut output = [NoteHash::empty(); N];
            for i in 0..N {
                output[i] = NoteHash {
                    value: silo_note_hash(note_hashes[i], tx_hash, i),
                    counter: 0, // Counter is cleared so it's not exposed to the public.
                };
            }
            output
        }

        pub fn compute_output_nullifiers<N>(_self: Self, nullifiers: [ScopedNullifier; N]) -> [Nullifier; N] {
            let mut output = [Nullifier::empty(); N];
            output[0].value = nullifiers[0].value();
            for i in 1..N {
                output[i] = Nullifier { value: silo_nullifier(nullifiers[i]), counter: 0, note_hash: 0 };
            }
            output
        }

        pub fn compute_output_note_logs<N>(_self: Self, logs: [NoteLogHash; N]) -> [LogHash; N] {
            let mut output = [LogHash::empty(); N];
            for i in 0..N {
                if logs[i].value != 0 {
                    output[i] = logs[i].expose_to_public(); // Counter is cleared so it's not exposed to the public.
                }
            }
            output
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_note_hashes.storage,
                |a: ScopedNoteHash, b: ScopedNoteHash| a.counter() < b.counter()
            );
            let sorted_new_note_hashes = sorted.sorted_array;
            let sorted_new_note_hashes_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.new_nullifiers.storage,
                |a: ScopedNullifier, b: ScopedNullifier| a.counter() < b.counter()
            );
            let sorted_new_nullifiers = sorted.sorted_array;
            let sorted_new_nullifiers_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.note_encrypted_logs_hashes.storage,
                |a: NoteLogHash, b: NoteLogHash| a.counter < b.counter
            );
            let sorted_note_encrypted_log_hashes = sorted.sorted_array;
            let sorted_note_encrypted_log_hashes_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.encrypted_logs_hashes.storage,
                |a: ScopedEncryptedLogHash, b: ScopedEncryptedLogHash| a.counter() < b.counter()
            );
            let sorted_encrypted_log_hashes = sorted.sorted_array;
            let sorted_encrypted_log_hashes_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.unencrypted_logs_hashes.storage,
                |a: ScopedLogHash, b: ScopedLogHash| a.counter() < b.counter()
            );
            let sorted_unencrypted_log_hashes = sorted.sorted_array;
            let sorted_unencrypted_log_hashes_indexes = sorted.sorted_index_hints;

            let sorted = sort_get_sorted_hints(
                self.previous_kernel.public_call_requests.storage,
                |a: CallRequest, b: CallRequest| a.counter() > b.counter()
            );
            let sorted_call_requests = sorted.sorted_array;
            let sorted_call_requests_indexes = sorted.sorted_index_hints;

            let hints = PrivateKernelTailToPublicHints {
                sorted_new_note_hashes,
                sorted_new_note_hashes_indexes,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                sorted_note_encrypted_log_hashes,
                sorted_note_encrypted_log_hashes_indexes,
                sorted_encrypted_log_hashes,
                sorted_encrypted_log_hashes_indexes,
                sorted_unencrypted_log_hashes,
                sorted_unencrypted_log_hashes_indexes,
                sorted_call_requests,
                sorted_call_requests_indexes
            };

            let kernel = PrivateKernelTailToPublicCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data(), hints };
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
    unconstrained fn ordering_of_note_hashes_and_nullifiers() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.append_new_note_hashes(10);
        builder.previous_kernel.append_new_nullifiers(10);

        let sorted_note_hashes = builder.previous_kernel.new_note_hashes.storage;
        let sorted_nullifiers = builder.previous_kernel.new_nullifiers.storage;

        let mut reversed_note_hashes = [ScopedNoteHash::empty(); 10];
        let mut reversed_nullifiers = [ScopedNullifier::empty(); 10];

        for i in 0..10 {
            reversed_note_hashes[9 - i] = builder.previous_kernel.new_note_hashes.pop();
            reversed_nullifiers[9 - i] = builder.previous_kernel.new_nullifiers.pop();
        }

        builder.previous_kernel.new_note_hashes.extend_from_array(reversed_note_hashes);
        builder.previous_kernel.new_nullifiers.extend_from_array(reversed_nullifiers);

        let public_inputs = builder.execute();

        let output_note_hashes = builder.compute_output_note_hashes(sorted_note_hashes);
        let output_nullifiers = builder.compute_output_nullifiers(sorted_nullifiers);
        for i in 0..10 {
            assert(public_inputs.end.new_note_hashes[i].eq(output_note_hashes[i]));
            assert(public_inputs.end.new_nullifiers[i].eq(output_nullifiers[i]));
        }
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.add_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Must have public calls when exporting public kernel data from the tail circuit")]
    unconstrained fn no_public_calls_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.failed();
    }

    #[test]
    unconstrained fn can_run_with_only_teardown() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.previous_kernel.push_public_teardown_call_request(1, false);

        builder.succeeded();
    }

    #[test]
    unconstrained fn split_nullifiers_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        // expect 3 non-revertible nullifiers: the tx nullifier + 2 new ones
        builder.previous_kernel.append_new_nullifiers(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible nullifiers
        builder.previous_kernel.append_new_nullifiers(2);

        let new_nullifiers = builder.previous_kernel.new_nullifiers.storage;
        let public_inputs = builder.execute();

        let output_nullifiers = builder.compute_output_nullifiers(new_nullifiers);

        assert(
            array_eq(
                public_inputs.end_non_revertible.new_nullifiers,
                [output_nullifiers[0], output_nullifiers[1], output_nullifiers[2]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.new_nullifiers,
                [output_nullifiers[3], output_nullifiers[4]]
            )
        );

        assert_eq(public_inputs.end.gas_used, Gas::new(2 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0));
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(3 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0) + Gas::tx_overhead()
        );
    }

    #[test]
    unconstrained fn split_note_hashes_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        // expect 2 non-revertible note hashes
        builder.previous_kernel.append_new_note_hashes_with_logs(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible note hashes
        builder.previous_kernel.append_new_note_hashes_with_logs(2);

        let new_note_hashes = builder.previous_kernel.new_note_hashes.storage;
        let new_note_logs = builder.previous_kernel.note_encrypted_logs_hashes.storage;
        let public_inputs = builder.execute();

        let siloed_note_hashes = builder.compute_output_note_hashes(new_note_hashes);
        let public_note_logs = builder.compute_output_note_logs(new_note_logs);

        assert(
            array_eq(
                public_inputs.end_non_revertible.new_note_hashes,
                [siloed_note_hashes[0], siloed_note_hashes[1]]
            )
        );

        assert(
            array_eq(
                public_inputs.end_non_revertible.note_encrypted_logs_hashes,
                [public_note_logs[0], public_note_logs[1]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.new_note_hashes,
                [siloed_note_hashes[2], siloed_note_hashes[3]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.note_encrypted_logs_hashes,
                [public_note_logs[2], public_note_logs[3]]
            )
        );

        let revertible_logs_len = (new_note_logs[2].length + new_note_logs[3].length) as u32;
        let non_revertible_logs_len = (new_note_logs[0].length + new_note_logs[1].length) as u32;

        assert_eq(
            public_inputs.end.gas_used, Gas::new(
                (2 * DA_BYTES_PER_FIELD + revertible_logs_len) * DA_GAS_PER_BYTE,
                0
            )
        );
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(
            (3 * DA_BYTES_PER_FIELD + non_revertible_logs_len) * DA_GAS_PER_BYTE,
            0
        )
            + Gas::tx_overhead()
        );
    }

    #[test(should_fail_with="Non empty note hash read requests")]
    unconstrained fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty nullifier read requests")]
    unconstrained fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_new_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty key validation requests")]
    unconstrained fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(GrumpkinPoint::new(1, 2), 27, GENERATOR_INDEX__TSK_M);
        builder.failed();
    }

    #[test]
    unconstrained fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();

        let expected_gas_consumed = Gas::new(300, 300) + Gas::tx_overhead();
        assert_eq(public_inputs.end.gas_used, expected_gas_consumed);
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    unconstrained fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        // Check that we carry forward if the fee payer is already set
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.fee_payer = fee_payer;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer remains empty if unset
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }
}
