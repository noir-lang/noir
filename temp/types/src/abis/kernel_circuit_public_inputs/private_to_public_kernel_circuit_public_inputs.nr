use crate::{
    abis::{
        accumulated_data::PrivateToPublicAccumulatedData, gas::Gas,
        public_call_request::PublicCallRequest, tx_constant_data::TxConstantData,
        validation_requests::RollupValidationRequests,
    },
    address::AztecAddress,
    constants::{
        GENERATOR_INDEX__PUBLIC_TX_HASH, PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    },
    hash::poseidon2_hash_with_separator,
    traits::{Deserialize, Empty, Hash, Serialize},
    utils::reader::Reader,
};

pub struct PrivateToPublicKernelCircuitPublicInputs {
    pub constants: TxConstantData,
    pub rollup_validation_requests: RollupValidationRequests,
    pub non_revertible_accumulated_data: PrivateToPublicAccumulatedData,
    pub revertible_accumulated_data: PrivateToPublicAccumulatedData,
    pub public_teardown_call_request: PublicCallRequest,
    pub gas_used: Gas,
    pub fee_payer: AztecAddress,
}

impl Empty for PrivateToPublicKernelCircuitPublicInputs {
    fn empty() -> Self {
        PrivateToPublicKernelCircuitPublicInputs {
            constants: TxConstantData::empty(),
            rollup_validation_requests: RollupValidationRequests::empty(),
            non_revertible_accumulated_data: PrivateToPublicAccumulatedData::empty(),
            revertible_accumulated_data: PrivateToPublicAccumulatedData::empty(),
            public_teardown_call_request: PublicCallRequest::empty(),
            gas_used: Gas::empty(),
            fee_payer: AztecAddress::empty(),
        }
    }
}

impl Eq for PrivateToPublicKernelCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.constants == other.constants)
            & (self.rollup_validation_requests == other.rollup_validation_requests)
            & (self.non_revertible_accumulated_data == other.non_revertible_accumulated_data)
            & (self.revertible_accumulated_data == other.revertible_accumulated_data)
            & (self.public_teardown_call_request == other.public_teardown_call_request)
            & (self.gas_used == other.gas_used)
            & (self.fee_payer == other.fee_payer)
    }
}

impl Serialize<PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateToPublicKernelCircuitPublicInputs {
    fn serialize(self) -> [Field; PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> =
            BoundedVec::new();

        fields.extend_from_array(self.constants.serialize());
        fields.extend_from_array(self.rollup_validation_requests.serialize());
        fields.extend_from_array(self.non_revertible_accumulated_data.serialize());
        fields.extend_from_array(self.revertible_accumulated_data.serialize());
        fields.extend_from_array(self.public_teardown_call_request.serialize());
        fields.extend_from_array(self.gas_used.serialize());
        fields.extend_from_array(self.fee_payer.serialize());

        assert_eq(fields.len(), PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH);

        fields.storage()
    }
}

impl Hash for PrivateToPublicKernelCircuitPublicInputs {
    fn hash(self) -> Field {
        poseidon2_hash_with_separator(self.serialize(), GENERATOR_INDEX__PUBLIC_TX_HASH)
    }
}

impl Deserialize<PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateToPublicKernelCircuitPublicInputs {
    fn deserialize(
        fields: [Field; PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH],
    ) -> PrivateToPublicKernelCircuitPublicInputs {
        let mut reader = Reader::new(fields);

        let item = PrivateToPublicKernelCircuitPublicInputs {
            constants: reader.read_struct(TxConstantData::deserialize),
            rollup_validation_requests: reader.read_struct(RollupValidationRequests::deserialize),
            non_revertible_accumulated_data: reader.read_struct(
                PrivateToPublicAccumulatedData::deserialize,
            ),
            revertible_accumulated_data: reader.read_struct(
                PrivateToPublicAccumulatedData::deserialize,
            ),
            public_teardown_call_request: reader.read_struct(PublicCallRequest::deserialize),
            gas_used: reader.read_struct(Gas::deserialize),
            fee_payer: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateToPublicKernelCircuitPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = PrivateToPublicKernelCircuitPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
