use crate::{
    abis::function_selector::FunctionSelector, address::AztecAddress,
    constants::{CALL_CONTEXT_LENGTH, GENERATOR_INDEX__CALL_CONTEXT}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize, Empty}, utils::reader::Reader
};

// docs:start:call-context
struct CallContext {
    msg_sender : AztecAddress,
    storage_contract_address : AztecAddress,
    function_selector : FunctionSelector,

    is_delegate_call : bool,
    is_static_call : bool,
}
// docs:end:call-context

impl Eq for CallContext {
    fn eq(self, other: CallContext) -> bool {
        self.serialize() == other.serialize()
    }
}

impl Hash for CallContext {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__CALL_CONTEXT)
    }
}

impl Serialize<CALL_CONTEXT_LENGTH> for CallContext {
    fn serialize(self) -> [Field; CALL_CONTEXT_LENGTH] {
        let mut serialized: BoundedVec<Field, CALL_CONTEXT_LENGTH> = BoundedVec::new();

        serialized.push(self.msg_sender.to_field());
        serialized.push(self.storage_contract_address.to_field());
        serialized.push(self.function_selector.to_field());
        serialized.push(self.is_delegate_call as Field);
        serialized.push(self.is_static_call as Field);
    
        serialized.storage
    }
}

impl Deserialize<CALL_CONTEXT_LENGTH> for CallContext {
    fn deserialize(serialized: [Field; CALL_CONTEXT_LENGTH]) -> CallContext {
        let mut reader = Reader::new(serialized);
        CallContext {
            msg_sender: AztecAddress::from_field(reader.read()),
            storage_contract_address: AztecAddress::from_field(reader.read()),
            function_selector: FunctionSelector::from_field(reader.read()),
            is_delegate_call: reader.read() as bool,
            is_static_call: reader.read() as bool,
        }
    }
}

impl Empty for CallContext {
    fn empty() -> Self {
        CallContext {
            msg_sender: AztecAddress::empty(),
            storage_contract_address: AztecAddress::empty(),
            function_selector: FunctionSelector::empty(),
            is_delegate_call: false,
            is_static_call: false,
        }
    }
}

#[test]
fn serialize_deserialize_of_empty() {
    let context = CallContext::empty();
    let serialized = context.serialize();
    let deserialized = CallContext::deserialize(serialized);
    assert(context.eq(deserialized));
}

#[test]
fn test_eq() {
    let mut context1 = CallContext::empty();
    let mut context2 = CallContext::empty();

    context1.is_delegate_call = true;
    context2.is_delegate_call = true;

    let address: AztecAddress = AztecAddress::from_field(69420);
    context1.msg_sender = address;
    context2.msg_sender = address;

    assert(context1.eq(context2));
}

#[test(should_fail)]
fn not_eq_test_eq() {
    let mut context1 = CallContext::empty();
    let mut context2 = CallContext::empty();

    context1.is_delegate_call = true;
    context2.is_delegate_call = false;

    let address1: AztecAddress = AztecAddress::from_field(69420);
    let address2: AztecAddress = AztecAddress::from_field(42069);

    context1.msg_sender = address1;
    context2.msg_sender = address2;

    assert(context1.eq(context2));
}

#[test]
fn hash_smoke() {
    let context = CallContext::empty();
    let _hashed = context.hash();
}
