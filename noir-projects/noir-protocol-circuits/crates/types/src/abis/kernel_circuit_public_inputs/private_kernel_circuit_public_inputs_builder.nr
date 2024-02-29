use crate::abis::{
    accumulated_data::CombinedAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{
    private_kernel_inner_circuit_public_inputs::PrivateKernelInnerCircuitPublicInputs,
    private_kernel_tail_circuit_public_inputs::PrivateKernelTailCircuitPublicInputs
}
};

use crate::mocked::AggregationObject;

struct PrivateKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
    is_private: bool,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn to_inner(self) -> PrivateKernelInnerCircuitPublicInputs {
        PrivateKernelInnerCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            end: self.end.finish(),
            constants: self.constants,
            is_private: self.is_private
        }
    }

    pub fn to_tail(self) -> PrivateKernelTailCircuitPublicInputs {
        let (end_non_revertible, end) = self.end.split(self.min_revertible_side_effect_counter);
        PrivateKernelTailCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            end_non_revertible,
            end,
            constants: self.constants,
            needs_setup: end_non_revertible.needs_setup(),
            needs_app_logic: end.needs_app_logic(),
            needs_teardown: end_non_revertible.needs_teardown()
        }
    }
}
