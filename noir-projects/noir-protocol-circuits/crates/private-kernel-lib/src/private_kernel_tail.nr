use crate::components::{
    previous_kernel_validator::PreviousKernelValidator, tail_output_composer::TailOutputComposer,
    tail_output_validator::TailOutputValidator
};
use dep::types::{
    abis::{private_kernel_data::PrivateKernelData, kernel_circuit_public_inputs::KernelCircuitPublicInputs},
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

struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelTailCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> KernelCircuitPublicInputs {
        TailOutputComposer::new(self.previous_kernel.public_inputs).finish()
    }

    pub fn execute(self) -> KernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        PreviousKernelValidator::new(self.previous_kernel.public_inputs).validate_for_private_tail();
        if !std::runtime::is_unconstrained() {
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        // Validate output.
        if dep::types::validate::should_validate_output() {
            TailOutputValidator::new(output, self.previous_kernel.public_inputs).validate();
        }

        output
    }
}

mod tests {
    use crate::private_kernel_tail::{PrivateKernelTailCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS};
    use dep::types::constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, GENERATOR_INDEX__IVSK_M, L2_GAS_PER_LOG_BYTE,
        L2_GAS_PER_NULLIFIER, L2_GAS_PER_NOTE_HASH, PRIVATE_KERNEL_INNER_INDEX, BASE_ROLLUP_INDEX
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::KernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        gas::Gas, log_hash::ScopedLogHash
    },
        address::{AztecAddress, EthAddress}, tests::fixture_builder::FixtureBuilder, traits::is_empty,
        point::Point
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailInputs and PrivateKernelTailToPublicInputs.
    struct PrivateKernelTailInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_INNER_INDEX);
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.set_first_nullifier();

            PrivateKernelTailInputsBuilder { previous_kernel }
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let kernel = PrivateKernelTailCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data() };
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
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let public_inputs = builder.execute();

        assert(is_empty(public_inputs.start_state));
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.rollup_validation_requests.max_block_number.unwrap(), 13);
    }

    #[test]
    fn measuring_of_log_lengths() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.add_masked_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
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
    }

    #[test]
    fn ordering_of_unencrypted_logs() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_unencrypted_log_hashes(2);

        // Reorder the logs
        let original_logs = builder.previous_kernel.unencrypted_logs_hashes.storage;
        let mut reversed_logs = [ScopedLogHash::empty(); 2];
        for i in 0..reversed_logs.len() {
            reversed_logs[i] = builder.previous_kernel.unencrypted_logs_hashes.pop();
        }
        builder.previous_kernel.unencrypted_logs_hashes.extend_from_array(reversed_logs);

        let public_inputs = builder.execute();

        let resulting_encrypted_logs = public_inputs.end.unencrypted_logs_hashes;

        assert_eq(
            resulting_encrypted_logs, original_logs.map(
                |mut log: ScopedLogHash| {
               log.log_hash.counter = 0;
               log
            }
            )
        );
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_private_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with="Public call stack must be empty when executing the tail circuit")]
    fn non_empty_public_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_public_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with="Public teardown call request must be empty when executing the tail circuit")]
    fn non_empty_public_teardown_call_request_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_public_teardown_call_request();
        builder.failed();
    }

    #[test(should_fail_with="split_counter must be 0 for pure private tx")]
    fn non_zero_validation_requests_split_counter_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.validation_requests_split_counter = Option::some(123);
        builder.failed();
    }

    #[test(should_fail_with="Non empty note hash read requests")]
    fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty nullifier read requests")]
    fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty key validation requests")]
    fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(
            Point { x: 1, y: 2, is_infinite: false },
            27,
            GENERATOR_INDEX__IVSK_M as Field
        );
        builder.failed();
    }

    #[test]
    fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();

        // addition follows the form:
        // teardown gas
        // tx overhead
        // tx nullifier (which has DA and L2 gas)
        let expected_gas_consumed = Gas::new(300, 300) + Gas::tx_overhead() + Gas::new(
                    DA_GAS_PER_BYTE * DA_BYTES_PER_FIELD * 1,
                    L2_GAS_PER_NULLIFIER * 1
                );
        assert_eq(public_inputs.end.gas_used, expected_gas_consumed);
    }

    #[test]
    unconstrained fn tx_consumes_gas_from_l2_l1_msgs() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());
        builder.previous_kernel.end_setup();
        builder.previous_kernel.add_l2_to_l1_message(42, EthAddress::zero());

        let public_inputs = builder.execute();

        assert_eq(
            Gas::tx_overhead() + Gas::new(
                4 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE,
                1 * L2_GAS_PER_NULLIFIER
            ), public_inputs.end.gas_used
        );
    }

    #[test]
    unconstrained fn tx_consumed_gas_from_logs() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 3);
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 4);
        builder.previous_kernel.add_unencrypted_log_hash(42, 5);
        builder.previous_kernel.end_setup();
        builder.previous_kernel.add_masked_encrypted_log_hash(42, 6);
        builder.previous_kernel.add_unencrypted_log_hash(42, 7);

        let public_inputs = builder.execute();

        assert_eq(
            Gas::tx_overhead() + Gas::new(
                (1 * DA_BYTES_PER_FIELD + 25) * DA_GAS_PER_BYTE ,
                1 * L2_GAS_PER_NULLIFIER + 25 * L2_GAS_PER_LOG_BYTE
            ), public_inputs.end.gas_used
        );
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    fn propagate_fee_payer() {
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

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelTailInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PrivateKernelTailInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
