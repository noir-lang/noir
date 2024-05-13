use crate::abis::{
    accumulated_data::PrivateAccumulatedData, combined_constant_data::CombinedConstantData,
    validation_requests::ValidationRequests, call_request::CallRequest
};
use crate::constants::PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH;
use crate::traits::{Serialize, Deserialize};
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
