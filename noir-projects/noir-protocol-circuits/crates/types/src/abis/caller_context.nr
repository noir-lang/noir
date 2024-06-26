use crate::address::AztecAddress;
use crate::traits::{Empty, Serialize, Deserialize};
use crate::constants::CALLER_CONTEXT_LENGTH;
use crate::utils::reader::Reader;

struct CallerContext {
    msg_sender: AztecAddress,
    storage_contract_address: AztecAddress,
    is_static_call: bool,
}

impl Eq for CallerContext {
    fn eq(self, other: CallerContext) -> bool {
        other.msg_sender.eq(self.msg_sender)
            & other.storage_contract_address.eq(self.storage_contract_address)
            & other.is_static_call == self.is_static_call
    }
}

impl Empty for CallerContext {
    fn empty() -> Self {
        CallerContext {
            msg_sender: AztecAddress::zero(),
            storage_contract_address: AztecAddress::zero(),
            is_static_call: false,
        }
    }
}

impl CallerContext {
    pub fn is_empty(self) -> bool {
        self.msg_sender.is_zero() & self.storage_contract_address.is_zero() & !self.is_static_call
    }

    // Different to an empty context, a hidden context won't reveal the caller's msg_sender and storage_contract_address,
    // but will still propagate the is_static_call flag.
    pub fn is_hidden(self) -> bool {
        self.msg_sender.is_zero() & self.storage_contract_address.is_zero()
    }
}

impl Serialize<CALLER_CONTEXT_LENGTH> for CallerContext {
  fn serialize(self) -> [Field; CALLER_CONTEXT_LENGTH] {
    let mut fields: BoundedVec<Field, CALLER_CONTEXT_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.msg_sender.serialize());
    fields.extend_from_array(self.storage_contract_address.serialize());
    fields.push(self.is_static_call as Field);

    assert_eq(fields.len(), CALLER_CONTEXT_LENGTH);

    fields.storage
  }
}

impl Deserialize<CALLER_CONTEXT_LENGTH> for CallerContext {
  fn deserialize(fields: [Field; CALLER_CONTEXT_LENGTH]) -> CallerContext {
    let mut reader = Reader::new(fields);

    let item = CallerContext {
      msg_sender: reader.read_struct(AztecAddress::deserialize),
      storage_contract_address: reader.read_struct(AztecAddress::deserialize),
      is_static_call: reader.read_bool(),
    };
    reader.finish();
    item
  }
}

#[test]
fn serialization_of_empty() {
    let item = CallerContext::empty();
    let serialized = item.serialize();
    let deserialized = CallerContext::deserialize(serialized);
    assert(item.eq(deserialized));
}
