mod propagate_accumulated_data;

use crate::{
    components::public_kernel_output_composer::propagate_accumulated_data::propagate_accumulated_data,
    public_kernel_phase::PublicKernelPhase
};
use dep::types::{
    abis::{
    accumulated_data::PublicAccumulatedDataBuilder,
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    public_call_data::PublicCallData, public_call_request::PublicCallRequest,
    public_circuit_public_inputs::PublicCircuitPublicInputs, public_data_read::PublicDataRead
},
    traits::is_empty, utils::arrays::array_to_bounded_vec
};

struct PublicKernelOutputComposer {
    output_builder: PublicKernelCircuitPublicInputsBuilder,
}

impl PublicKernelOutputComposer {
    pub fn new_from_previous_kernel(previous_kernel: PublicKernelCircuitPublicInputs) -> Self {
        let output_builder = PublicKernelCircuitPublicInputsBuilder::new(previous_kernel);
        PublicKernelOutputComposer { output_builder }
    }

    pub fn remove_top_call_request(&mut self, phase: u8) -> Self {
        if phase == PublicKernelPhase.SETUP {
            let _ = self.output_builder.end_non_revertible.public_call_stack.pop();
        }
        if phase == PublicKernelPhase.APP_LOGIC {
            let _ = self.output_builder.end.public_call_stack.pop();
        }
        if phase == PublicKernelPhase.TEARDOWN {
            let _ = self.output_builder.public_teardown_call_stack.pop();
        }
        *self
    }

    pub fn propagate_from_public_call(&mut self, public_call: PublicCircuitPublicInputs, phase: u8) -> Self {
        self.output_builder.constants.global_variables = public_call.global_variables;
        self.propagate_revert_code(public_call, phase);
        self.propagate_validation_requests(public_call);
        self.propagate_accumulated_data(public_call, phase);
        self.update_gas_used(public_call, phase);
        *self
    }

    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        self.output_builder.finish()
    }

    fn propagate_revert_code(&mut self, public_call: PublicCircuitPublicInputs, phase: u8) {
        let prev_revert_code = self.output_builder.revert_code;
        let public_call_revert_code = public_call.revert_code;
        if phase == PublicKernelPhase.SETUP {
            // Keep prev_revert_code.
            // public_call_revert_code should be 0. If not, PublicCallDataValidator > validate_revert_code will fail.
        }
        if phase == PublicKernelPhase.APP_LOGIC {
            if prev_revert_code == 0 {
                self.output_builder.revert_code = public_call_revert_code;
            }
        }
        if phase == PublicKernelPhase.TEARDOWN {
            // See https://docs.aztec.network/protocol-specs/gas-and-fees/kernel-tracking#handling-reverts
            if public_call_revert_code != 0 {
                self.output_builder.revert_code = if prev_revert_code == 0 {
                    // Case where there is a new error in teardown
                    2
                } else {
                    // Case where there is an error in both app logic and teardown
                    3
                };
            }
        }
    }

    fn propagate_validation_requests(&mut self, public_call: PublicCircuitPublicInputs) {
        // Note that the public kernel cannot modify the max block number value - it simply forwards it to the rollup

        let storage_contract_address = public_call.call_context.storage_contract_address;

        let nullifier_read_requests = public_call.nullifier_read_requests;
        for i in 0..nullifier_read_requests.len() {
            let request = nullifier_read_requests[i];
            if !is_empty(request) {
                self.output_builder.validation_requests.nullifier_read_requests.push(request.scope(storage_contract_address));
            }
        }

        let nullifier_non_existent_read_requests = public_call.nullifier_non_existent_read_requests;
        for i in 0..nullifier_non_existent_read_requests.len() {
            let request = nullifier_non_existent_read_requests[i];
            if !is_empty(request) {
                self.output_builder.validation_requests.nullifier_non_existent_read_requests.push(request.scope(storage_contract_address));
            }
        }

        let read_requests = public_call.contract_storage_reads;
        for i in 0..read_requests.len() {
            let read_request = read_requests[i];
            if !is_empty(read_request) {
                self.output_builder.validation_requests.public_data_reads.push(PublicDataRead::from_contract_storage_read(storage_contract_address, read_request));
            }
        }
    }

    fn propagate_accumulated_data(&mut self, public_call: PublicCircuitPublicInputs, phase: u8) {
        let revert_in_phase = (self.output_builder.revert_code != 0)
            & ((phase != PublicKernelPhase.TEARDOWN) | (self.output_builder.revert_code != 1)); // Revert in APP_LOGIC

        if phase == PublicKernelPhase.SETUP {
            self.output_builder.end_non_revertible = propagate_accumulated_data(&mut self.output_builder.end_non_revertible, public_call);
        }
        if (phase == PublicKernelPhase.APP_LOGIC) | (phase == PublicKernelPhase.TEARDOWN) {
            self.output_builder.end = propagate_accumulated_data(&mut self.output_builder.end, public_call);
        }

        // TODO: Should keep the data even when reverts. 
        // The data is required for verifying validation requests in the tail circuit, which will then discard the 
        // revertible data.
        if revert_in_phase {
            self.output_builder.end = PublicAccumulatedDataBuilder::empty();
            if phase == PublicKernelPhase.TEARDOWN {
                self.output_builder.public_teardown_call_stack = BoundedVec::new();
            }
        }
        if !revert_in_phase {
            self.propagate_call_requests(public_call, phase);
        }
    }

    fn propagate_call_requests(&mut self, public_call: PublicCircuitPublicInputs, phase: u8) {
        let call_requests = array_to_bounded_vec(public_call.public_call_requests);

        // TODO: Insert call requests in reversed order.
        if phase == PublicKernelPhase.SETUP {
            self.output_builder.end_non_revertible.public_call_stack.extend_from_bounded_vec(call_requests);
        }
        if phase == PublicKernelPhase.APP_LOGIC {
            self.output_builder.end.public_call_stack.extend_from_bounded_vec(call_requests);
        }
        if phase == PublicKernelPhase.TEARDOWN {
            self.output_builder.public_teardown_call_stack.extend_from_bounded_vec(call_requests);
        }
    }

    fn update_gas_used(&mut self, public_call: PublicCircuitPublicInputs, phase: u8) {
        let tx_gas_limits = self.output_builder.constants.tx_context.gas_settings.gas_limits;
        let call_gas_left = public_call.end_gas_left;
        if phase == PublicKernelPhase.SETUP {
            let accum_end_gas_used = self.output_builder.end.gas_used;
            self.output_builder.end_non_revertible.gas_used = tx_gas_limits
                .sub(call_gas_left)
                .sub(accum_end_gas_used);
        }

        if phase == PublicKernelPhase.APP_LOGIC {
            let accum_end_non_revertible_gas_used = self.output_builder.end_non_revertible.gas_used;
            self.output_builder.end.gas_used = tx_gas_limits
                .sub(call_gas_left)
                .sub(accum_end_non_revertible_gas_used);
        }
    }
}
