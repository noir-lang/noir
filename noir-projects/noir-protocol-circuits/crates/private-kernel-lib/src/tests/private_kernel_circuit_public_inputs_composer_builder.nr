mod new_from_previous_kernel_with_private_call;
mod new_from_tx_request;
mod propagate_from_private_call;

use crate::components::private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    private_call_request::ScopedPrivateCallRequest
},
    constants::MAX_NEW_NOTE_HASHES_PER_CALL, tests::fixture_builder::FixtureBuilder,
    transaction::tx_request::TxRequest
};

struct PrivateKernelCircuitPublicInputsComposerBuilder {
    tx_request: TxRequest,
    previous_kernel: FixtureBuilder,
    private_call: FixtureBuilder,
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
}

impl PrivateKernelCircuitPublicInputsComposerBuilder {
    pub fn new() -> Self {
        let previous_kernel = FixtureBuilder::new_from_counter(17);

        let mut private_call = FixtureBuilder::new_from_counter(203);
        // Add an offset to the mock values so that the data in the private call won't be the same as those in the previous kernel.
        private_call.value_offset = 9999;

        let tx_request = private_call.build_tx_request();
        let note_hash_nullifier_counters = [0; MAX_NEW_NOTE_HASHES_PER_CALL];
        PrivateKernelCircuitPublicInputsComposerBuilder { tx_request, previous_kernel, private_call, note_hash_nullifier_counters }
    }

    pub fn new_from_tx_request(self) -> PrivateKernelCircuitPublicInputsComposer {
        let private_call = self.private_call.to_private_circuit_public_inputs();
        PrivateKernelCircuitPublicInputsComposer::new_from_tx_request(self.tx_request, private_call)
    }

    pub fn new_from_previous_kernel(self) -> PrivateKernelCircuitPublicInputsComposer {
        let mut previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        // Append one private call request for the current call.
        let num_private_call_requests = self.previous_kernel.private_call_requests.len();
        previous_kernel.end.private_call_stack[num_private_call_requests] = ScopedPrivateCallRequest::empty();
        previous_kernel.end.private_call_stack[num_private_call_requests].call_request.hash = 98765432;
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel)
    }

    pub fn compose_from_tx_request(self) -> PrivateKernelCircuitPublicInputs {
        let private_call = self.private_call.to_private_call_data();
        self.new_from_tx_request().compose(
            private_call.call_stack_item.public_inputs,
            private_call.call_stack_item.contract_address,
            self.note_hash_nullifier_counters,
            private_call.public_call_stack,
            private_call.public_teardown_call_request
        ).finish()
    }

    pub fn compose_from_previous_kernel(self) -> PrivateKernelCircuitPublicInputs {
        let private_call = self.private_call.to_private_call_data();
        self.new_from_previous_kernel().compose(
            private_call.call_stack_item.public_inputs,
            private_call.call_stack_item.contract_address,
            self.note_hash_nullifier_counters,
            private_call.public_call_stack,
            private_call.public_teardown_call_request
        ).finish()
    }
}
