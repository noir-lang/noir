pub global NULLIFIER_LEAF_PREIMAGE_LENGTH: u32 = 3;

use crate::{
    abis::{read_request::ScopedReadRequest, side_effect::Readable},
    hash::compute_siloed_nullifier,
    merkle_tree::leaf_preimage::{IndexedTreeLeafPreimage, LeafPreimage},
    traits::{Empty, Hash},
};

pub struct NullifierLeafPreimage {
    pub nullifier: Field,
    pub next_nullifier: Field,
    pub next_index: u32,
}

impl Empty for NullifierLeafPreimage {
    fn empty() -> Self {
        Self { nullifier: 0, next_nullifier: 0, next_index: 0 }
    }
}

impl Hash for NullifierLeafPreimage {
    fn hash(self) -> Field {
        if self.is_empty() {
            0
        } else {
            crate::hash::poseidon2_hash(self.serialize())
        }
    }
}

impl LeafPreimage for NullifierLeafPreimage {
    fn get_key(self) -> Field {
        self.nullifier
    }

    fn as_leaf(self) -> Field {
        self.hash()
    }
}

impl IndexedTreeLeafPreimage<Field> for NullifierLeafPreimage {
    fn get_next_key(self) -> Field {
        self.next_nullifier
    }

    fn points_to_infinity(self) -> bool {
        (self.next_nullifier == 0) & (self.next_index == 0)
    }

    fn update_pointers(self, next_key: Field, next_index: u32) -> Self {
        Self { nullifier: self.nullifier, next_nullifier: next_key, next_index }
    }

    fn update_value(self, _nullifier: Field) -> Self {
        assert(false, "Tried to update a nullifier");
        Self::empty()
    }

    fn build_insertion_leaf(nullifier: Field, low_leaf: Self) -> Self {
        Self { nullifier, next_nullifier: low_leaf.next_nullifier, next_index: low_leaf.next_index }
    }
}

impl Readable<ScopedReadRequest> for NullifierLeafPreimage {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        let siloed_value =
            compute_siloed_nullifier(read_request.contract_address, read_request.value());
        assert_eq(
            self.nullifier,
            siloed_value,
            "Value of the nullifier leaf does not match read request",
        );
    }
}

impl NullifierLeafPreimage {
    pub fn is_empty(self) -> bool {
        (self.nullifier == 0) & (self.next_nullifier == 0) & (self.next_index == 0)
    }

    pub fn serialize(self) -> [Field; NULLIFIER_LEAF_PREIMAGE_LENGTH] {
        [self.nullifier, self.next_nullifier, self.next_index as Field]
    }

    pub fn deserialize(fields: [Field; NULLIFIER_LEAF_PREIMAGE_LENGTH]) -> Self {
        Self { nullifier: fields[0], next_nullifier: fields[1], next_index: fields[2] as u32 }
    }
}

impl Eq for NullifierLeafPreimage {
    fn eq(self, other: Self) -> bool {
        (self.nullifier == other.nullifier)
            & (self.next_nullifier == other.next_nullifier)
            & (self.next_index == other.next_index)
    }
}

#[test]
fn serialization_of_empty() {
    let item = NullifierLeafPreimage::empty();
    let serialized = item.serialize();
    let deserialized = NullifierLeafPreimage::deserialize(serialized);
    assert(item.eq(deserialized));
}
