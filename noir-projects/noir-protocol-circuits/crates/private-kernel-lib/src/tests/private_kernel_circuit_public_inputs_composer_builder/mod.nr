mod new_from_previous_kernel_with_private_call;
mod new_from_tx_request;
mod propagate_from_private_call;

use crate::components::private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    private_call_request::PrivateCallRequest
},
    tests::fixture_builder::FixtureBuilder, transaction::tx_request::TxRequest
};

struct PrivateKernelCircuitPublicInputsComposerBuilder {
    tx_request: TxRequest,
    previous_kernel: FixtureBuilder,
    private_call: FixtureBuilder,
}

impl PrivateKernelCircuitPublicInputsComposerBuilder {
    pub fn new() -> Self {
        let previous_kernel = FixtureBuilder::new_from_counter(17);

        let mut private_call = FixtureBuilder::new_from_counter(203);
        // Add an offset to the mock values so that the data in the private call won't be the same as those in the previous kernel.
        private_call.value_offset = 9999;

        let tx_request = private_call.build_tx_request();
        PrivateKernelCircuitPublicInputsComposerBuilder { tx_request, previous_kernel, private_call }
    }

    pub fn new_from_tx_request(self) -> PrivateKernelCircuitPublicInputsComposer {
        let private_call = self.private_call.to_private_circuit_public_inputs();
        PrivateKernelCircuitPublicInputsComposer::new_from_tx_request(self.tx_request, private_call, FixtureBuilder::vk_tree_root())
    }

    pub fn new_from_previous_kernel(self) -> PrivateKernelCircuitPublicInputsComposer {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel)
    }

    pub fn compose_from_tx_request(self) -> PrivateKernelCircuitPublicInputs {
        let private_call = self.private_call.to_private_call_data();
        self.new_from_tx_request().with_private_call(
            private_call.call_stack_item.public_inputs,
            private_call.call_stack_item.contract_address
        ).finish()
    }

    pub fn compose_from_previous_kernel(self) -> PrivateKernelCircuitPublicInputs {
        // Append one private call request for the previous kernel.
        let mut previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        let num_private_call_requests = self.previous_kernel.private_call_requests.len();
        previous_kernel.end.private_call_stack[num_private_call_requests] = PrivateCallRequest::empty();
        previous_kernel.end.private_call_stack[num_private_call_requests].args_hash = 98765432;

        let private_call = self.private_call.to_private_call_data();

        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel).pop_top_call_request().with_private_call(
            private_call.call_stack_item.public_inputs,
            private_call.call_stack_item.contract_address
        ).finish()
    }
}
