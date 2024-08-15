use crate::components::{
    previous_kernel_validator::PreviousKernelValidator,
    tail_to_public_output_composer::TailToPublicOutputComposer,
    tail_to_public_output_validator::TailToPublicOutputValidator
};
use dep::types::{
    abis::{private_kernel_data::PrivateKernelData, kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs},
    constants::{
    PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_FULL_INDEX,
    PRIVATE_KERNEL_RESET_BIG_INDEX, PRIVATE_KERNEL_RESET_MEDIUM_INDEX, PRIVATE_KERNEL_RESET_SMALL_INDEX,
    PRIVATE_KERNEL_RESET_TINY_INDEX
}
};

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_INIT_INDEX,
    PRIVATE_KERNEL_INNER_INDEX,
    PRIVATE_KERNEL_RESET_FULL_INDEX,
    PRIVATE_KERNEL_RESET_BIG_INDEX,
    PRIVATE_KERNEL_RESET_MEDIUM_INDEX,
    PRIVATE_KERNEL_RESET_SMALL_INDEX,
    PRIVATE_KERNEL_RESET_TINY_INDEX,
];

struct PrivateKernelTailToPublicCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelTailToPublicCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> PublicKernelCircuitPublicInputs {
        TailToPublicOutputComposer::new(self.previous_kernel.public_inputs).finish()
    }

    pub fn execute(self) -> PublicKernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        PreviousKernelValidator::new(self.previous_kernel.public_inputs).validate_for_private_tail_to_public();
        if !std::runtime::is_unconstrained() {
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        // Validate output.
        if dep::types::validate::should_validate_output() {
            TailToPublicOutputValidator::new(output, self.previous_kernel.public_inputs).validate();
        }

        output
    }
}

