use crate::{
    abis::{
    accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::RollupValidationRequests, gas::Gas, gas_fees::GasFees
},
    address::AztecAddress, partial_state_reference::PartialStateReference,
    traits::{Empty, Eq, Serialize, Deserialize}, constants::KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    utils::reader::Reader
};

struct KernelCircuitPublicInputs {
    rollup_validation_requests: RollupValidationRequests,
    end: CombinedAccumulatedData,
    constants: CombinedConstantData,
    start_state: PartialStateReference,
    revert_code: u8,
    fee_payer: AztecAddress
}

impl KernelCircuitPublicInputs {
    pub fn compute_transaction_fee(self, gas_fees: GasFees) -> Field {
        let inclusion_fee = self.constants.tx_context.gas_settings.inclusion_fee;
        inclusion_fee + self.end.gas_used.compute_fee(gas_fees)
    }
}

impl Empty for KernelCircuitPublicInputs {
    fn empty() -> Self {
        KernelCircuitPublicInputs {
            rollup_validation_requests: RollupValidationRequests::empty(),
            end: CombinedAccumulatedData::empty(),
            constants: CombinedConstantData::empty(),
            start_state: PartialStateReference::empty(),
            revert_code: 0,
            fee_payer: AztecAddress::empty()
        }
    }
}

impl Eq for KernelCircuitPublicInputs {
  fn eq(self, other: Self) -> bool {
    (self.rollup_validation_requests.eq(other.rollup_validation_requests)) &
    (self.end.eq(other.end)) &
    (self.constants.eq(other.constants)) &
    (self.start_state.eq(other.start_state)) &
    (self.revert_code == other.revert_code) &
    (self.fee_payer.eq(other.fee_payer))
  }
}

impl Serialize<KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for KernelCircuitPublicInputs {
  fn serialize(self) -> [Field; KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
    let mut fields: BoundedVec<Field, KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.rollup_validation_requests.serialize());
    fields.extend_from_array(self.end.serialize());
    fields.extend_from_array(self.constants.serialize());
    fields.extend_from_array(self.start_state.serialize());
    fields.push(self.revert_code as Field);
    fields.extend_from_array(self.fee_payer.serialize());

    assert_eq(fields.len(), KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for KernelCircuitPublicInputs {
  fn deserialize(fields: [Field; KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> KernelCircuitPublicInputs {
    let mut reader = Reader::new(fields);
    let item = Self {
        rollup_validation_requests: reader.read_struct(RollupValidationRequests::deserialize),
        end: reader.read_struct(CombinedAccumulatedData::deserialize),
        constants: reader.read_struct(CombinedConstantData::deserialize),
        start_state: reader.read_struct(PartialStateReference::deserialize),
        revert_code: reader.read() as u8,
        fee_payer: reader.read_struct(AztecAddress::deserialize),
    };

    reader.finish();
    item
  }
}

mod tests {
    use crate::abis::{
        kernel_circuit_public_inputs::kernel_circuit_public_inputs::KernelCircuitPublicInputs,
        accumulated_data::CombinedAccumulatedData, combined_constant_data::CombinedConstantData,
        validation_requests::RollupValidationRequests, gas::Gas, gas_fees::GasFees
    };
    use crate::address::AztecAddress;
    use crate::partial_state_reference::PartialStateReference;

    #[test]
    unconstrained fn empty_has_zero_gas_and_fee() {
        let empty = KernelCircuitPublicInputs::empty();
        assert_eq(empty.compute_transaction_fee(GasFees::empty()), 0);
    }

    #[test]
    unconstrained fn non_empty_gas_and_fee() {
        let mut inputs = KernelCircuitPublicInputs {
            rollup_validation_requests: RollupValidationRequests::empty(),
            end: CombinedAccumulatedData::empty(),
            constants: CombinedConstantData::empty(),
            start_state: PartialStateReference::empty(),
            revert_code: 0,
            fee_payer: AztecAddress::empty()
        };

        inputs.end.gas_used = Gas {
            da_gas: 10,
            l2_gas: 20
        };

        inputs.constants.tx_context.gas_settings.inclusion_fee = 42;

        let gas_fees = GasFees { fee_per_da_gas: 2, fee_per_l2_gas: 3 };
        let transaction_fee = inputs.compute_transaction_fee(gas_fees);
        assert_eq(transaction_fee, 42 + 2 * 10 + 3 * 20);
    }

    #[test]
    fn serialization_of_empty() {
        let item = KernelCircuitPublicInputs::empty();
        let serialized = item.serialize();
        let deserialized = KernelCircuitPublicInputs::deserialize(serialized);
        assert(item.eq(deserialized));
    }
}
