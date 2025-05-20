use crate::{
    abis::{
        accumulated_data::PrivateAccumulatedData, public_call_request::PublicCallRequest,
        tx_constant_data::TxConstantData, validation_requests::PrivateValidationRequests,
    },
    address::AztecAddress,
    constants::PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_length, reader::Reader},
};

pub struct PrivateKernelCircuitPublicInputsArrayLengths {
    pub note_hash_read_requests: u32,
    pub nullifier_read_requests: u32,
    pub scoped_key_validation_requests_and_generators: u32,
    pub note_hashes: u32,
    pub nullifiers: u32,
    pub l2_to_l1_msgs: u32,
    pub private_logs: u32,
    pub contract_class_logs_hashes: u32,
    pub public_call_requests: u32,
    pub private_call_stack: u32,
}

impl PrivateKernelCircuitPublicInputsArrayLengths {
    pub fn new(public_inputs: PrivateKernelCircuitPublicInputs) -> Self {
        PrivateKernelCircuitPublicInputsArrayLengths {
            note_hash_read_requests: array_length(
                public_inputs.validation_requests.note_hash_read_requests,
            ),
            nullifier_read_requests: array_length(
                public_inputs.validation_requests.nullifier_read_requests,
            ),
            scoped_key_validation_requests_and_generators: array_length(
                public_inputs.validation_requests.scoped_key_validation_requests_and_generators,
            ),
            note_hashes: array_length(public_inputs.end.note_hashes),
            nullifiers: array_length(public_inputs.end.nullifiers),
            l2_to_l1_msgs: array_length(public_inputs.end.l2_to_l1_msgs),
            private_logs: array_length(public_inputs.end.private_logs),
            contract_class_logs_hashes: array_length(public_inputs.end.contract_class_logs_hashes),
            public_call_requests: array_length(public_inputs.end.public_call_requests),
            private_call_stack: array_length(public_inputs.end.private_call_stack),
        }
    }

    pub fn empty() -> Self {
        PrivateKernelCircuitPublicInputsArrayLengths {
            note_hash_read_requests: 0,
            nullifier_read_requests: 0,
            scoped_key_validation_requests_and_generators: 0,
            note_hashes: 0,
            nullifiers: 0,
            l2_to_l1_msgs: 0,
            private_logs: 0,
            contract_class_logs_hashes: 0,
            public_call_requests: 0,
            private_call_stack: 0,
        }
    }
}

impl Eq for PrivateKernelCircuitPublicInputsArrayLengths {
    fn eq(self, other: Self) -> bool {
        (self.note_hash_read_requests == other.note_hash_read_requests)
            & (self.nullifier_read_requests == other.nullifier_read_requests)
            & (
                self.scoped_key_validation_requests_and_generators
                    == other.scoped_key_validation_requests_and_generators
            )
            & (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
            & (self.private_logs == other.private_logs)
            & (self.contract_class_logs_hashes == other.contract_class_logs_hashes)
            & (self.public_call_requests == other.public_call_requests)
            & (self.private_call_stack == other.private_call_stack)
    }
}

pub struct PrivateKernelCircuitPublicInputs {
    pub constants: TxConstantData,
    pub min_revertible_side_effect_counter: u32,
    pub validation_requests: PrivateValidationRequests,
    pub end: PrivateAccumulatedData,
    pub public_teardown_call_request: PublicCallRequest,
    pub fee_payer: AztecAddress,
    pub is_private_only: bool,
    pub claimed_first_nullifier: Field,
}

impl Serialize<PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateKernelCircuitPublicInputs {
    fn serialize(self) -> [Field; PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> =
            BoundedVec::new();

        fields.extend_from_array(self.constants.serialize());
        fields.push(self.min_revertible_side_effect_counter as Field);
        fields.extend_from_array(self.validation_requests.serialize());
        fields.extend_from_array(self.end.serialize());
        fields.extend_from_array(self.public_teardown_call_request.serialize());
        fields.extend_from_array(self.fee_payer.serialize());
        fields.push(self.is_private_only as Field);
        fields.push(self.claimed_first_nullifier);

        assert_eq(fields.len(), PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateKernelCircuitPublicInputs {
    fn deserialize(
        fields: [Field; PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH],
    ) -> PrivateKernelCircuitPublicInputs {
        let mut reader = Reader::new(fields);
        let item = Self {
            constants: reader.read_struct(TxConstantData::deserialize),
            min_revertible_side_effect_counter: reader.read() as u32,
            validation_requests: reader.read_struct(PrivateValidationRequests::deserialize),
            end: reader.read_struct(PrivateAccumulatedData::deserialize),
            public_teardown_call_request: reader.read_struct(PublicCallRequest::deserialize),
            fee_payer: reader.read_struct(AztecAddress::deserialize),
            is_private_only: reader.read_bool(),
            claimed_first_nullifier: reader.read(),
        };

        reader.finish();
        item
    }
}

impl Eq for PrivateKernelCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.constants.eq(other.constants))
            & (self.min_revertible_side_effect_counter == other.min_revertible_side_effect_counter)
            & (self.validation_requests.eq(other.validation_requests))
            & (self.end.eq(other.end))
            & (self.public_teardown_call_request.eq(other.public_teardown_call_request))
            & (self.fee_payer.eq(other.fee_payer))
            & (self.is_private_only == other.is_private_only)
            & (self.claimed_first_nullifier == other.claimed_first_nullifier)
    }
}

impl Empty for PrivateKernelCircuitPublicInputs {
    fn empty() -> Self {
        Self {
            constants: TxConstantData::empty(),
            min_revertible_side_effect_counter: 0,
            validation_requests: PrivateValidationRequests::empty(),
            end: PrivateAccumulatedData::empty(),
            public_teardown_call_request: PublicCallRequest::empty(),
            fee_payer: AztecAddress::empty(),
            is_private_only: false,
            claimed_first_nullifier: 0,
        }
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateKernelCircuitPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = PrivateKernelCircuitPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
