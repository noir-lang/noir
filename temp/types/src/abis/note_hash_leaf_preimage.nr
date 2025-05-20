global NOTE_HASH_LEAF_PREIMAGE_LENGTH: u32 = 1;

use crate::{
    abis::{read_request::ScopedReadRequest, side_effect::Readable},
    merkle_tree::leaf_preimage::LeafPreimage,
    traits::Empty,
};

pub struct NoteHashLeafPreimage {
    pub value: Field,
}

impl Empty for NoteHashLeafPreimage {
    fn empty() -> Self {
        Self { value: 0 }
    }
}

impl LeafPreimage for NoteHashLeafPreimage {
    fn get_key(self) -> Field {
        self.value
    }

    fn as_leaf(self) -> Field {
        self.value
    }
}

impl Readable<ScopedReadRequest> for NoteHashLeafPreimage {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        assert_eq(
            self.value,
            read_request.value(),
            "Value of the note hash leaf does not match read request",
        );
    }
}

impl NoteHashLeafPreimage {
    pub fn is_empty(self) -> bool {
        (self.value == 0)
    }

    pub fn serialize(self) -> [Field; NOTE_HASH_LEAF_PREIMAGE_LENGTH] {
        [self.value]
    }

    pub fn deserialize(fields: [Field; NOTE_HASH_LEAF_PREIMAGE_LENGTH]) -> Self {
        Self { value: fields[0] }
    }
}

impl Eq for NoteHashLeafPreimage {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

#[test]
fn serialization_of_empty() {
    let item = NoteHashLeafPreimage::empty();
    let serialized = item.serialize();
    let deserialized = NoteHashLeafPreimage::deserialize(serialized);
    assert(item.eq(deserialized));
}
