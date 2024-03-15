use crate::abis::{
    accumulated_data::{PublicAccumulatedNonRevertibleData, PublicAccumulatedRevertibleData},
    combined_constant_data::CombinedConstantData, validation_requests::ValidationRequests
};
use crate::mocked::AggregationObject;

struct PublicKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    validation_requests: ValidationRequests,
    end_non_revertible: PublicAccumulatedNonRevertibleData,
    end: PublicAccumulatedRevertibleData,
    constants: CombinedConstantData,
    needs_setup: bool,
    needs_app_logic: bool,
    needs_teardown: bool,
    reverted: bool,
}
