use crate::{
    abis::{function_data::FunctionData, private_circuit_public_inputs::PrivateCircuitPublicInputs},
    address::AztecAddress,
    constants::{GENERATOR_INDEX__CALL_STACK_ITEM, PRIVATE_CALL_STACK_ITEM_LENGTH}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize}, utils::reader::Reader
};

struct PrivateCallStackItem {
    // This is the _actual_ contract address relating to where this function's code resides in the
    // contract tree. Regardless of whether this is a call or delegatecall, this
    // `contract_address` _does not change_. Amongst other things, it's used as a lookup for
    // getting the correct code from the tree. There is a separate `storage_contract_address`
    // within a CallStackItem which varies depending on whether this is a call or delegatecall.
    contract_address: AztecAddress,
    function_data: FunctionData,
    public_inputs: PrivateCircuitPublicInputs,
}

impl Eq for PrivateCallStackItem {
    fn eq(self, other: Self) -> bool {
        self.contract_address.eq(other.contract_address) &
        self.function_data.eq(other.function_data) &
        self.public_inputs.eq(other.public_inputs)
    }
}

impl Serialize<PRIVATE_CALL_STACK_ITEM_LENGTH> for PrivateCallStackItem {
    fn serialize(self) -> [Field; PRIVATE_CALL_STACK_ITEM_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CALL_STACK_ITEM_LENGTH> = BoundedVec::new();

        fields.push(self.contract_address.to_field());
        fields.extend_from_array(self.function_data.serialize());
        fields.extend_from_array(self.public_inputs.serialize());

        assert_eq(fields.len(), PRIVATE_CALL_STACK_ITEM_LENGTH);

        fields.storage
    }
}

impl Deserialize<PRIVATE_CALL_STACK_ITEM_LENGTH> for PrivateCallStackItem {
    fn deserialize(serialized: [Field; PRIVATE_CALL_STACK_ITEM_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);

        let item = Self {
            contract_address: reader.read_struct(AztecAddress::deserialize),
            function_data: reader.read_struct(FunctionData::deserialize),
            public_inputs: reader.read_struct(PrivateCircuitPublicInputs::deserialize),
        };

        reader.finish();
        item
    }
}

impl Hash for PrivateCallStackItem {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__CALL_STACK_ITEM)
    }
}

#[test]
fn serialization_of_empty() {
    let item: PrivateCallStackItem = dep::std::unsafe::zeroed();
    let serialized = item.serialize();
    let deserialized = PrivateCallStackItem::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn empty_hash() {
    let mut item: PrivateCallStackItem = dep::std::unsafe::zeroed();
    item.function_data.is_private = true;
    let hash = item.hash();

    // Value from private_call_stack_item.test.ts "computes empty item hash" test
    let test_data_empty_hash = 0x13f518365c690d1b96d31454afed495ad29fe530939caf7189dd44f9bd63ef89;
    assert_eq(hash, test_data_empty_hash);
}
