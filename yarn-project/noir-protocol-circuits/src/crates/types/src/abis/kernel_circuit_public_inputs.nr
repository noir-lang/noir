use crate::abis::{
    combined_accumulated_data::{
    CombinedAccumulatedData, FinalAccumulatedData, AccumulatedNonRevertibleData,
    CombinedAccumulatedDataBuilder
},
    combined_constant_data::CombinedConstantData
};
use crate::mocked::AggregationObject;

struct PrivateKernelInnerCircuitPublicInputs {
    aggregation_object: AggregationObject,
    max_non_revertible_side_effect_counter: u32,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool,
}

struct PrivateKernelTailCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: AccumulatedNonRevertibleData,
    end: FinalAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool,
}

struct PublicKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: AccumulatedNonRevertibleData,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool,
}

struct PrivateKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    max_non_revertible_side_effect_counter: u32,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
    is_private: bool,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn to_inner(self) -> PrivateKernelInnerCircuitPublicInputs {
        PrivateKernelInnerCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            max_non_revertible_side_effect_counter: self.max_non_revertible_side_effect_counter,
            end: self.end.finish(),
            constants: self.constants,
            is_private: self.is_private
        }
    }

    pub fn to_tail(
        self,
        end_non_revertible: AccumulatedNonRevertibleData
    ) -> PrivateKernelTailCircuitPublicInputs {
        PrivateKernelTailCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            end_non_revertible,
            end: self.end.to_final(),
            constants: self.constants,
            is_private: self.is_private
        }
    }
}

struct PublicKernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    end_non_revertible: AccumulatedNonRevertibleData,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
    is_private: bool,
}

impl PublicKernelCircuitPublicInputsBuilder {
    pub fn to_inner(self) -> PublicKernelCircuitPublicInputs {
        PublicKernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            end_non_revertible: self.end_non_revertible,
            end: self.end.finish(),
            constants: self.constants,
            is_private: self.is_private
        }
    }
}

