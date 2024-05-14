use crate::abis::{
    accumulated_data::PrivateAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests, call_request::CallRequest
};
use crate::constants::PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH;
use crate::traits::{Serialize, Deserialize, Eq, Empty};
use crate::utils::reader::Reader;
use crate::address::AztecAddress;

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
