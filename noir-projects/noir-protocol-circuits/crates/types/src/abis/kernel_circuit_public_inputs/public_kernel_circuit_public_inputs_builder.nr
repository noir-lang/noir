use crate::abis::{
    accumulated_data::{AccumulatedNonRevertibleDataBuilder, AccumulatedRevertibleDataBuilder},
    combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs
};

use crate::mocked::AggregationObject;

struct PublicKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    end_non_revertible: AccumulatedNonRevertibleDataBuilder,
    end: AccumulatedRevertibleDataBuilder,
    constants: CombinedConstantData,
}

impl PublicKernelCircuitPublicInputsBuilder {
    pub fn to_inner(self) -> PublicKernelCircuitPublicInputs {
        let end_non_revertible = self.end_non_revertible.to_public();
        let end = self.end.to_public();
        PublicKernelCircuitPublicInputs {
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
