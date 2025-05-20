use crate::components::{
    previous_kernel_validator::PreviousKernelValidator, tail_output_composer::TailOutputComposer,
    tail_output_validator::TailOutputValidator,
};
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateToRollupKernelCircuitPublicInputs,
        private_kernel_data::{PrivateKernelData, PrivateKernelDataWithoutPublicInputs},
    },
    constants::{PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX},
    PrivateKernelCircuitPublicInputs,
    proof::traits::Verifiable,
};

global ALLOWED_PREVIOUS_CIRCUITS: [u32; 3] =
    [PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX];

pub struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelTailCircuitPrivateInputs {
    pub fn new(
        previous_kernel: PrivateKernelDataWithoutPublicInputs,
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        Self {
            previous_kernel: previous_kernel.to_private_kernel_data(previous_kernel_public_inputs),
        }
    }

    unconstrained fn generate_output(self) -> PrivateToRollupKernelCircuitPublicInputs {
        TailOutputComposer::new(self.previous_kernel.public_inputs).finish()
    }

    pub fn execute(self) -> PrivateToRollupKernelCircuitPublicInputs {
        if !std::runtime::is_unconstrained() {
            self.previous_kernel.verify();
        }
        // Generate output.
        // Safety: The output is validated below by TailOutputValidator.
        let output = unsafe { self.generate_output() };

        // Validate inputs.
        let previous_kernel_validator = PreviousKernelValidator::new(self.previous_kernel);
        previous_kernel_validator.validate_proof(ALLOWED_PREVIOUS_CIRCUITS);
        previous_kernel_validator.validate_for_private_tail();

        // Validate output.
        if dep::types::validate::should_validate_output() {
            TailOutputValidator::new(output, self.previous_kernel.public_inputs).validate();
        }

        output
    }
}

mod tests {
    use crate::private_kernel_tail::{
        ALLOWED_PREVIOUS_CIRCUITS, PrivateKernelTailCircuitPrivateInputs,
    };
    use dep::types::{
        abis::{
            gas::Gas, kernel_circuit_public_inputs::PrivateToRollupKernelCircuitPublicInputs,
            max_block_number::MaxBlockNumber,
        },
        address::{AztecAddress, EthAddress},
        messaging::l2_to_l1_message::ScopedL2ToL1Message,
        point::Point,
        tests::fixture_builder::FixtureBuilder,
        traits::{Empty, FromField},
    };
    use dep::types::constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, GENERATOR_INDEX__IVSK_M, L2_GAS_PER_CONTRACT_CLASS_LOG,
        L2_GAS_PER_L2_TO_L1_MSG, L2_GAS_PER_NULLIFIER, L2_GAS_PER_PRIVATE_LOG,
        PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_TAIL_INDEX,
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailInputs and PrivateKernelTailToPublicInputs.
    struct PrivateKernelTailInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_INNER_INDEX);
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.set_protocol_nullifier();
            previous_kernel.set_fee_payer(AztecAddress::from_field(234234));
            previous_kernel.is_private_only = true;

            PrivateKernelTailInputsBuilder { previous_kernel }
        }

        pub fn execute(&mut self) -> PrivateToRollupKernelCircuitPublicInputs {
            let kernel = PrivateKernelTailCircuitPrivateInputs {
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

    #[test]
    fn execution_succeeded() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let _public_inputs = builder.execute();
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.rollup_validation_requests.max_block_number.unwrap(), 13);
    }

    #[test]
    fn ordering_of_msgs() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.previous_kernel.append_l2_to_l1_msgs(2);

        // Reorder the msgs
        let original_msgs = builder.previous_kernel.l2_to_l1_msgs.storage();
        let mut reversed_msgs = [ScopedL2ToL1Message::empty(); 2];
        for i in 0..reversed_msgs.len() {
            reversed_msgs[i] = builder.previous_kernel.l2_to_l1_msgs.pop();
        }
        builder.previous_kernel.l2_to_l1_msgs.extend_from_array(reversed_msgs);

        let public_inputs = builder.execute();

        let resulting_msgs = public_inputs.end.l2_to_l1_msgs;

        assert_eq(
            resulting_msgs,
            original_msgs.map(|mut msg: ScopedL2ToL1Message| msg.expose_to_public()),
        );
    }

    #[test(should_fail_with = "Private call stack must be empty when executing the tail circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_private_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with = "Public call stack must be empty when executing the tail circuit")]
    fn non_empty_public_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_public_call_requests(1);
        builder.failed();
    }

    #[test(should_fail_with = "Public teardown call request must be empty when executing the tail circuit")]
    fn non_empty_public_teardown_call_request_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.set_public_teardown_call_request();
        builder.failed();
    }

    #[test(should_fail_with = "split_counter must be 0 for pure private tx")]
    fn non_zero_validation_requests_split_counter_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.validation_requests_split_counter = Option::some(123);
        builder.failed();
    }

    #[test(should_fail_with = "Non empty note hash read requests")]
    fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with = "Non empty nullifier read requests")]
    fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with = "Non empty key validation requests")]
    fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(
            Point { x: 1, y: 2, is_infinite: false },
            27,
            GENERATOR_INDEX__IVSK_M as Field,
        );
        builder.failed();
    }

    #[test(should_fail_with = "First nullifier claim was not satisfied")]
    fn first_nullifier_validation() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let mut first_nullifier = builder.previous_kernel.nullifiers.get(0);
        first_nullifier.nullifier.value += 1;
        builder.previous_kernel.nullifiers.set(0, first_nullifier);
        builder.failed();
    }

    #[test]
    fn empty_tx_consumes_fixed_gas() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();

        // addition follows the form:
        // tx overhead
        // tx nullifier (which has DA and L2 gas)
        let expected_gas_consumed = Gas::tx_overhead()
            + Gas::new(
                DA_GAS_PER_BYTE * DA_BYTES_PER_FIELD * 1,
                L2_GAS_PER_NULLIFIER * 1,
            );
        assert_eq(public_inputs.gas_used, expected_gas_consumed);
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
            Gas::tx_overhead()
                + Gas::new(
                    4 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE,
                    1 * L2_GAS_PER_NULLIFIER + 3 * L2_GAS_PER_L2_TO_L1_MSG,
                ),
            public_inputs.gas_used,
        );
    }

    #[test]
    unconstrained fn tx_consumed_gas_from_logs() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.append_siloed_private_logs_for_note_with_lengths([7], 33);
        builder.previous_kernel.add_contract_class_log_hash(999, 12);
        builder.previous_kernel.end_setup();
        builder.previous_kernel.append_siloed_private_logs_for_note_with_lengths([8, 5], 44);

        let public_inputs = builder.execute();

        let num_nullifiers = 1; // A nullifier is added by default.
        let num_private_logs = 1 + 2;
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

        let l2_gas = num_nullifiers * L2_GAS_PER_NULLIFIER
            + num_private_logs * L2_GAS_PER_PRIVATE_LOG
            + num_cc_logs * L2_GAS_PER_CONTRACT_CLASS_LOG;

        assert_eq(Gas::tx_overhead() + Gas::new(da_gas, l2_gas), public_inputs.gas_used);
    }

    #[test(should_fail_with = "The gas used exceeds the gas limits")]
    fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelTailInputsBuilder::new();
            builder.previous_kernel =
                builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }

    #[test(should_fail_with = "Invalid vk index")]
    fn invalid_previous_kernel() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel = builder.previous_kernel.in_vk_tree(PRIVATE_KERNEL_TAIL_INDEX);
        let _res = builder.execute();
    }
}
