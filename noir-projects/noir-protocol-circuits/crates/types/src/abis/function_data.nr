use crate::{
    abis::function_selector::FunctionSelector,
    constants::{GENERATOR_INDEX__FUNCTION_DATA, FUNCTION_DATA_LENGTH}, hash::pedersen_hash,
    traits::{Serialize, Hash, Deserialize, Empty}
};

struct FunctionData {
    selector : FunctionSelector,
    is_private : bool,
    is_static : bool,
}

impl Eq for FunctionData {
    fn eq(self, other: Self) -> bool {
        self.selector.eq(other.selector) &
        (self.is_private == other.is_private) &
        (self.is_static == other.is_static)
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
            self.is_static as Field
        ]
    }
}

impl Deserialize<FUNCTION_DATA_LENGTH> for FunctionData {
    fn deserialize(serialized: [Field; FUNCTION_DATA_LENGTH]) -> Self {
        Self {
            selector: FunctionSelector::from_field(serialized[0]),
            is_private: serialized[1] as bool,
            is_static: serialized[2] as bool
        }
    }
}

impl Hash for FunctionData {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__FUNCTION_DATA)
    }
}

impl Empty for FunctionData {
    fn empty() -> Self {
        FunctionData {
            selector: FunctionSelector::empty(),
            is_private: false,
            is_static: false
        }
    }

}

#[test]
fn serialization_of_empty() {
    let data = FunctionData::empty();
    let serialized = data.serialize();
    let deserialized = FunctionData::deserialize(serialized);
    assert(data.eq(deserialized));
}

#[test]
fn empty_hash() {
    let data = FunctionData::empty();
    let hash = data.hash();

    // Value from function_data.test.ts "computes empty function data hash" test
    let test_data_empty_hash = 0x066e6cdc4a6ba5e4781deda650b0be6c12f975f064fc38df72c1060716759b17;
    assert_eq(hash, test_data_empty_hash);
}
