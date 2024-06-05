global NOTE_HASH_LEAF_PREIMAGE_LENGTH: u64 = 1;

use crate::{
    abis::{read_request::ScopedReadRequest, side_effect::Readable}, hash::compute_siloed_note_hash,
    merkle_tree::leaf_preimage::LeafPreimage, traits::{Empty, Hash, Eq}
};

struct NoteHashLeafPreimage {
    value : Field,
}

impl Empty for NoteHashLeafPreimage {
    fn empty() -> Self {
        Self {
            value : 0,
        }
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

impl Readable for NoteHashLeafPreimage {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        let siloed_value = compute_siloed_note_hash(read_request.contract_address, read_request.value());
        assert_eq(self.value, siloed_value, "Value of the note hash leaf does not match read request");
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

impl Eq for NoteHashLeafPreimage{
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
