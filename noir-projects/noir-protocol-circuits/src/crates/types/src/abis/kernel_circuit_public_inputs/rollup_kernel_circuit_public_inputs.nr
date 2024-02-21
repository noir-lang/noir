use crate::abis::{accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData};

use crate::mocked::AggregationObject;

struct RollupKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
}
