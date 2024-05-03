use crate::{
    abis::{
    accumulated_data::{CombinedAccumulatedData, PublicAccumulatedDataBuilder},
    combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs},
    validation_requests::ValidationRequestsBuilder, call_request::CallRequest
},
    mocked::AggregationObject, traits::Empty
};

struct PublicKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    validation_requests: ValidationRequestsBuilder,
    end_non_revertible: PublicAccumulatedDataBuilder,
    end: PublicAccumulatedDataBuilder,
    constants: CombinedConstantData,
    revert_code: u8,
    public_teardown_call_request: CallRequest,
}

impl PublicKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        PublicKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            // Note that we're including both the validation_requests AND the rollup_validation requests, because this
            // struct is used as an input for both the public kernel and base rollup circuits. In the near future the
            // base rollup will only receive rollup_validation_requests, and the public kernel only validation_requests.
            validation_requests: self.validation_requests.finish(),
            end_non_revertible: self.end_non_revertible.finish(),
            end: self.end.finish(),
            constants: self.constants,
            revert_code: self.revert_code,
            public_teardown_call_request: self.public_teardown_call_request
        }
    }
}

impl Empty for PublicKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PublicKernelCircuitPublicInputsBuilder {
            aggregation_object: AggregationObject::empty(),
            validation_requests: ValidationRequestsBuilder::empty(),
            end_non_revertible: PublicAccumulatedDataBuilder::empty(),
            end: PublicAccumulatedDataBuilder::empty(),
            constants: CombinedConstantData::empty(),
            revert_code: 0 as u8,
            public_teardown_call_request: CallRequest::empty()
        }
    }
}
