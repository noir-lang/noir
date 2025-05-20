use crate::components::{
    previous_kernel_validator::PreviousKernelValidator,
    tail_to_public_output_composer::TailToPublicOutputComposer,
    tail_to_public_output_validator::TailToPublicOutputValidator,
};
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateToPublicKernelCircuitPublicInputs,
        private_kernel_data::{PrivateKernelData, PrivateKernelDataWithoutPublicInputs},
    },
    constants::{PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX},
    PrivateKernelCircuitPublicInputs,
    proof::traits::Verifiable,
};

global ALLOWED_PREVIOUS_CIRCUITS: [u32; 3] =
    [PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX];

pub struct PrivateKernelTailToPublicCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelTailToPublicCircuitPrivateInputs {
    pub fn new(
        previous_kernel: PrivateKernelDataWithoutPublicInputs,
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        Self {
            previous_kernel: previous_kernel.to_private_kernel_data(previous_kernel_public_inputs),
        }
    }

    unconstrained fn generate_output(self) -> PrivateToPublicKernelCircuitPublicInputs {
        TailToPublicOutputComposer::new(self.previous_kernel.public_inputs).finish()
    }

    pub fn execute(self) -> PrivateToPublicKernelCircuitPublicInputs {
        if !std::runtime::is_unconstrained() {
            self.previous_kernel.verify();
        }
        // Generate output.
        // Safety: The output is validated below by TailToPublicOutputValidator.
        let output = unsafe { self.generate_output() };

        // Validate inputs.
        let previous_kernel_validator = PreviousKernelValidator::new(self.previous_kernel);
        previous_kernel_validator.validate_proof(ALLOWED_PREVIOUS_CIRCUITS);
        previous_kernel_validator.validate_for_private_tail_to_public();

        // Validate output.
        if dep::types::validate::should_validate_output() {
            TailToPublicOutputValidator::new(output, self.previous_kernel.public_inputs).validate();
        }

        output
    }
}

