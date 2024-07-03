use crate::abis::{
    accumulated_data::PrivateAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests, call_request::CallRequest
};
use crate::constants::PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH;
use crate::traits::{Serialize, Deserialize, Eq, Empty};
use crate::utils::{arrays::array_length, reader::Reader};
use crate::address::AztecAddress;

struct PrivateKernelCircuitPublicInputsArrayLengths {
    note_hash_read_requests: u32,
    nullifier_read_requests: u32,
    scoped_key_validation_requests_and_generators: u32,
    note_hashes: u32,
    nullifiers: u32,
    l2_to_l1_msgs: u32,
    note_encrypted_logs_hashes: u32,
    encrypted_logs_hashes: u32,
    unencrypted_logs_hashes: u32,
    private_call_stack: u32,
    public_call_stack: u32,
}

impl PrivateKernelCircuitPublicInputsArrayLengths {
    pub fn new(public_inputs: PrivateKernelCircuitPublicInputs) -> Self {
        PrivateKernelCircuitPublicInputsArrayLengths {
            note_hash_read_requests: array_length(public_inputs.validation_requests.note_hash_read_requests),
            nullifier_read_requests: array_length(public_inputs.validation_requests.nullifier_read_requests),
            scoped_key_validation_requests_and_generators: array_length(public_inputs.validation_requests.scoped_key_validation_requests_and_generators),
            note_hashes: array_length(public_inputs.end.note_hashes),
            nullifiers: array_length(public_inputs.end.nullifiers),
            l2_to_l1_msgs: array_length(public_inputs.end.l2_to_l1_msgs),
            note_encrypted_logs_hashes: array_length(public_inputs.end.note_encrypted_logs_hashes),
            encrypted_logs_hashes: array_length(public_inputs.end.encrypted_logs_hashes),
            unencrypted_logs_hashes: array_length(public_inputs.end.unencrypted_logs_hashes),
            private_call_stack: array_length(public_inputs.end.private_call_stack),
            public_call_stack: array_length(public_inputs.end.public_call_stack)
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
            note_encrypted_logs_hashes: 0,
            encrypted_logs_hashes: 0,
            unencrypted_logs_hashes: 0,
            private_call_stack: 0,
            public_call_stack: 0
        }
    }
}

impl Eq for PrivateKernelCircuitPublicInputsArrayLengths {
    fn eq(self, other: Self) -> bool {
        (self.note_hash_read_requests == other.note_hash_read_requests) &
        (self.nullifier_read_requests == other.nullifier_read_requests) &
        (self.scoped_key_validation_requests_and_generators == other.scoped_key_validation_requests_and_generators) &
        (self.note_hashes == other.note_hashes) &
        (self.nullifiers == other.nullifiers) &
        (self.l2_to_l1_msgs == other.l2_to_l1_msgs) &
        (self.note_encrypted_logs_hashes == other.note_encrypted_logs_hashes) &
        (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
        (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
        (self.private_call_stack == other.private_call_stack) &
        (self.public_call_stack== other.public_call_stack)
    }
}

struct PrivateKernelCircuitPublicInputs {
    min_revertible_side_effect_counter: u32,
    validation_requests: ValidationRequests,
    end: PrivateAccumulatedData,
    constants: CombinedConstantData,
    public_teardown_call_request: CallRequest,
    fee_payer: AztecAddress
}

impl Serialize<PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateKernelCircuitPublicInputs {
  fn serialize(self) -> [Field; PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
    let mut fields: BoundedVec<Field, PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();

    fields.push(self.min_revertible_side_effect_counter as Field);
    fields.extend_from_array(self.validation_requests.serialize());
    fields.extend_from_array(self.end.serialize());
    fields.extend_from_array(self.constants.serialize());
    fields.extend_from_array(self.public_teardown_call_request.serialize());
    fields.extend_from_array(self.fee_payer.serialize());

    assert_eq(fields.len(), PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateKernelCircuitPublicInputs {
  fn deserialize(fields: [Field; PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> PrivateKernelCircuitPublicInputs {
    let mut reader = Reader::new(fields);
    let item = Self {
        min_revertible_side_effect_counter: reader.read() as u32,
        validation_requests: reader.read_struct(ValidationRequests::deserialize),
        end: reader.read_struct(PrivateAccumulatedData::deserialize),
        constants: reader.read_struct(CombinedConstantData::deserialize),
        public_teardown_call_request: reader.read_struct(CallRequest::deserialize),
        fee_payer: reader.read_struct(AztecAddress::deserialize),
    };

    reader.finish();
    item
  }
}

impl Eq for PrivateKernelCircuitPublicInputs {
  fn eq(self, other: Self) -> bool {
    (self.min_revertible_side_effect_counter == other.min_revertible_side_effect_counter) &
    (self.validation_requests.eq(other.validation_requests)) &
    (self.end.eq(other.end)) &
    (self.constants.eq(other.constants)) &
    (self.public_teardown_call_request.eq(other.public_teardown_call_request))
  }
}

impl Empty for PrivateKernelCircuitPublicInputs {
  fn empty() -> Self {
    Self {
      min_revertible_side_effect_counter: 0,
      validation_requests: ValidationRequests::empty(),
      end: PrivateAccumulatedData::empty(),
      constants: CombinedConstantData::empty(),
      public_teardown_call_request: CallRequest::empty(),
      fee_payer: AztecAddress::empty()
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
