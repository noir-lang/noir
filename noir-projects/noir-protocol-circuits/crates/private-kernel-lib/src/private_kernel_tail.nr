use crate::components::{
    kernel_circuit_public_inputs_composer::KernelCircuitPublicInputsComposer,
    kernel_circuit_output_hints::generate_hints,
    kernel_circuit_output_validator::KernelCircuitOutputValidator,
    previous_kernel_validator::PreviousKernelValidator
};
use dep::types::{
    abis::{
    private_kernel_data::PrivateKernelData, kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier,
    log_hash::{NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_KEY_VALIDATION_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX
}
};

// TODO: Build hints in noir.
struct PrivateKernelTailHints {
    sorted_new_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hashes_indexes: [u64; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hashes_indexes: [u64; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hashes_indexes: [u64; MAX_UNENCRYPTED_LOGS_PER_TX],
}

struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    hints: PrivateKernelTailHints,
}

impl PrivateKernelTailCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> KernelCircuitPublicInputs {
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
        ).compose().finish()
    }

    pub fn execute(self) -> KernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        PreviousKernelValidator::new(self.previous_kernel.public_inputs).validate_for_private_tail();
        if !dep::std::runtime::is_unconstrained() {
            // verify/aggregate the previous kernel
            self.previous_kernel.verify();
        }

        // Validate output.
        if !dep::std::runtime::is_unconstrained() {
            let hints = generate_hints(self.previous_kernel.public_inputs);
            KernelCircuitOutputValidator::new(output, self.previous_kernel.public_inputs).validate(hints);
        }
        output
    }
}

