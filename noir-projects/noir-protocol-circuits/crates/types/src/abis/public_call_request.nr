use crate::{
    abis::{public_call_stack_item_compressed::PublicCallStackItemCompressed, side_effect::Ordered},
    address::AztecAddress, constants::PUBLIC_CALL_REQUEST_LENGTH,
    traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};

struct PublicCallRequest {
    item: PublicCallStackItemCompressed,
    counter: u32,
}

impl Ordered for PublicCallRequest {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for PublicCallRequest {
    fn eq(self, other: PublicCallRequest) -> bool {
        (other.item == self.item)
            & (other.counter == self.counter)
    }
}

impl Empty for PublicCallRequest {
    fn empty() -> Self {
        PublicCallRequest {
            item: PublicCallStackItemCompressed::empty(),
            counter: 0,
        }
    }
}

impl PublicCallRequest {
    pub fn expose_to_public(self) -> Self {
        PublicCallRequest { item: self.item, counter: 0 }
    }
}

impl Serialize<PUBLIC_CALL_REQUEST_LENGTH> for PublicCallRequest {
  fn serialize(self) -> [Field; PUBLIC_CALL_REQUEST_LENGTH] {
    let mut fields: BoundedVec<Field, PUBLIC_CALL_REQUEST_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.item.serialize());
    fields.push(self.counter as Field);

    assert_eq(fields.len(), PUBLIC_CALL_REQUEST_LENGTH);

    fields.storage
  }
}

impl Deserialize<PUBLIC_CALL_REQUEST_LENGTH> for PublicCallRequest {
  fn deserialize(fields: [Field; PUBLIC_CALL_REQUEST_LENGTH]) -> PublicCallRequest {
    let mut reader = Reader::new(fields);

    let request = PublicCallRequest {
      item: reader.read_struct(PublicCallStackItemCompressed::deserialize),
      counter: reader.read_u32(),
    };
    reader.finish();
    request
  }
}

#[test]
fn serialization_of_empty() {
    let item = PublicCallRequest::empty();
    let serialized = item.serialize();
    let deserialized = PublicCallRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
