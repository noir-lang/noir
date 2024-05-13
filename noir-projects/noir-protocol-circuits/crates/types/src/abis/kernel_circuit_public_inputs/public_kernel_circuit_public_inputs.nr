use crate::abis::{
    accumulated_data::PublicAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::{RollupValidationRequests, ValidationRequests}, call_request::CallRequest
};
use crate::address::AztecAddress;
use crate::constants::MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX;

struct PublicKernelCircuitPublicInputs {
    validation_requests: ValidationRequests,
    end_non_revertible: PublicAccumulatedData,
    end: PublicAccumulatedData,
    constants: CombinedConstantData,
    revert_code: u8,
    public_teardown_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    fee_payer: AztecAddress
}

impl PublicKernelCircuitPublicInputs {
    pub fn needs_setup(self) -> bool {
        // public calls for setup are deposited in the non-revertible public call stack.
        // if an element is present, we need to run setup
        self.end_non_revertible.public_call_stack[0].hash != 0
    }

    pub fn needs_app_logic(self) -> bool {
        // public calls for app logic are deposited in the revertible public call stack.
        // if an element is present, we need to run app logic
        self.end.public_call_stack[0].hash != 0
    }

    pub fn needs_teardown(self) -> bool {
        // the public call specified for teardown, if any, is placed in the teardown call stack
        self.public_teardown_call_stack[0].hash != 0
    }
}
