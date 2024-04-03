use crate::abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::RollupValidationRequests
};
use crate::mocked::AggregationObject;

struct KernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    rollup_validation_requests: RollupValidationRequests,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    revert_code: u8,
}
