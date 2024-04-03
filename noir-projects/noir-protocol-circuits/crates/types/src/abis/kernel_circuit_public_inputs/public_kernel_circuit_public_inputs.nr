use crate::abis::{
    accumulated_data::PublicAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::{RollupValidationRequests, ValidationRequests}
};
use crate::mocked::AggregationObject;

struct PublicKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    validation_requests: ValidationRequests,
    end_non_revertible: PublicAccumulatedData,
    end: PublicAccumulatedData,
    constants: CombinedConstantData,
    revert_code: u8,
}

impl PublicKernelCircuitPublicInputs {
    pub fn needs_setup(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        // So if we have more than one element, we need setup.
        !self.end_non_revertible.public_call_stack[1].is_empty()
    }

    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.end.public_call_stack[0].is_empty()
    }

    pub fn needs_teardown(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        !self.end_non_revertible.public_call_stack[0].is_empty()
    }
}
