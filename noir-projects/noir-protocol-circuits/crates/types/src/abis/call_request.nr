use crate::{
    abis::{caller_context::CallerContext, side_effect::Ordered}, address::AztecAddress,
    constants::CALL_REQUEST_LENGTH, traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};

struct CallRequest {
    hash: Field,
    caller_contract_address: AztecAddress,
    caller_context: CallerContext,
    start_side_effect_counter: u32,
    end_side_effect_counter: u32,
}

impl Ordered for CallRequest {
    fn counter(self) -> u32 {
        self.start_side_effect_counter
    }
}

impl Eq for CallRequest {
    fn eq(self, call_request: CallRequest) -> bool {
        (call_request.hash == self.hash)
            & (call_request.caller_contract_address.eq(self.caller_contract_address))
            & (call_request.caller_context.eq(self.caller_context))
            & (call_request.start_side_effect_counter == self.start_side_effect_counter)
            & (call_request.end_side_effect_counter == self.end_side_effect_counter)
    }
}

impl Empty for CallRequest {
    fn empty() -> Self {
        CallRequest {
            hash: 0,
            caller_contract_address: AztecAddress::zero(),
            caller_context: CallerContext::empty(),
            start_side_effect_counter: 0,
            end_side_effect_counter: 0,
        }
    }
}

impl CallRequest {
    pub fn expose_to_public(self) -> Self {
        CallRequest {
            hash: self.hash,
            caller_contract_address: self.caller_contract_address,
            caller_context: self.caller_context,
            start_side_effect_counter: 0,
            end_side_effect_counter: 0
        }
    }
}

impl Serialize<CALL_REQUEST_LENGTH> for CallRequest {
  fn serialize(self) -> [Field; CALL_REQUEST_LENGTH] {
    let mut fields: BoundedVec<Field, CALL_REQUEST_LENGTH> = BoundedVec::new();

    fields.push(self.hash);
    fields.extend_from_array(self.caller_contract_address.serialize());
    fields.extend_from_array(self.caller_context.serialize());
    fields.push(self.start_side_effect_counter as Field);
    fields.push(self.end_side_effect_counter as Field);

    assert_eq(fields.len(), CALL_REQUEST_LENGTH);

    fields.storage
  }
}

impl Deserialize<CALL_REQUEST_LENGTH> for CallRequest {
  fn deserialize(fields: [Field; CALL_REQUEST_LENGTH]) -> CallRequest {
    let mut reader = Reader::new(fields);

    let item = CallRequest {
      hash: reader.read(),
      caller_contract_address: reader.read_struct(AztecAddress::deserialize),
      caller_context: reader.read_struct(CallerContext::deserialize),
      start_side_effect_counter: reader.read() as u32,
      end_side_effect_counter: reader.read() as u32,
    };
    reader.finish();
    item
  }
}

#[test]
fn serialization_of_empty() {
    let item = CallRequest::empty();
    let serialized = item.serialize();
    let deserialized = CallRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
