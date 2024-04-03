use crate::abis::{
    accumulated_data::{CombinedAccumulatedData, PublicAccumulatedDataBuilder},
    combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{
    kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    public_kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs
},
    validation_requests::ValidationRequestsBuilder
};
use crate::mocked::AggregationObject;

struct PublicKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    validation_requests: ValidationRequestsBuilder,
    end_non_revertible: PublicAccumulatedDataBuilder,
    end: PublicAccumulatedDataBuilder,
    constants: CombinedConstantData,
    revert_code: u8,
}

impl PublicKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        PublicKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            // Note that we're including both the validation_requests AND the rollup_validation requests, because this
            // struct is used as an input for both the public kernel and base rollup circuits. In the near future the
            // base rollup will only receive rollup_validation_requests, and the public kernel only validation_requests.
            validation_requests: self.validation_requests.finish(),
            end_non_revertible: self.end_non_revertible.finish(),
            end: self.end.finish(),
            constants: self.constants,
            revert_code: self.revert_code
        }
    }

    pub fn finish_tail(self) -> KernelCircuitPublicInputs {
        KernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            rollup_validation_requests: self.validation_requests.to_rollup(),
            // TODO: Sort by counters.
            end: CombinedAccumulatedData::recombine(self.end_non_revertible.finish(), self.end.finish()),
            constants: self.constants,
            revert_code: self.revert_code
        }
    }
}
