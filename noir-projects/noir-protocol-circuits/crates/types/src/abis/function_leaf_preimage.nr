use crate::{
    abis::function_selector::FunctionSelector,
    constants::{GENERATOR_INDEX__FUNCTION_LEAF, FUNCTION_LEAF_PREIMAGE_LENGTH}, hash::pedersen_hash,
    traits::{Serialize, Hash, Deserialize}
};

struct FunctionLeafPreimage {
    selector : FunctionSelector,
    is_internal : bool,
    is_private : bool,
    vk_hash : Field,
    acir_hash : Field
}

impl Eq for FunctionLeafPreimage {
    fn eq(self, other: Self) -> bool {
        self.selector.eq(other.selector) &
        (self.is_internal == other.is_internal) &
        (self.is_private == other.is_private) &
        (self.vk_hash == other.vk_hash) &
        (self.acir_hash == other.acir_hash)
    }
}

impl Serialize<FUNCTION_LEAF_PREIMAGE_LENGTH> for FunctionLeafPreimage {
    fn serialize(self) -> [Field; FUNCTION_LEAF_PREIMAGE_LENGTH] {
        [
            self.selector.to_field(),
            self.is_internal as Field,
            self.is_private as Field,
            self.vk_hash,
            self.acir_hash,
        ]
    }
}

impl Deserialize<FUNCTION_LEAF_PREIMAGE_LENGTH> for FunctionLeafPreimage {
    fn deserialize(serialized: [Field; FUNCTION_LEAF_PREIMAGE_LENGTH]) -> Self {
        Self {
            selector: FunctionSelector::from_field(serialized[0]),
            is_internal: serialized[1] as bool,
            is_private: serialized[2] as bool,
            vk_hash: serialized[3],
            acir_hash: serialized[4],
        }
    }
}

impl Hash for FunctionLeafPreimage {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__FUNCTION_LEAF)
    }
}

#[test]
fn serialization_of_empty() {
    let data: FunctionLeafPreimage = dep::std::unsafe::zeroed();
    let serialized = data.serialize();
    let deserialized = FunctionLeafPreimage::deserialize(serialized);
    assert(data.eq(deserialized));
}

#[test]
fn empty_hash() {
    let data: FunctionLeafPreimage = dep::std::unsafe::zeroed();
    let hash = data.hash();

    // Value from function_leaf_preimage.test.ts "computes a function leaf" test
    assert_eq(hash, 0x1f2e3193c7187347a099ee7cb5d6ac077da6b18706fe5508e658a3d0a05494f7);
}

#[test]
fn compute_function_leaf() {
    let leaf = FunctionLeafPreimage { selector: FunctionSelector::from_u32(27), is_internal: false, is_private: true, vk_hash: 1, acir_hash: 2 };

    assert_eq(leaf.hash(), 0x1ad8ece7f40e63d011ae47c6ce6cdaf31d632a23f5cf35bbeaaf69c8302afdbc);
}
