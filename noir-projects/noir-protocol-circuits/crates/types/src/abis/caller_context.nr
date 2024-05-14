use crate::address::AztecAddress;
use dep::std::cmp::Eq;
use crate::traits::{Empty, Serialize, Deserialize};
use crate::constants::CALLER_CONTEXT_LENGTH;
use crate::utils::reader::Reader;

struct CallerContext {
    msg_sender: AztecAddress,
    storage_contract_address: AztecAddress,
}

impl Eq for CallerContext {
    fn eq(self, caller_context: CallerContext) -> bool {
        caller_context.msg_sender.eq(self.msg_sender)
            & caller_context.storage_contract_address.eq(self.storage_contract_address)
    }
}

impl Empty for CallerContext {
    fn empty() -> Self {
        CallerContext {
            msg_sender: AztecAddress::zero(),
            storage_contract_address: AztecAddress::zero(),
        }
    }
}

impl CallerContext {
    pub fn is_empty(self) -> bool {
        self.msg_sender.is_zero() & self.storage_contract_address.is_zero()
    }
}

impl Serialize<CALLER_CONTEXT_LENGTH> for CallerContext {
  fn serialize(self) -> [Field; CALLER_CONTEXT_LENGTH] {
    let mut fields: BoundedVec<Field, CALLER_CONTEXT_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.msg_sender.serialize());
    fields.extend_from_array(self.storage_contract_address.serialize());

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
