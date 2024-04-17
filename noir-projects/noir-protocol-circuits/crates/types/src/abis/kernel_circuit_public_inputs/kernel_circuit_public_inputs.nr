use crate::{
    abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::RollupValidationRequests
},
    partial_state_reference::PartialStateReference
};
use crate::mocked::AggregationObject;

struct KernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    rollup_validation_requests: RollupValidationRequests,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    start_state: PartialStateReference,
    revert_code: u8,
}
