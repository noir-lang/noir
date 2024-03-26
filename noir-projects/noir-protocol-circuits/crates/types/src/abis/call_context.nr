use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress},
    constants::{CALL_CONTEXT_LENGTH, GENERATOR_INDEX__CALL_CONTEXT}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize}, abis::side_effect::Ordered
};

// docs:start:call-context
struct CallContext {
    msg_sender : AztecAddress,
    storage_contract_address : AztecAddress,
    portal_contract_address : EthAddress,

    function_selector : FunctionSelector,

    is_delegate_call : bool,
    is_static_call : bool,

    side_effect_counter : u32,
}
// docs:end:call-context

impl CallContext {
    fn assert_is_zero(self) {
        let serialized: [Field; CALL_CONTEXT_LENGTH] = self.serialize();

        for i in 0..CALL_CONTEXT_LENGTH {
            assert(serialized[i] == 0);
        }
    }
}

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
        [
            self.msg_sender.to_field(),
            self.storage_contract_address.to_field(),
            self.portal_contract_address.to_field(),
            self.function_selector.to_field(),
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.side_effect_counter as Field,
        ]
    }
}

impl Deserialize<CALL_CONTEXT_LENGTH> for CallContext {
    fn deserialize(serialized: [Field; CALL_CONTEXT_LENGTH]) -> CallContext {
        CallContext {
            msg_sender: AztecAddress::from_field(serialized[0]),
            storage_contract_address: AztecAddress::from_field(serialized[1]),
            portal_contract_address: EthAddress::from_field(serialized[2]),
            function_selector: FunctionSelector::from_field(serialized[3]),
            is_delegate_call: serialized[4] as bool,
            is_static_call: serialized[5] as bool,
            side_effect_counter: serialized[6] as u32,
        }
    }
}

#[test]
fn serialize_deserialize_of_empty() {
    let context: CallContext = dep::std::unsafe::zeroed();
    let serialized = context.serialize();
    let deserialized = CallContext::deserialize(serialized);
    assert(context.eq(deserialized));
}

#[test]
fn assert_is_zero() {
    let context: CallContext = dep::std::unsafe::zeroed();
    context.assert_is_zero();
}

#[test(should_fail)]
fn not_zero_assert_is_zero() {
    let mut context: CallContext = dep::std::unsafe::zeroed();
    context.is_delegate_call = true;
    context.assert_is_zero();
}

#[test]
fn test_eq() {
    let mut context1: CallContext = dep::std::unsafe::zeroed();
    let mut context2: CallContext = dep::std::unsafe::zeroed();

    context1.is_delegate_call = true;
    context2.is_delegate_call = true;

    let address: AztecAddress = AztecAddress::from_field(69420);
    context1.msg_sender = address;
    context2.msg_sender = address;

    assert(context1.eq(context2));
}

#[test(should_fail)]
fn not_eq_test_eq() {
    let mut context1: CallContext = dep::std::unsafe::zeroed();
    let mut context2: CallContext = dep::std::unsafe::zeroed();

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
    let context: CallContext = dep::std::unsafe::zeroed();
    let _hashed = context.hash();
}
