use crate::{
    abis::function_selector::FunctionSelector,
    constants::{GENERATOR_INDEX__FUNCTION_DATA, FUNCTION_DATA_LENGTH},
    hash::poseidon2_hash_with_separator, traits::{Serialize, Hash, Deserialize, Empty}
};

struct FunctionData {
    selector : FunctionSelector,
    is_private : bool,
}

impl Eq for FunctionData {
    fn eq(self, other: Self) -> bool {
        self.selector.eq(other.selector) &
        (self.is_private == other.is_private)
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
        ]
    }
}

impl Deserialize<FUNCTION_DATA_LENGTH> for FunctionData {
    fn deserialize(serialized: [Field; FUNCTION_DATA_LENGTH]) -> Self {
        Self {
            selector: FunctionSelector::from_field(serialized[0]),
            is_private: serialized[1] as bool,
        }
    }
}

impl Hash for FunctionData {
    fn hash(self) -> Field {
        poseidon2_hash_with_separator(self.serialize(), GENERATOR_INDEX__FUNCTION_DATA)
    }
}

impl Empty for FunctionData {
    fn empty() -> Self {
        FunctionData {
            selector: FunctionSelector::empty(),
            is_private: false
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
    let test_data_empty_hash = 0x2ee7682681179c2d8290a805cdeb64cc9744167fd5c801c26f1b3f9757c4eebc;
    assert_eq(hash, test_data_empty_hash);
}
