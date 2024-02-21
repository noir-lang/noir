use crate::abis::{
    accumulated_data::{PublicAccumulatedNonRevertibleData, PublicAccumulatedRevertibleData},
    combined_constant_data::CombinedConstantData
};

use crate::mocked::AggregationObject;

struct PublicKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: PublicAccumulatedNonRevertibleData,
    end: PublicAccumulatedRevertibleData,
    constants: CombinedConstantData,
    needs_setup: bool,
    needs_app_logic: bool,
    needs_teardown: bool,
}
