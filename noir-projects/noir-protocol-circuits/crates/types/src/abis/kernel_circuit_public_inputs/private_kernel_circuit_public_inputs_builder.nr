use crate::{
    abis::{
    accumulated_data::PrivateAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{
    kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    private_kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs
},
    gas::Gas, validation_requests::validation_requests_builder::ValidationRequestsBuilder,
    call_request::CallRequest
},
    address::AztecAddress, constants::MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    partial_state_reference::PartialStateReference, traits::{Empty, is_empty}
};

// Builds:
// .finish: PrivateKernelCircuitPublicInputs
// .finish_tail: KernelCircuitPublicInputs (from KernelCircuitPublicInputsComposer)
// .finish_to_public: PublicKernelCircuitPublicInputs (from KernelCircuitPublicInputsComposer)
struct PrivateKernelCircuitPublicInputsBuilder {
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequestsBuilder,
    end: PrivateAccumulatedDataBuilder,
    constants: CombinedConstantData,
    public_teardown_call_request: CallRequest,
    fee_payer: AztecAddress
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs {
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            validation_requests: self.validation_requests.finish(),
            end: self.end.finish(),
            constants: self.constants,
            public_teardown_call_request: self.public_teardown_call_request,
            fee_payer: self.fee_payer
        }
    }

    pub fn finish_tail(self, teardown_gas: Gas) -> KernelCircuitPublicInputs {
        KernelCircuitPublicInputs {
            rollup_validation_requests: self.validation_requests.to_rollup(),
            end: self.end.to_combined(teardown_gas),
            constants: self.constants,
            start_state: PartialStateReference::empty(),
            revert_code: 0,
            fee_payer: self.fee_payer
        }
    }

    pub fn finish_to_public(
        self,
        teardown_gas: Gas,
        min_revertible_side_effect_counter: u32
    ) -> PublicKernelCircuitPublicInputs {
        let (end_non_revertible, end) = self.end.split_to_public(min_revertible_side_effect_counter, teardown_gas);
        let mut public_teardown_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX> = BoundedVec::new();
        if (!is_empty(self.public_teardown_call_request)) {
            public_teardown_call_stack.push(self.public_teardown_call_request);
        }

        PublicKernelCircuitPublicInputs {
            validation_requests: self.validation_requests.finish(),
            end_non_revertible,
            end,
            constants: self.constants,
            revert_code: 0,
            public_teardown_call_stack: public_teardown_call_stack.storage,
            fee_payer: self.fee_payer
        }
    }
}

impl Empty for PrivateKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateKernelCircuitPublicInputsBuilder {
            min_revertible_side_effect_counter: 0 as u32,
            validation_requests: ValidationRequestsBuilder::empty(),
            end: PrivateAccumulatedDataBuilder::empty(),
            constants: CombinedConstantData::empty(),
            public_teardown_call_request: CallRequest::empty(),
            fee_payer: AztecAddress::empty()
        }
    }
}
