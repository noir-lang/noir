use crate::{
    components::{
    private_call_data_validator::PrivateCallDataValidator,
    private_kernel_circuit_output_validator::PrivateKernelCircuitOutputValidator,
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer
}
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    private_kernel::private_call_data::PrivateCallData
},
    transaction::tx_request::TxRequest
};

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInitCircuitPrivateInputs {
    tx_request: TxRequest,
    vk_tree_root: Field,
    private_call: PrivateCallData,
}

impl PrivateKernelInitCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> PrivateKernelCircuitPublicInputs {
        let private_call_public_inputs = self.private_call.call_stack_item.public_inputs;
        PrivateKernelCircuitPublicInputsComposer::new_from_tx_request(self.tx_request, private_call_public_inputs, self.vk_tree_root).with_private_call(
            private_call_public_inputs,
            self.private_call.call_stack_item.contract_address
        ).finish()
    }

    pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        let private_call_data_validator = PrivateCallDataValidator::new(self.private_call);
        private_call_data_validator.validate_as_first_call();
        private_call_data_validator.validate_against_tx_request(self.tx_request);
        private_call_data_validator.validate(output.end.note_hashes);

        // Validate output.
        if !std::runtime::is_unconstrained() {
            PrivateKernelCircuitOutputValidator::new(output).validate_as_first_call(
                self.tx_request,
                self.private_call.call_stack_item.public_inputs,
                private_call_data_validator.array_lengths,
                self.vk_tree_root
            );
        }
        output
    }
}

mod tests {
    use crate::private_kernel_init::PrivateKernelInitCircuitPrivateInputs;
    use dep::types::{
        abis::{kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs},
        tests::{fixture_builder::FixtureBuilder, fixtures, utils::assert_array_eq},
        transaction::tx_request::TxRequest
    };

    struct PrivateKernelInitInputsBuilder {
        tx_request: TxRequest,
        private_call: FixtureBuilder,
    }

    impl PrivateKernelInitInputsBuilder {
        pub fn new() -> Self {
            let private_call = FixtureBuilder::new().is_first_call();
            let tx_request = private_call.build_tx_request();
            PrivateKernelInitInputsBuilder { tx_request, private_call }
        }

        pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
            let private_call = self.private_call.to_private_call_data();
            PrivateKernelInitCircuitPrivateInputs { tx_request: self.tx_request, private_call, vk_tree_root: FixtureBuilder::vk_tree_root() }.execute()
        }
    }

    #[test]
    fn private_kernel_init_output_as_expected() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // note_hash_read_requests
        builder.private_call.append_note_hash_read_requests(2);
        let note_hash_read_requests = builder.private_call.note_hash_read_requests.storage;

        // encrypted_logs_hashes
        builder.private_call.append_encrypted_log_hashes(1);
        let encrypted_log_hashes = builder.private_call.encrypted_logs_hashes.storage;

        let public_inputs = builder.execute();
        assert_array_eq(
            public_inputs.validation_requests.note_hash_read_requests,
            [note_hash_read_requests[0], note_hash_read_requests[1]]
        );
        assert_array_eq(
            public_inputs.end.encrypted_logs_hashes,
            [encrypted_log_hashes[0]]
        );
    }
}
