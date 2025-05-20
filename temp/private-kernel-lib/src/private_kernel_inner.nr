use crate::components::{
    previous_kernel_validator::PreviousKernelValidator,
    private_call_data_validator::PrivateCallDataValidator,
    private_kernel_circuit_output_validator::PrivateKernelCircuitOutputValidator,
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer,
};
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::{
            PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsArrayLengths,
        },
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        private_kernel::private_call_data::{PrivateCallData, PrivateCallDataWithoutPublicInputs},
        private_kernel_data::{PrivateKernelData, PrivateKernelDataWithoutPublicInputs},
    },
    constants::{PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX},
    proof::traits::Verifiable,
};

global ALLOWED_PREVIOUS_CIRCUITS: [u32; 3] =
    [PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, PRIVATE_KERNEL_RESET_INDEX];

pub struct PrivateKernelInnerCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    private_call: PrivateCallData,
}

impl PrivateKernelInnerCircuitPrivateInputs {
    pub fn new(
        previous_kernel: PrivateKernelDataWithoutPublicInputs,
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
        private_call: PrivateCallDataWithoutPublicInputs,
        app_public_inputs: PrivateCircuitPublicInputs,
    ) -> Self {
        Self {
            previous_kernel: previous_kernel.to_private_kernel_data(previous_kernel_public_inputs),
            private_call: private_call.to_private_call_data(app_public_inputs),
        }
    }

    unconstrained fn generate_output(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(
            self.previous_kernel.public_inputs,
        )
            .pop_top_call_request()
            .with_private_call(self.private_call)
            .finish()
    }

    pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
        if !std::runtime::is_unconstrained() {
            self.previous_kernel.verify();
            self.private_call.verify(false);
        }

        // Generate output.
        // Safety: The output is validated below by PrivateKernelCircuitOutputValidator.
        let output = unsafe { self.generate_output() };

        // Validate inputs.
        let previous_kernel_validator = PreviousKernelValidator::new(self.previous_kernel);
        previous_kernel_validator.validate_proof(ALLOWED_PREVIOUS_CIRCUITS);

        let private_call_data_validator = PrivateCallDataValidator::new(self.private_call);
        let previous_kernel_array_lengths =
            PrivateKernelCircuitPublicInputsArrayLengths::new(self.previous_kernel.public_inputs);
        let private_call_stack_size = previous_kernel_array_lengths.private_call_stack;
        let call_request =
            self.previous_kernel.public_inputs.end.private_call_stack[private_call_stack_size - 1];
        private_call_data_validator.validate_against_call_request(call_request);
        private_call_data_validator.validate_against_previous_kernel(
            self.previous_kernel.public_inputs,
        );
        private_call_data_validator.validate(
            output.end.note_hashes,
            self.previous_kernel.public_inputs.constants.protocol_contract_tree_root,
        );

        // Validate output.
        if dep::types::validate::should_validate_output() {
            PrivateKernelCircuitOutputValidator::new(output).validate_as_inner_call(
                self.previous_kernel.public_inputs,
                previous_kernel_array_lengths,
                self.private_call,
                private_call_data_validator.array_lengths,
            );
        }
        output
    }
}

mod tests {
    use crate::private_kernel_inner::{
        ALLOWED_PREVIOUS_CIRCUITS, PrivateKernelInnerCircuitPrivateInputs,
    };
    use dep::types::{
        abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        constants::{PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_TAIL_INDEX},
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq},
    };

    struct PrivateKernelInnerInputsBuilder {
        previous_kernel: FixtureBuilder,
        private_call: FixtureBuilder,
    }

    impl PrivateKernelInnerInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new_from_counter(15)
                .in_vk_tree(PRIVATE_KERNEL_INIT_INDEX)
                .as_parent_contract();
            let private_call = FixtureBuilder::new_from_counter(200);

            // 0th nullifier must be non-zero.
            previous_kernel.append_nullifiers(1);

            PrivateKernelInnerInputsBuilder { previous_kernel, private_call }
        }

        pub fn execute(&mut self) -> PrivateKernelCircuitPublicInputs {
            self.private_call.compute_update_tree_and_hints();
            let private_call = self.private_call.to_private_call_data();
            self.previous_kernel.add_private_call_request_for_private_call(private_call);
            self.previous_kernel.set_historical_header_from_call_data(private_call);
            let previous_kernel = self.previous_kernel.to_private_kernel_data();

            let kernel = PrivateKernelInnerCircuitPrivateInputs { previous_kernel, private_call };

            kernel.execute()
        }
    }

    #[test]
    fn private_kernel_inner_output_as_expected() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // note_hash_read_requests
        builder.previous_kernel.append_note_hash_read_requests(1);
        let prev_note_hash_read_requests =
            builder.previous_kernel.note_hash_read_requests.storage();
        builder.private_call.append_note_hash_read_requests(2);
        let curr_note_hash_read_requests = builder.private_call.note_hash_read_requests.storage();

        // private_logs
        builder.previous_kernel.append_private_logs(2);
        let prev_private_logs = builder.previous_kernel.private_logs.storage();
        builder.private_call.append_private_logs(1);
        let curr_private_logs = builder.private_call.private_logs.storage();

        let public_inputs = builder.execute();
        assert_array_eq(
            public_inputs.validation_requests.note_hash_read_requests,
            [
                prev_note_hash_read_requests[0],
                curr_note_hash_read_requests[0],
                curr_note_hash_read_requests[1],
            ],
        );
        assert_array_eq(
            public_inputs.end.private_logs,
            [prev_private_logs[0], prev_private_logs[1], curr_private_logs[0]],
        );
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelInnerInputsBuilder::new();
            builder.previous_kernel =
                builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }

    #[test(should_fail_with = "Invalid vk index")]
    fn invalid_previous_kernel() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        builder.previous_kernel = builder.previous_kernel.in_vk_tree(PRIVATE_KERNEL_TAIL_INDEX);
        let _res = builder.execute();
    }
}
