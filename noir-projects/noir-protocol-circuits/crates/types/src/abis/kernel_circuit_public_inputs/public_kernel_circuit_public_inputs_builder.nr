use crate::{
    abis::{
    accumulated_data::{CombinedAccumulatedData, PublicAccumulatedDataBuilder},
    combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs},
    public_call_request::PublicCallRequest, validation_requests::ValidationRequestsBuilder
},
    address::AztecAddress, constants::MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, traits::Empty
};

struct PublicKernelCircuitPublicInputsBuilder {
    validation_requests: ValidationRequestsBuilder,
    end_non_revertible: PublicAccumulatedDataBuilder,
    end: PublicAccumulatedDataBuilder,
    constants: CombinedConstantData,
    revert_code: u8,
    public_teardown_call_stack: BoundedVec<PublicCallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    fee_payer: AztecAddress,
}

impl PublicKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        PublicKernelCircuitPublicInputs {
            // Note that we're including both the validation_requests AND the rollup_validation requests, because this
            // struct is used as an input for both the public kernel and base rollup circuits. In the near future the
            // base rollup will only receive rollup_validation_requests, and the public kernel only validation_requests.
            validation_requests: self.validation_requests.finish(),
            end_non_revertible: self.end_non_revertible.finish(),
            end: self.end.finish(),
            constants: self.constants,
            revert_code: self.revert_code,
            public_teardown_call_stack: self.public_teardown_call_stack.storage,
            fee_payer: self.fee_payer
        }
    }
}

impl Empty for PublicKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PublicKernelCircuitPublicInputsBuilder {
            validation_requests: ValidationRequestsBuilder::empty(),
            end_non_revertible: PublicAccumulatedDataBuilder::empty(),
            end: PublicAccumulatedDataBuilder::empty(),
            constants: CombinedConstantData::empty(),
            revert_code: 0 as u8,
            public_teardown_call_stack: BoundedVec::new(),
            fee_payer: AztecAddress::empty(),
        }
    }
}
