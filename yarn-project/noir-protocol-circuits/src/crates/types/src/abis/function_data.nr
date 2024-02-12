use crate::{
    abis::function_selector::FunctionSelector,
    constants::{GENERATOR_INDEX__FUNCTION_DATA, FUNCTION_DATA_LENGTH}, hash::pedersen_hash,
    traits::{Serialize, Hash, Deserialize}
};

struct FunctionData {
    // First four bytes of the abi encoding 
    // of a function. 
    selector : FunctionSelector,
    is_internal : bool,
    is_private : bool,
    is_constructor : bool,
}

impl Eq for FunctionData {
    fn eq(self, other: Self) -> bool {
        self.selector.eq(other.selector) &
        self.is_internal == other.is_internal &
        self.is_private == other.is_private &
        self.is_constructor == other.is_constructor
    }
}

impl Serialize<FUNCTION_DATA_LENGTH> for FunctionData {
    // A field is ~256 bits
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3057): Since, function data can fit into a Field,
    // This method will simply return a bit packed Field instead of hashing
    fn serialize(self) -> [Field; FUNCTION_DATA_LENGTH] {
        [
            self.selector.to_field(),
            self.is_internal as Field,
            self.is_private as Field,
            self.is_constructor as Field,
        ]
    }
}

impl Deserialize<FUNCTION_DATA_LENGTH> for FunctionData {
    fn deserialize(serialized: [Field; FUNCTION_DATA_LENGTH]) -> Self {
        Self {
            selector: FunctionSelector::from_field(serialized[0]),
            is_internal: serialized[1] as bool,
            is_private: serialized[2] as bool,
            is_constructor: serialized[3] as bool,
        }
    }
}

impl Hash for FunctionData {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__FUNCTION_DATA)
    }
}

#[test]
fn serialization_of_empty() {
    let data: FunctionData = dep::std::unsafe::zeroed();
    let serialized = data.serialize();
    let deserialized = FunctionData::deserialize(serialized);
    assert(data.eq(deserialized));
}

#[test]
fn empty_hash() {
    let data: FunctionData = dep::std::unsafe::zeroed();
    let hash = data.hash();

    // Value from function_data.test.ts "computes empty item hash" test
    assert_eq(hash, 0x200569267c0f73ac89aaa414239398db9445dd4ad3a8cf37015cd55b8d4c5e8d);
}
