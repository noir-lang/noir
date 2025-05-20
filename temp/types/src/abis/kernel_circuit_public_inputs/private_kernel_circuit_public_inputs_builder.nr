use crate::{
    abis::{
        accumulated_data::PrivateAccumulatedDataBuilder,
        kernel_circuit_public_inputs::private_kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        public_call_request::PublicCallRequest, tx_constant_data::TxConstantData,
        validation_requests::PrivateValidationRequestsBuilder,
    },
    address::AztecAddress,
    traits::Empty,
};

pub struct PrivateKernelCircuitPublicInputsBuilder {
    pub constants: TxConstantData,
    pub min_revertible_side_effect_counter: u32,
    pub validation_requests: PrivateValidationRequestsBuilder,
    pub end: PrivateAccumulatedDataBuilder,
    pub public_teardown_call_request: PublicCallRequest,
    pub fee_payer: AztecAddress,
    pub is_private_only: bool,
    pub claimed_first_nullifier: Field,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs {
            constants: self.constants,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            validation_requests: self.validation_requests.finish(),
            end: self.end.finish(),
            public_teardown_call_request: self.public_teardown_call_request,
            fee_payer: self.fee_payer,
            is_private_only: self.is_private_only,
            claimed_first_nullifier: self.claimed_first_nullifier,
        }
    }
}

impl Empty for PrivateKernelCircuitPublicInputsBuilder {
    fn empty() -> Self {
        PrivateKernelCircuitPublicInputsBuilder {
            constants: TxConstantData::empty(),
            min_revertible_side_effect_counter: 0,
            validation_requests: PrivateValidationRequestsBuilder::empty(),
            end: PrivateAccumulatedDataBuilder::empty(),
            public_teardown_call_request: PublicCallRequest::empty(),
            fee_payer: AztecAddress::empty(),
            is_private_only: false,
            claimed_first_nullifier: 0,
        }
    }
}
