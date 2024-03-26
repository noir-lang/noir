use crate::{
    abis::function_selector::FunctionSelector,
    constants::{GENERATOR_INDEX__FUNCTION_DATA, FUNCTION_DATA_LENGTH}, hash::pedersen_hash,
    traits::{Serialize, Hash, Deserialize}
};

struct FunctionData {
    selector : FunctionSelector,
    is_private : bool,
    // Remove once the AVM is fully operational.
    is_transpiled: bool,
}

impl Eq for FunctionData {
    fn eq(self, other: Self) -> bool {
        self.selector.eq(other.selector) &
        (self.is_private == other.is_private) &
        (self.is_transpiled == other.is_transpiled)
    }
}

impl Serialize<FUNCTION_DATA_LENGTH> for FunctionData {
    // A field is ~256 bits
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3057): Since, function data can fit into a Field,
    // This method will simply return a bit packed Field instead of hashing
    fn serialize(self) -> [Field; FUNCTION_DATA_LENGTH] {
        [
            self.selector.to_field(),
            self.is_private as Field,
            self.is_transpiled as Field,
        ]
    }
}

impl Deserialize<FUNCTION_DATA_LENGTH> for FunctionData {
    fn deserialize(serialized: [Field; FUNCTION_DATA_LENGTH]) -> Self {
        Self {
            selector: FunctionSelector::from_field(serialized[0]),
            is_private: serialized[1] as bool,
            is_transpiled: serialized[2] as bool,
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
    let test_data_empty_hash = 0x066e6cdc4a6ba5e4781deda650b0be6c12f975f064fc38df72c1060716759b17;
    assert_eq(hash, test_data_empty_hash);
}
