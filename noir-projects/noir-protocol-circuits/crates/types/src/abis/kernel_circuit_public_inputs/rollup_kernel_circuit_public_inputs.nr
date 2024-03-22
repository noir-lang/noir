use crate::abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::RollupValidationRequests
};

use crate::mocked::AggregationObject;

struct RollupKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    rollup_validation_requests: RollupValidationRequests,
}
