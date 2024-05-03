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
    mocked::AggregationObject, partial_state_reference::PartialStateReference, traits::Empty
};

// Builds:
// .finish: PrivateKernelCircuitPublicInputs
// .finish_tail: KernelCircuitPublicInputs (from KernelCircuitPublicInputsComposer)
// .finish_to_public: PublicKernelCircuitPublicInputs (from KernelCircuitPublicInputsComposer)
struct PrivateKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequestsBuilder,
    end: PrivateAccumulatedDataBuilder,
    constants: CombinedConstantData,
    public_teardown_call_request: CallRequest,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            validation_requests: self.validation_requests.finish(),
            end: self.end.finish(),
            constants: self.constants,
            public_teardown_call_request: self.public_teardown_call_request
        }
    }

    pub fn finish_tail(self, teardown_gas: Gas) -> KernelCircuitPublicInputs {
        KernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            rollup_validation_requests: self.validation_requests.to_rollup(),
            end: self.end.to_combined(teardown_gas),
            constants: self.constants,
            start_state: PartialStateReference::empty(),
            revert_code: 0
        }
    }

    pub fn finish_to_public(
        self,
        teardown_gas: Gas,
        min_revertible_side_effect_counter: u32
    ) -> PublicKernelCircuitPublicInputs {
        let (end_non_revertible, end) = self.end.split_to_public(min_revertible_side_effect_counter, teardown_gas);

        PublicKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            validation_requests: self.validation_requests.finish(),
            end_non_revertible,
            end,
            constants: self.constants,
            revert_code: 0,
            public_teardown_call_request: self.public_teardown_call_request
        }
    }
}

impl Empty for PrivateKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateKernelCircuitPublicInputsBuilder {
            aggregation_object: AggregationObject::empty(),
            min_revertible_side_effect_counter: 0 as u32,
            validation_requests: ValidationRequestsBuilder::empty(),
            end: PrivateAccumulatedDataBuilder::empty(),
            constants: CombinedConstantData::empty(),
            public_teardown_call_request: CallRequest::empty()
        }
    }
}
