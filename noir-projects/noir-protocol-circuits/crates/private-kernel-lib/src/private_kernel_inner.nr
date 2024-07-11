use crate::{
    components::{
    private_call_data_validator::PrivateCallDataValidator,
    private_kernel_circuit_output_validator::PrivateKernelCircuitOutputValidator,
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer
}
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsArrayLengths},
    private_kernel_data::PrivateKernelData, private_kernel::private_call_data::PrivateCallData
},
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

struct PrivateKernelInnerCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    private_call: PrivateCallData,
}

impl PrivateKernelInnerCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(self.previous_kernel.public_inputs).pop_top_call_request().with_private_call(
            self.private_call.call_stack_item.public_inputs,
            self.private_call.call_stack_item.contract_address,
            self.private_call.public_call_stack,
            self.private_call.public_teardown_call_request
        ).finish()
    }

    pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        let private_call_data_validator = PrivateCallDataValidator::new(self.private_call);
        let previous_kernel_array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::new(self.previous_kernel.public_inputs);
        let private_call_stack_size = previous_kernel_array_lengths.private_call_stack;
        let call_request = self.previous_kernel.public_inputs.end.private_call_stack[private_call_stack_size - 1];
        private_call_data_validator.validate_against_call_request(call_request);
        private_call_data_validator.validate_against_previous_kernel(self.previous_kernel.public_inputs);
        private_call_data_validator.validate(output.end.note_hashes);
        if !std::runtime::is_unconstrained() {
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        // Validate output.
        if !std::runtime::is_unconstrained() {
            PrivateKernelCircuitOutputValidator::new(output).validate_as_inner_call(
                self.previous_kernel.public_inputs,
                previous_kernel_array_lengths,
                self.private_call.call_stack_item.public_inputs,
                private_call_data_validator.array_lengths,
                self.private_call.call_stack_item.contract_address,
                self.private_call.public_call_stack,
                self.private_call.public_teardown_call_request
            );
        }
        output
    }
}

mod tests {
    use crate::private_kernel_inner::{PrivateKernelInnerCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS};
    use dep::types::{
        abis::{kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs},
        constants::{PRIVATE_KERNEL_INIT_INDEX, PRIVATE_KERNEL_INNER_INDEX, BASE_ROLLUP_INDEX},
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq}
    };

    struct PrivateKernelInnerInputsBuilder {
        previous_kernel: FixtureBuilder,
        private_call: FixtureBuilder,
    }

    impl PrivateKernelInnerInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new_from_counter(15).in_vk_tree(PRIVATE_KERNEL_INIT_INDEX).as_parent_contract();
            let private_call = FixtureBuilder::new_from_counter(200);

            // 0th nullifier must be non-zero.
            previous_kernel.append_nullifiers(1);

            PrivateKernelInnerInputsBuilder { previous_kernel, private_call }
        }

        pub fn execute(&mut self) -> PrivateKernelCircuitPublicInputs {
            let private_call = self.private_call.to_private_call_data();
            // Update the previous_kernel's private_call_stack with the current call_stack_item.
            self.previous_kernel.add_private_call_from_call_stack_item(private_call.call_stack_item);
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
        let prev_note_hash_read_requests = builder.previous_kernel.note_hash_read_requests.storage;
        builder.private_call.append_note_hash_read_requests(2);
        let curr_note_hash_read_requests = builder.private_call.note_hash_read_requests.storage;

        // encrypted_logs_hashes
        builder.previous_kernel.append_encrypted_log_hashes(2);
        let prev_encrypted_log_hashes = builder.previous_kernel.encrypted_logs_hashes.storage;
        builder.private_call.append_encrypted_log_hashes(1);
        let curr_encrypted_log_hashes = builder.private_call.encrypted_logs_hashes.storage;

        let public_inputs = builder.execute();
        assert_array_eq(
            public_inputs.validation_requests.note_hash_read_requests,
            [
            prev_note_hash_read_requests[0], curr_note_hash_read_requests[0], curr_note_hash_read_requests[1]
        ]
        );
        assert_array_eq(
            public_inputs.end.encrypted_logs_hashes,
            [prev_encrypted_log_hashes[0], prev_encrypted_log_hashes[1], curr_encrypted_log_hashes[0]]
        );
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PrivateKernelInnerInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PrivateKernelInnerInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
