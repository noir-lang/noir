use crate::abis::{
    accumulated_data::PrivateAccumulatedDataBuilder, combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{
    kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    private_kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs
},
    validation_requests::validation_requests_builder::ValidationRequestsBuilder
};
use crate::mocked::AggregationObject;
use dep::std::unsafe;

struct PrivateKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequestsBuilder,
    end: PrivateAccumulatedDataBuilder,
    constants: CombinedConstantData,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            validation_requests: self.validation_requests.finish(),
            end: self.end.finish(),
            constants: self.constants
        }
    }

    pub fn finish_tail(self) -> KernelCircuitPublicInputs {
        KernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            rollup_validation_requests: self.validation_requests.to_rollup(),
            end: self.end.to_combined(),
            constants: self.constants,
            revert_code: 0
        }
    }

    pub fn finish_to_public(self, min_revertible_side_effect_counter: u32) -> PublicKernelCircuitPublicInputs {
        let (end_non_revertible, end) = self.end.split_to_public(min_revertible_side_effect_counter);

        PublicKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            validation_requests: self.validation_requests.finish(),
            end_non_revertible,
            end,
            constants: self.constants,
            revert_code: 0
        }
    }
}
