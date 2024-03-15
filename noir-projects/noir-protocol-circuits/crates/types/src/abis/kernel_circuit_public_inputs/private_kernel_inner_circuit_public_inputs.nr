use crate::abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests
};
use crate::mocked::AggregationObject;

struct PrivateKernelInnerCircuitPublicInputs {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequests,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool, // TODO can we remove this?
}
