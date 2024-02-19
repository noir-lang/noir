use crate::abis::{
    combined_accumulated_data::{
    CombinedAccumulatedData, PrivateAccumulatedRevertibleData, PrivateAccumulatedNonRevertibleData,
    CombinedAccumulatedDataBuilder, PublicAccumulatedNonRevertibleData, PublicAccumulatedRevertibleData,
    AccumulatedNonRevertibleDataBuilder, AccumulatedRevertibleDataBuilder
},
    combined_constant_data::CombinedConstantData
};
use dep::std::{unsafe};
use crate::constants::{MAX_NEW_COMMITMENTS_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX};

use crate::mocked::AggregationObject;

struct PrivateKernelInnerCircuitPublicInputs {
    aggregation_object: AggregationObject,
    min_revertible_side_effect_counter: u32,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool, // TODO can we remove this?
}

struct PrivateKernelTailCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: PrivateAccumulatedNonRevertibleData,
    end: PrivateAccumulatedRevertibleData,
    constants: CombinedConstantData,
    needs_setup: bool,
    needs_app_logic: bool,
    needs_teardown: bool,
}

struct PublicKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end_non_revertible: PublicAccumulatedNonRevertibleData,
    end: PublicAccumulatedRevertibleData,
    constants: CombinedConstantData,
    needs_setup: bool,
    needs_app_logic: bool,
    needs_teardown: bool,
}

struct RollupKernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
}

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