mod tests {
    use crate::private_kernel_tail::{PrivateKernelTailCircuitPrivateInputs, PrivateKernelTailHints};
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
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX,
        MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, GENERATOR_INDEX__IVSK_M
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::KernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier},
        log_hash::{NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}, gas::Gas
    },
        address::AztecAddress, grumpkin_private_key::GrumpkinPrivateKey,
        hash::{
        sha256_to_field, silo_note_hash, silo_nullifier, compute_siloed_encrypted_log_hash,
        compute_siloed_unencrypted_log_hash
    },
        tests::{fixture_builder::FixtureBuilder, sort::sort_get_sorted_hints},
        utils::{arrays::array_length}, traits::{Empty, is_empty, is_empty_array},
        grumpkin_point::GrumpkinPoint
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailInputs and PrivateKernelTailToPublicInputs.
    struct PrivateKernelTailInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);

            previous_kernel.append_new_nullifiers(1);

            PrivateKernelTailInputsBuilder { previous_kernel }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_output_note_hashes<N>(self, note_hashes: [ScopedNoteHash; N]) -> [Field; N] {
            let first_nullifier = self.previous_kernel.new_nullifiers.get_unchecked(0).value();
            let mut output = [0; N];
            for i in 0..N {
                output[i] = silo_note_hash(note_hashes[i], first_nullifier, i);
            }
            output
        }

        pub fn compute_output_nullifiers<N>(_self: Self, nullifiers: [ScopedNullifier; N]) -> [Field; N] {
            let mut output = [0; N];
            output[0] = nullifiers[0].value();
            for i in 1..N {
                output[i] = silo_nullifier(nullifiers[i]);
            }
            output
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
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

            let hints = PrivateKernelTailHints {
                sorted_new_note_hashes,
                sorted_new_note_hashes_indexes,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                sorted_note_encrypted_log_hashes,
                sorted_note_encrypted_log_hashes_indexes,
                sorted_encrypted_log_hashes,
                sorted_encrypted_log_hashes_indexes,
                sorted_unencrypted_log_hashes,
                sorted_unencrypted_log_hashes_indexes
            };

            let kernel = PrivateKernelTailCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data(), hints };
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
    unconstrained fn execution_succeeded() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let public_inputs = builder.execute();

        assert(is_empty(public_inputs.start_state));
    }

    #[test]
    unconstrained fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.rollup_validation_requests.max_block_number.unwrap(), 13);
    }

    #[test]
    unconstrained fn logs_are_handled_as_expected() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.add_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.add_unencrypted_log_hash(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.previous_kernel.add_unencrypted_log_hash(unencrypted_logs_hash, unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        let siloed_encrypted_logs_hash = compute_siloed_encrypted_log_hash(
            builder.previous_kernel.storage_contract_address,
            builder.previous_kernel.encrypted_logs_hashes.storage[0].log_hash.randomness,
            prev_encrypted_logs_hash
        );

        // noir-fmt:ignore
        let hash_bytes: [u8; MAX_ENCRYPTED_LOGS_PER_TX * 32] = siloed_encrypted_logs_hash
            .to_be_bytes(32)
            .append(&[0; MAX_ENCRYPTED_LOGS_PER_TX * 32 - 32])
            .as_array();
        let expected_encrypted_logs_hash = sha256_to_field(hash_bytes);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

        let siloed_unencrypted_logs_hashes = [
            compute_siloed_unencrypted_log_hash(
                builder.previous_kernel.storage_contract_address,
                prev_unencrypted_logs_hash
            ), compute_siloed_unencrypted_log_hash(
                builder.previous_kernel.storage_contract_address,
                unencrypted_logs_hash
            )
        ];
        // noir-fmt:ignore
        let hash_bytes: [u8; MAX_UNENCRYPTED_LOGS_PER_TX * 32] = siloed_unencrypted_logs_hashes[0]
            .to_be_bytes(32)
            .append(siloed_unencrypted_logs_hashes[1].to_be_bytes(32))
            .append(&[0; MAX_UNENCRYPTED_LOGS_PER_TX * 32 - 64])
            .as_array();
        let expected_unencrypted_logs_hash = sha256_to_field(hash_bytes);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test]
    unconstrained fn encrypted_logs_are_hashed_as_expected() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        builder.previous_kernel.add_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);

        // Set the randomness to 0 to signal not masking the address to silo with
        builder.previous_kernel.encrypted_logs_hashes.storage[0].log_hash.randomness = 0;

        let new_encrypted_logs_hash = 26;
        let new_encrypted_log_preimages_length = 50;
        builder.previous_kernel.add_encrypted_log_hash(new_encrypted_logs_hash, new_encrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(
            public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length + new_encrypted_log_preimages_length
        );

        let siloed_encrypted_logs_hash = compute_siloed_encrypted_log_hash(
            builder.previous_kernel.storage_contract_address,
            builder.previous_kernel.encrypted_logs_hashes.storage[0].log_hash.randomness,
            prev_encrypted_logs_hash
        );

        let siloed_hash_bytes: [u8; 64] = builder.previous_kernel.storage_contract_address.to_field().to_be_bytes(32).append(prev_encrypted_logs_hash.to_be_bytes(32)).as_array();
        assert_eq(siloed_encrypted_logs_hash, sha256_to_field(siloed_hash_bytes));

        let new_siloed_encrypted_logs_hash = compute_siloed_encrypted_log_hash(
            builder.previous_kernel.storage_contract_address,
            builder.previous_kernel.encrypted_logs_hashes.storage[1].log_hash.randomness,
            new_encrypted_logs_hash
        );
        // noir-fmt:ignore
        let hash_bytes: [u8; MAX_ENCRYPTED_LOGS_PER_TX * 32] = siloed_encrypted_logs_hash
            .to_be_bytes(32)
            .append(new_siloed_encrypted_logs_hash.to_be_bytes(32))
            .append(&[0; MAX_ENCRYPTED_LOGS_PER_TX * 32 - 64])
            .as_array();
        let expected_encrypted_logs_hash = sha256_to_field(hash_bytes);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);
    }

    #[test]
    unconstrained fn ordering_of_note_hashes_and_nullifiers() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

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

        let expected_note_hashes = builder.compute_output_note_hashes(sorted_note_hashes);
        let expected_nullifiers = builder.compute_output_nullifiers(sorted_nullifiers);
        for i in 0..10 {
            assert(public_inputs.end.new_note_hashes[i].eq(expected_note_hashes[i]));
            assert(public_inputs.end.new_nullifiers[i].eq(expected_nullifiers[i]));
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

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.add_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Public call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_public_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.push_public_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Public teardown call request must be empty when executing the tail circuit")]
    unconstrained fn non_empty_public_teardown_call_request_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.push_public_teardown_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Non empty note hash read requests")]
    unconstrained fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty nullifier read requests")]
    unconstrained fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_new_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty key validation requests")]
    unconstrained fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(GrumpkinPoint::new(1, 2), 27, GENERATOR_INDEX__IVSK_M);
        builder.failed();
    }

    #[test]
    unconstrained fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();

        // addition follows the form:
        // teardown gas
        // tx overhead
        // tx nullifier
        let expected_gas_consumed = Gas::new(300, 300)
            + Gas::tx_overhead()
            + Gas::new(DA_GAS_PER_BYTE * DA_BYTES_PER_FIELD * 1, 0);
        assert_eq(public_inputs.end.gas_used, expected_gas_consumed);
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    unconstrained fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        // Check that we carry forward if the fee payer is already set
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.fee_payer = fee_payer;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer remains empty if unset
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }
}
