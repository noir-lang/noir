use crate::{
    abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::RollupValidationRequests, gas::Gas
},
    partial_state_reference::PartialStateReference, traits::Empty
};
use crate::mocked::AggregationObject;

struct KernelCircuitPublicInputs {
    aggregation_object: AggregationObject,
    rollup_validation_requests: RollupValidationRequests,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    start_state: PartialStateReference,
    revert_code: u8,
}

impl KernelCircuitPublicInputs {

    pub fn compute_transaction_fee(self) -> Field {
        let gas_fees = self.constants.global_variables.gas_fees;
        let inclusion_fee = self.constants.tx_context.gas_settings.inclusion_fee;
        inclusion_fee + self.end.gas_used.compute_fee(gas_fees)
    }
}

impl Empty for KernelCircuitPublicInputs {
    fn empty() -> Self {
        KernelCircuitPublicInputs {
            aggregation_object: AggregationObject::empty(),
            rollup_validation_requests: RollupValidationRequests::empty(),
            end: CombinedAccumulatedData::empty(),
            constants: CombinedConstantData::empty(),
            start_state: PartialStateReference::empty(),
            revert_code: 0,
        }
    }
}

mod tests {
    use crate::abis::{
        kernel_circuit_public_inputs::kernel_circuit_public_inputs::KernelCircuitPublicInputs,
        accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
        validation_requests::RollupValidationRequests, gas::Gas, gas_fees::GasFees
    };
    use crate::partial_state_reference::PartialStateReference;
    use crate::mocked::AggregationObject;

    #[test]
    unconstrained fn empty_has_zero_gas_and_fee() {
        let empty = KernelCircuitPublicInputs::empty();
        assert_eq(empty.compute_transaction_fee(), 0);
    }

    #[test]
    unconstrained fn non_empty_gas_and_fee() {
        let mut inputs = KernelCircuitPublicInputs {
            aggregation_object: AggregationObject::empty(),
            rollup_validation_requests: RollupValidationRequests::empty(),
            end: CombinedAccumulatedData::empty(),
            constants: CombinedConstantData::empty(),
            start_state: PartialStateReference::empty(),
            revert_code: 0
        };

        inputs.end.gas_used = Gas {
            da_gas: 10,
            l2_gas: 20
        };

        inputs.constants.tx_context.gas_settings.inclusion_fee = 42;

        inputs.constants.global_variables.gas_fees = GasFees {
            fee_per_da_gas: 2,
            fee_per_l2_gas: 3
        };

        let transaction_fee = inputs.compute_transaction_fee();
        assert_eq(transaction_fee, 42 + 2 * 10 + 3 * 20);
    }
}
