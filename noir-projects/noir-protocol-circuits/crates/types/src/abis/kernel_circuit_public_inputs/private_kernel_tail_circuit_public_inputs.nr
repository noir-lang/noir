use crate::abis::{
    accumulated_data::{PrivateAccumulatedNonRevertibleData, PrivateAccumulatedRevertibleData},
    combined_constant_data::CombinedConstantData
};

use crate::mocked::AggregationObject;

struct PrivateKernelTailCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: PrivateAccumulatedNonRevertibleData,
    end: PrivateAccumulatedRevertibleData,
    constants: CombinedConstantData,
    needs_setup: bool,
    needs_app_logic: bool,
    needs_teardown: bool,
}
