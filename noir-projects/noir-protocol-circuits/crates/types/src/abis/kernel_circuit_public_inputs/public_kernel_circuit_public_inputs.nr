use crate::abis::{
    accumulated_data::PublicAccumulatedData, combined_constant_data::CombinedConstantData,
    public_call_request::PublicCallRequest,
    validation_requests::{RollupValidationRequests, ValidationRequests}
};
use crate::constants::{MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH};
use crate::traits::{Empty, Serialize, Deserialize, Eq};
use crate::utils::reader::Reader;
use crate::address::AztecAddress;

struct PublicKernelCircuitPublicInputs {
    validation_requests: ValidationRequests,
    end_non_revertible: PublicAccumulatedData,
    end: PublicAccumulatedData,
    constants: CombinedConstantData,
    revert_code: u8,
    public_teardown_call_stack: [PublicCallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    fee_payer: AztecAddress
}

impl PublicKernelCircuitPublicInputs {
    pub fn needs_setup(self) -> bool {
        // public calls for setup are deposited in the non-revertible public call stack.
        // if an element is present, we need to run setup
        !self.end_non_revertible.public_call_stack[0].item.contract_address.is_zero()
    }

    pub fn needs_app_logic(self) -> bool {
        // public calls for app logic are deposited in the revertible public call stack.
        // if an element is present, we need to run app logic
        !self.end.public_call_stack[0].item.contract_address.is_zero()
    }

    pub fn needs_teardown(self) -> bool {
        // the public call specified for teardown, if any, is placed in the teardown call stack
        !self.public_teardown_call_stack[0].item.contract_address.is_zero()
    }
}

impl Empty for PublicKernelCircuitPublicInputs {
    fn empty() -> Self {
        PublicKernelCircuitPublicInputs {
            validation_requests: ValidationRequests::empty(),
            end_non_revertible: PublicAccumulatedData::empty(),
            end: PublicAccumulatedData::empty(),
            constants: CombinedConstantData::empty(),
            revert_code: 0,
            public_teardown_call_stack: [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
            fee_payer: AztecAddress::empty()
        }
    }
}

impl Serialize<PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PublicKernelCircuitPublicInputs {
  fn serialize(self) -> [Field; PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
    let mut fields: BoundedVec<Field, PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.validation_requests.serialize());
    fields.extend_from_array(self.end_non_revertible.serialize());
    fields.extend_from_array(self.end.serialize());
    fields.extend_from_array(self.constants.serialize());
    fields.push(self.revert_code as Field);

    for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
      fields.extend_from_array(self.public_teardown_call_stack[i].serialize());
    }

    fields.extend_from_array(self.fee_payer.serialize());
    

    assert_eq(fields.len(), PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PublicKernelCircuitPublicInputs {
  fn deserialize(fields: [Field; PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> PublicKernelCircuitPublicInputs {
    let mut reader = Reader::new(fields);

    let item = PublicKernelCircuitPublicInputs {
        validation_requests: reader.read_struct(ValidationRequests::deserialize),
        end_non_revertible: reader.read_struct(PublicAccumulatedData::deserialize),
        end: reader.read_struct(PublicAccumulatedData::deserialize),
        constants: reader.read_struct(CombinedConstantData::deserialize),
        revert_code: reader.read() as u8,
        public_teardown_call_stack: reader.read_struct_array(PublicCallRequest::deserialize, [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]),
        fee_payer: reader.read_struct(AztecAddress::deserialize),
    };
    reader.finish();
    item
  }
}

impl Eq for PublicKernelCircuitPublicInputs {
  fn eq(self, other: Self) -> bool {
    (self.validation_requests == other.validation_requests) &
    (self.end_non_revertible == other.end_non_revertible) &
    (self.end == other.end) &
    (self.constants == other.constants) &
    (self.revert_code == other.revert_code) &
    (self.public_teardown_call_stack == other.public_teardown_call_stack)
  }
}

#[test]
fn serialization_of_empty() {
    let item = PublicKernelCircuitPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = PublicKernelCircuitPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
