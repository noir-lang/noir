use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress},
    constants::{CALL_CONTEXT_LENGTH, GENERATOR_INDEX__CALL_CONTEXT}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize, Empty}, abis::side_effect::Ordered,
    abis::gas_settings::GasSettings, utils::reader::Reader
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

    gas_settings: GasSettings,
    transaction_fee: Field,
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
        let mut serialized: BoundedVec<Field, CALL_CONTEXT_LENGTH> = BoundedVec::new();

        serialized.push(self.msg_sender.to_field());
        serialized.push(self.storage_contract_address.to_field());
        serialized.push(self.portal_contract_address.to_field());
        serialized.push(self.function_selector.to_field());
        serialized.push(self.is_delegate_call as Field);
        serialized.push(self.is_static_call as Field);
        serialized.push(self.side_effect_counter as Field);
        serialized.extend_from_array(self.gas_settings.serialize());
        serialized.push(self.transaction_fee);
    
        serialized.storage
    }
}

impl Deserialize<CALL_CONTEXT_LENGTH> for CallContext {
    fn deserialize(serialized: [Field; CALL_CONTEXT_LENGTH]) -> CallContext {
        let mut reader = Reader::new(serialized);
        CallContext {
            msg_sender: AztecAddress::from_field(reader.read()),
            storage_contract_address: AztecAddress::from_field(reader.read()),
            portal_contract_address: EthAddress::from_field(reader.read()),
            function_selector: FunctionSelector::from_field(reader.read()),
            is_delegate_call: reader.read() as bool,
            is_static_call: reader.read() as bool,
            side_effect_counter: reader.read() as u32,
            gas_settings: reader.read_struct(GasSettings::deserialize),
            transaction_fee: reader.read(),
        }
    }
}

impl Empty for CallContext {
    fn empty() -> Self {
        CallContext {
            msg_sender: AztecAddress::empty(),
            storage_contract_address: AztecAddress::empty(),
            portal_contract_address: EthAddress::empty(),
            function_selector: FunctionSelector::empty(),
            is_delegate_call: false,
            is_static_call: false,
            side_effect_counter: 0,
            gas_settings: GasSettings::empty(),
            transaction_fee: 0,
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
fn assert_is_zero() {
    let context = CallContext::empty();
    context.assert_is_zero();
}

#[test(should_fail)]
fn not_zero_assert_is_zero() {
    let mut context = CallContext::empty();
    context.is_delegate_call = true;
    context.assert_is_zero();
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
