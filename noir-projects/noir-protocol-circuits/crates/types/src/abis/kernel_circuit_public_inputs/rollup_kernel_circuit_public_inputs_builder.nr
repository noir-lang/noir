use crate::abis::{
    accumulated_data::CombinedAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::rollup_kernel_circuit_public_inputs::RollupKernelCircuitPublicInputs,
    validation_requests::RollupValidationRequests
};

use crate::mocked::AggregationObject;

struct RollupKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
    rollup_validation_requests: RollupValidationRequests,
}

impl RollupKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> RollupKernelCircuitPublicInputs {
        RollupKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            end: self.end.finish(),
            constants: self.constants,
            rollup_validation_requests: self.rollup_validation_requests
        }
    }
}
