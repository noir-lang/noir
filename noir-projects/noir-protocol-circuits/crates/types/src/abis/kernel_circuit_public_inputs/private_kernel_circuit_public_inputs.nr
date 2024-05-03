use crate::abis::{
    accumulated_data::PrivateAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests, call_request::CallRequest
};
use crate::mocked::AggregationObject;

struct PrivateKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequests,
    end: PrivateAccumulatedData,
    constants: CombinedConstantData,
    public_teardown_call_request: CallRequest,
}
