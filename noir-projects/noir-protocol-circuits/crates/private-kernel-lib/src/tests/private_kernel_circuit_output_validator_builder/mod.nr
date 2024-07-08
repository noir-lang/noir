mod validate_aggregated_values;
mod validate_initial_values;
mod validate_propagated_from_previous_kernel;
mod validate_propagated_from_private_call;

use crate::{
    components::{
    private_kernel_circuit_output_validator::PrivateKernelCircuitOutputValidator,
    private_kernel_circuit_public_inputs_composer::create_first_nullifier
}
};
use dep::types::{
    abis::{
    private_call_request::ScopedPrivateCallRequest,
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputsArrayLengths,
    private_circuit_public_inputs::PrivateCircuitPublicInputsArrayLengths
},
    tests::{fixture_builder::FixtureBuilder}, transaction::tx_request::TxRequest,
    constants::PRIVATE_KERNEL_INIT_INDEX
};

struct PrivateKernelCircuitOutputValidatorBuilder {
    previous_kernel: FixtureBuilder,
    private_call: FixtureBuilder,
    output: FixtureBuilder,
    tx_request: TxRequest,
}

impl PrivateKernelCircuitOutputValidatorBuilder {
    pub fn new() -> Self {
        let mut previous_kernel = FixtureBuilder::new();
        let private_call = FixtureBuilder::new();
        let mut output = FixtureBuilder::new();
        let tx_request = output.build_tx_request();
        let first_nullifier = create_first_nullifier(tx_request);
        output.nullifiers.push(first_nullifier);
        previous_kernel.nullifiers.push(first_nullifier);
        previous_kernel = previous_kernel.in_vk_tree(PRIVATE_KERNEL_INIT_INDEX);

        PrivateKernelCircuitOutputValidatorBuilder { previous_kernel, private_call, output, tx_request }
    }

    pub fn with_previous_kernel_vk_index(&mut self, vk_index: u32) {
        self.previous_kernel = self.previous_kernel.in_vk_tree(vk_index);
    }

    pub fn offset_values(&mut self, num_prepended_items: Field) {
        // Add an offset to the mock values so that the data in the private call won't be the same as those in the previous kernel.
        self.private_call.value_offset = 9999;
        self.private_call.counter = 777;
        self.output.value_offset = 9999 - num_prepended_items;
        self.output.counter = 777;
    }

    pub fn validate_as_first_call(self) {
        let private_call = self.private_call.to_private_call_data();
        let array_lengths = PrivateCircuitPublicInputsArrayLengths::new(private_call.call_stack_item.public_inputs);
        let output = self.output.to_private_kernel_circuit_public_inputs();
        PrivateKernelCircuitOutputValidator::new(output).validate_as_first_call(
            self.tx_request,
            private_call.call_stack_item.public_inputs,
            array_lengths,
            private_call.call_stack_item.contract_address,
            private_call.public_call_stack,
            private_call.public_teardown_call_request,
            FixtureBuilder::vk_tree_root()
        );
    }

    pub fn validate_as_inner_call(self) {
        let mut previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        // Append one private call request for the current call.
        let num_private_call_requests = self.previous_kernel.private_call_requests.len();
        previous_kernel.end.private_call_stack[num_private_call_requests] = ScopedPrivateCallRequest::empty();
        previous_kernel.end.private_call_stack[num_private_call_requests].call_request.args_hash = 98765432;

        let previous_kernel_array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::new(previous_kernel);
        let private_call = self.private_call.to_private_call_data();
        let private_call_array_lengths = PrivateCircuitPublicInputsArrayLengths::new(private_call.call_stack_item.public_inputs);
        let output = self.output.to_private_kernel_circuit_public_inputs();
        PrivateKernelCircuitOutputValidator::new(output).validate_as_inner_call(
            previous_kernel,
            previous_kernel_array_lengths,
            private_call.call_stack_item.public_inputs,
            private_call_array_lengths,
            private_call.call_stack_item.contract_address,
            private_call.public_call_stack,
            private_call.public_teardown_call_request
        );
    }
}
