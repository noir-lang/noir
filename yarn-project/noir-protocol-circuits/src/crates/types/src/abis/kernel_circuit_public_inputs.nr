use crate::abis::{
    combined_accumulated_data::{
        CombinedAccumulatedData,
        FinalAccumulatedData,
        AccumulatedMetaData,
        AccumulatedMetaDataBuilder,
        CombinedAccumulatedDataBuilder,
    },
    combined_constant_data::CombinedConstantData
};
use crate::mocked::AggregationObject;

struct KernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    meta_hwm: Field,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool,
}

struct KernelCircuitPublicInputsFinal {
    aggregation_object: AggregationObject,
    meta_hwm: Field,
    end_meta: AccumulatedMetaData,
    // TODO(fees) change this to end_app_logic
    end: FinalAccumulatedData,
    constants: CombinedConstantData,
    is_private: bool,
}

struct KernelCircuitPublicInputsBuilder {
    aggregation_object: AggregationObject,
    meta_hwm: Field,
    end_meta: AccumulatedMetaDataBuilder,
    end: CombinedAccumulatedDataBuilder,
    constants: CombinedConstantData,
    is_private: bool,
}

impl KernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> KernelCircuitPublicInputs {
        KernelCircuitPublicInputs {
            aggregation_object: self.aggregation_object,
            meta_hwm: self.meta_hwm,
            end: self.end.finish(),
            constants: self.constants,
            is_private: self.is_private,
        }
    }

    pub fn to_final(self) -> KernelCircuitPublicInputsFinal {
        let meta = self.end_meta.finish();
        let app = self.end.to_final();
        KernelCircuitPublicInputsFinal {
            aggregation_object: self.aggregation_object,
            meta_hwm: self.meta_hwm,
            end_meta: meta,
            end: app,
            constants: self.constants,
            is_private: self.is_private,
        }
    }
}