mod tests {
    use crate::private_kernel_tail_to_public::{PrivateKernelTailToPublicCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS};
    use dep::types::constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, GENERATOR_INDEX__TSK_M, L2_GAS_PER_LOG_BYTE,
        L2_GAS_PER_NOTE_HASH, L2_GAS_PER_NULLIFIER, FIXED_AVM_STARTUP_L2_GAS
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, gas::Gas,
        note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier},
        log_hash::{LogHash, NoteLogHash}
    },
        address::{AztecAddress, EthAddress},
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq}, point::Point,
        constants::{BASE_ROLLUP_INDEX, PRIVATE_KERNEL_INNER_INDEX}
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailToPublicInputs and PrivateKernelTailInputs.
    struct PrivateKernelTailToPublicInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailToPublicInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_INNER_INDEX);
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.set_first_nullifier();
            previous_kernel.end_setup();
            previous_kernel.append_public_call_requests(1);

            PrivateKernelTailToPublicInputsBuilder { previous_kernel }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_output_note_hashes<let N: u32>(_self: Self, note_hashes: [ScopedNoteHash; N]) -> [ScopedNoteHash; N] {
            let mut output = [ScopedNoteHash::empty(); N];
            for i in 0..N {
                output[i].note_hash.value = note_hashes[i].value();
            }
            output
        }

        pub fn compute_output_nullifiers<let N: u32>(_self: Self, nullifiers: [ScopedNullifier; N]) -> [Nullifier; N] {
            let mut output = [Nullifier::empty(); N];
            for i in 0..N {
                output[i].value = nullifiers[i].value();
            }
            output
        }

        pub fn compute_output_note_logs<let N: u32>(_self: Self, logs: [NoteLogHash; N]) -> [LogHash; N] {
            let mut output = [LogHash::empty(); N];
            for i in 0..N {
                if logs[i].value != 0 {
                    output[i] = logs[i].expose_to_public(); // Counter is cleared so it's not exposed to the public.
                }
            }
            output
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let kernel = PrivateKernelTailToPublicCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data() };
            kernel.execute()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_private_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with="min_revertible_side_effect_counter must not be 0")]
    fn zero_min_revertible_side_effect_counter_fails() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.min_revertible_side_effect_counter = 0;
        builder.failed();
    }

    #[test(should_fail_with="split_counter does not match min_revertible_side_effect_counter")]
    fn mismatch_validation_requests_split_counter_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.min_revertible_side_effect_counter = 123;
        builder.previous_kernel.validation_requests_split_counter = Option::some(4567);

        builder.failed();
    }

    #[test(should_fail_with="Must have public calls when exporting public kernel data from the tail circuit")]
    fn no_public_calls_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.failed();
    }

    #[test]
    fn can_run_with_only_teardown() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.previous_kernel.append_public_teardown_call_request();

        builder.succeeded();
    }

    #[test]
    fn split_nullifiers_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        // expect 3 non-revertible nullifiers: the tx nullifier + 2 new ones
        builder.previous_kernel.append_siloed_nullifiers(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible nullifiers
        builder.previous_kernel.append_siloed_nullifiers(2);

        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let public_inputs = builder.execute();

        let output_nullifiers = builder.compute_output_nullifiers(nullifiers);

        assert_array_eq(
            public_inputs.end_non_revertible.nullifiers,
            [output_nullifiers[0], output_nullifiers[1], output_nullifiers[2]]
        );

        assert_array_eq(
            public_inputs.end.nullifiers,
            [output_nullifiers[3], output_nullifiers[4]]
        );

        assert_eq(
            public_inputs.end.gas_used, Gas::new(
                2 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE,
                2 * L2_GAS_PER_NULLIFIER
            )
        );
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(
            3 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE,
            FIXED_AVM_STARTUP_L2_GAS + 3 * L2_GAS_PER_NULLIFIER
        )
            + Gas::tx_overhead()
        );
    }

    #[test]
    fn split_note_hashes_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        // expect 2 non-revertible note hashes
        builder.previous_kernel.append_siloed_note_hashes(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible note hashes
        builder.previous_kernel.append_siloed_note_hashes(2);

        let note_hashes = builder.previous_kernel.note_hashes.storage;
        let public_inputs = builder.execute();

        let exposed_note_hashes = builder.compute_output_note_hashes(note_hashes);

        assert_array_eq(
            public_inputs.end_non_revertible.note_hashes,
            [exposed_note_hashes[0], exposed_note_hashes[1]]
        );

        assert_array_eq(
            public_inputs.end.note_hashes,
            [exposed_note_hashes[2], exposed_note_hashes[3]]
        );

        assert_eq(
            public_inputs.end.gas_used, Gas::new(
                (2 * DA_BYTES_PER_FIELD) * DA_GAS_PER_BYTE,
                2 * L2_GAS_PER_NOTE_HASH
            )
        );
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(
            (3 * DA_BYTES_PER_FIELD) * DA_GAS_PER_BYTE,
            FIXED_AVM_STARTUP_L2_GAS + L2_GAS_PER_NULLIFIER + 2 * L2_GAS_PER_NOTE_HASH
        )
            + Gas::tx_overhead()
        );
    }

    #[test(should_fail_with="Non empty note hash read requests")]
    fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty nullifier read requests")]
    fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty key validation requests")]
    fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(
            Point { x: 1, y: 2, is_infinite: false },
            27,
            GENERATOR_INDEX__TSK_M as Field
        );
        builder.failed();
    }

    #[test]
    fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.end_setup();
        let public_inputs = builder.execute();

        let expected_revertible_gas_used = Gas::new(300, 300);
        assert_eq(public_inputs.end.gas_used, expected_revertible_gas_used);

        let expected_non_revertible_gas_used = Gas::tx_overhead() + Gas::new(
                DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE * 1,
                L2_GAS_PER_NULLIFIER * 1 + FIXED_AVM_STARTUP_L2_GAS
            );

        assert_eq(public_inputs.end_non_revertible.gas_used, expected_non_revertible_gas_used);
    }

    #[test]
    unconstrained fn enqueued_public_calls_consume_startup_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        // add an extra non-revertible call
        builder.previous_kernel.append_public_call_requests(1);
        builder.previous_kernel.end_setup();
        // add some revertible calls
        builder.previous_kernel.append_public_call_requests(3);
        let public_inputs = builder.execute();

        let expected_revertible_gas_used = Gas::new(0, 3 * FIXED_AVM_STARTUP_L2_GAS);
        assert_eq(public_inputs.end.gas_used, expected_revertible_gas_used);

        let expected_non_revertible_gas_used = Gas::tx_overhead() + Gas::new(
                DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE * 1,
                L2_GAS_PER_NULLIFIER * 1 + 2 * FIXED_AVM_STARTUP_L2_GAS
            );

        assert_eq(public_inputs.end_non_revertible.gas_used, expected_non_revertible_gas_used);
    }

    #[test]
    unconstrained fn tx_consumes_gas_from_l2_l1_msgs() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.end_setup();
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());

        let public_inputs = builder.execute();

        assert_eq(Gas::new(1 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0), public_inputs.end.gas_used);
        assert_eq(
            Gas::tx_overhead() + Gas::new(
                3 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE,
                FIXED_AVM_STARTUP_L2_GAS + 1 * L2_GAS_PER_NULLIFIER
            ), public_inputs.end_non_revertible.gas_used
        );
    }

    #[test]
    unconstrained fn tx_consumed_gas_from_logs() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 3);
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 4);
        builder.previous_kernel.add_unencrypted_log_hash(42, 5);
        builder.previous_kernel.end_setup();
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 6);
        builder.previous_kernel.add_unencrypted_log_hash(42, 7);

        let public_inputs = builder.execute();

        assert_eq(Gas::new(13 * DA_GAS_PER_BYTE, 13 * L2_GAS_PER_LOG_BYTE), public_inputs.end.gas_used);

        assert_eq(
            Gas::tx_overhead() + Gas::new(
                (1 * DA_BYTES_PER_FIELD + 12) * DA_GAS_PER_BYTE ,
                FIXED_AVM_STARTUP_L2_GAS + 1 * L2_GAS_PER_NULLIFIER + 12 * L2_GAS_PER_LOG_BYTE
            ), public_inputs.end_non_revertible.gas_used
        );
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    fn propagate_fee_payer() {
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

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
