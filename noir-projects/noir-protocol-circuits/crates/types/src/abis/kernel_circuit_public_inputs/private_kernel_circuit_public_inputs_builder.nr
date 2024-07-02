use crate::{
    abis::{
    accumulated_data::PrivateAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::private_kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    validation_requests::validation_requests_builder::ValidationRequestsBuilder,
    call_request::CallRequest
},
    address::AztecAddress, traits::Empty
};

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
}

impl Empty for PrivateKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateKernelCircuitPublicInputsBuilder {
            min_revertible_side_effect_counter: 0,
            validation_requests: ValidationRequestsBuilder::empty(),
            end: PrivateAccumulatedDataBuilder::empty(),
            constants: CombinedConstantData::empty(),
            public_teardown_call_request: CallRequest::empty(),
            fee_payer: AztecAddress::empty()
        }
    }
}
