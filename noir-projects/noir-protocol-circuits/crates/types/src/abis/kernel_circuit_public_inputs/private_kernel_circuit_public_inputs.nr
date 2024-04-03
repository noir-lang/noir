use crate::abis::{
    accumulated_data::PrivateAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests
};
use crate::mocked::AggregationObject;

struct PrivateKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequests,
    end: PrivateAccumulatedData,
    constants: CombinedConstantData,
}