mod tests {
    use crate::private_kernel_tail_to_public::{
        ALLOWED_PREVIOUS_CIRCUITS, PrivateKernelTailToPublicCircuitPrivateInputs,
    };
    use dep::types::{
        abis::{
            gas::Gas, kernel_circuit_public_inputs::PrivateToPublicKernelCircuitPublicInputs,
            note_hash::ScopedNoteHash, nullifier::ScopedNullifier, private_log::PrivateLogData,
            side_effect::scoped::Scoped,
        },
        address::{AztecAddress, EthAddress},
        point::Point,
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq},
        traits::FromField,
    };
    use dep::types::constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS, GENERATOR_INDEX__TSK_M,
        L2_GAS_PER_CONTRACT_CLASS_LOG, L2_GAS_PER_L2_TO_L1_MSG, L2_GAS_PER_NOTE_HASH,
        L2_GAS_PER_NULLIFIER, L2_GAS_PER_PRIVATE_LOG, PRIVATE_KERNEL_INNER_INDEX,
        PRIVATE_KERNEL_TAIL_INDEX,
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailToPublicInputs and PrivateKernelTailInputs.
    struct PrivateKernelTailToPublicInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailToPublicInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_INNER_INDEX);
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.set_protocol_nullifier();
            previous_kernel.set_fee_payer(AztecAddress::from_field(234234));
            previous_kernel.end_setup();
            previous_kernel.append_public_call_requests(1);

            PrivateKernelTailToPublicInputsBuilder { previous_kernel }
        }

        pub fn execute(&mut self) -> PrivateToPublicKernelCircuitPublicInputs {
            let kernel = PrivateKernelTailToPublicCircuitPrivateInputs {
                previous_kernel: self.previous_kernel.to_private_kernel_data(),
            };
            kernel.execute()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test(should_fail_with = "Private call stack must be empty when executing the tail circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_private_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must not be 0")]
    fn zero_min_revertible_side_effect_counter_fails() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.min_revertible_side_effect_counter = 0;
        builder.failed();
    }

    #[test(should_fail_with = "split_counter does not match min_revertible_side_effect_counter")]
    fn mismatch_validation_requests_split_counter_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.min_revertible_side_effect_counter = 123;
        builder.previous_kernel.validation_requests_split_counter = Option::some(4567);

        builder.failed();
    }

    #[test(should_fail_with = "Must have public calls when exporting public kernel data from the tail circuit")]
    fn no_public_calls_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.failed();
    }

    #[test]
    fn can_run_with_only_teardown() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.previous_kernel.set_public_teardown_call_request();

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

        let public_inputs = builder.execute();

        let output_nullifiers = builder
            .previous_kernel
            .nullifiers
            .storage()
            .map(|n: ScopedNullifier| n.nullifier.value);

        assert_array_eq(
            public_inputs.non_revertible_accumulated_data.nullifiers,
            [output_nullifiers[0], output_nullifiers[1], output_nullifiers[2]],
        );

        assert_array_eq(
            public_inputs.revertible_accumulated_data.nullifiers,
            [output_nullifiers[3], output_nullifiers[4]],
        );

        let num_nullifiers = 1 /* tx nullifier */
            + 2 /* non-revertible */
            + 2 /* revertible */;
        let da_gas = num_nullifiers * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;
        let l2_gas = FIXED_AVM_STARTUP_L2_GAS + num_nullifiers * L2_GAS_PER_NULLIFIER;
        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test]
    unconstrained fn split_note_hashes_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        // expect 2 non-revertible note hashes
        builder.previous_kernel.append_siloed_note_hashes(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible note hashes
        builder.previous_kernel.append_siloed_note_hashes(2);

        let public_inputs = builder.execute();

        let exposed_note_hashes = builder
            .previous_kernel
            .note_hashes
            .storage()
            .map(|n: ScopedNoteHash| n.note_hash.value);

        assert_array_eq(
            public_inputs.non_revertible_accumulated_data.note_hashes,
            [exposed_note_hashes[0], exposed_note_hashes[1]],
        );

        assert_array_eq(
            public_inputs.revertible_accumulated_data.note_hashes,
            [exposed_note_hashes[2], exposed_note_hashes[3]],
        );

        let num_note_hashes = 2 /* non-revertible */
            + 2 /* revertible */;
        let num_side_effects = num_note_hashes + 1 /* tx nullifier */;
        let da_gas = (num_side_effects * DA_BYTES_PER_FIELD) * DA_GAS_PER_BYTE;
        let l2_gas = FIXED_AVM_STARTUP_L2_GAS
            + L2_GAS_PER_NULLIFIER
            + num_note_hashes * L2_GAS_PER_NOTE_HASH;
        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test]
    unconstrained fn split_private_logs() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        // expect 2 non-revertible note hashes
        builder.previous_kernel.append_siloed_private_logs_for_note(2, 11);
        builder.previous_kernel.end_setup();

        // expect 2 revertible note hashes
        builder.previous_kernel.append_siloed_private_logs_for_note(3, 12);

        let exposed_private_logs = builder
            .previous_kernel
            .private_logs
            .storage()
            .map(|l: Scoped<PrivateLogData>| l.expose_to_public());

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.non_revertible_accumulated_data.private_logs,
            [exposed_private_logs[0], exposed_private_logs[1]],
        );

        assert_array_eq(
            public_inputs.revertible_accumulated_data.private_logs,
            [exposed_private_logs[2], exposed_private_logs[3], exposed_private_logs[4]],
        );
    }

    #[test(should_fail_with = "Non empty note hash read requests")]
    fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with = "Non empty nullifier read requests")]
    fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with = "Non empty key validation requests")]
    fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(
            Point { x: 1, y: 2, is_infinite: false },
            27,
            GENERATOR_INDEX__TSK_M as Field,
        );
        builder.failed();
    }

    #[test(should_fail_with = "First nullifier claim was not satisfied")]
    fn first_nullifier_validation() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let mut first_nullifier = builder.previous_kernel.nullifiers.get(0);
        first_nullifier.nullifier.value += 1;
        builder.previous_kernel.nullifiers.set(0, first_nullifier);
        builder.failed();
    }

    #[test(should_fail_with = "First nullifier must be non revertible")]
    fn first_nullifier_non_revertible_validation() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let mut first_nullifier = builder.previous_kernel.nullifiers.get(0);
        first_nullifier.nullifier.counter =
            builder.previous_kernel.min_revertible_side_effect_counter + 1;
        builder.previous_kernel.nullifiers.set(0, first_nullifier);
        builder.failed();
    }

    #[test]
    fn empty_tx_consumes_fixed_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.end_setup();
        let public_inputs = builder.execute();

        let da_gas = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE * 1;
        let l2_gas = L2_GAS_PER_NULLIFIER * 1 + FIXED_AVM_STARTUP_L2_GAS;
        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test]
    unconstrained fn enqueued_public_calls_consume_startup_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        let teardown_gas_limits = Gas::new(1, 2);
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = teardown_gas_limits;

        // add an extra non-revertible call
        builder.previous_kernel.append_public_call_requests(1);
        builder.previous_kernel.end_setup();
        // add some revertible calls
        builder.previous_kernel.append_public_call_requests(3);

        let public_inputs = builder.execute();

        let num_public_calls = 2 /* non-revertible */
            + 3 /* revertible */;
        let da_gas = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE * 1;
        let l2_gas = L2_GAS_PER_NULLIFIER * 1 + num_public_calls * FIXED_AVM_STARTUP_L2_GAS;
        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test]
    unconstrained fn enqueued_public_calls_with_teardown_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        let teardown_gas_limits = Gas::new(1, 2);
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = teardown_gas_limits;

        // add an extra non-revertible call
        builder.previous_kernel.append_public_call_requests(1);
        builder.previous_kernel.end_setup();
        // add some revertible calls
        builder.previous_kernel.append_public_call_requests(3);
        // add a teardown call
        builder.previous_kernel.set_public_teardown_call_request();

        let public_inputs = builder.execute();

        let num_public_calls = 2 /* non-revertible */
            + 3 /* revertible */;
        let da_gas = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE * 1;
        let l2_gas = L2_GAS_PER_NULLIFIER * 1 + num_public_calls * FIXED_AVM_STARTUP_L2_GAS;
        assert_eq(
            public_inputs.gas_used,
            Gas::tx_overhead() + Gas::new(da_gas, l2_gas) + teardown_gas_limits,
        );
    }

    #[test]
    unconstrained fn tx_consumes_gas_from_l2_l1_msgs() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.end_setup();
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());

        let public_inputs = builder.execute();

        let num_msgs = 2 /* non-revertible */
            + 1 /* revertible */;
        let num_side_effects = num_msgs + 1 /* tx nullifier */;
        let da_gas = num_side_effects * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;
        let l2_gas = FIXED_AVM_STARTUP_L2_GAS
            + 1 * L2_GAS_PER_NULLIFIER
            + num_msgs * L2_GAS_PER_L2_TO_L1_MSG;
        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test]
    unconstrained fn tx_consumed_gas_from_logs() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.append_siloed_private_logs_for_note_with_lengths([7], 11);
        builder.previous_kernel.add_contract_class_log_hash(420, 12);

        builder.previous_kernel.end_setup();

        builder.previous_kernel.append_siloed_private_logs_for_note_with_lengths([8, 5], 33);

        let public_inputs = builder.execute();

        let num_nullifiers = 1; // A nullifier is added by default.
        let num_private_logs = 3;
        let num_private_log_fields = 7 + 8 + 5;
        let num_cc_logs = 1;
        let num_contract_class_log_fields = 12;

        let num_da_fields = num_nullifiers
            + num_private_log_fields
            + num_private_logs // One extra field per private log for its length.
            + num_contract_class_log_fields
            + num_cc_logs * 2; // Two extra fields per contract class log: length and contract address.
        let num_da_bytes = (num_da_fields * DA_BYTES_PER_FIELD);
        let da_gas = num_da_bytes * DA_GAS_PER_BYTE;

        let l2_gas = FIXED_AVM_STARTUP_L2_GAS
            + num_nullifiers * L2_GAS_PER_NULLIFIER
            + num_private_logs * L2_GAS_PER_PRIVATE_LOG
            + num_cc_logs * L2_GAS_PER_CONTRACT_CLASS_LOG;

        assert_eq(public_inputs.gas_used, Gas::tx_overhead() + Gas::new(da_gas, l2_gas));
    }

    #[test(should_fail_with = "The gas used exceeds the gas limits")]
    fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
            builder.previous_kernel =
                builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }

    #[test(should_fail_with = "Invalid vk index")]
    fn invalid_previous_kernel() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel = builder.previous_kernel.in_vk_tree(PRIVATE_KERNEL_TAIL_INDEX);
        let _res = builder.execute();
    }
}
