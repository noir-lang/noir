use crate::abis::{
    accumulated_data::CombinedAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::rollup_kernel_circuit_public_inputs::RollupKernelCircuitPublicInputs
};

use crate::mocked::AggregationObject;

struct RollupKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
}

impl RollupKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> RollupKernelCircuitPublicInputs {
        RollupKernelCircuitPublicInputs { aggregation_object: self.aggregation_object, end: self.end.finish(), constants: self.constants }
    }
}
