global NOTE_HASH_LEAF_PREIMAGE_LENGTH: u64 = 1;

use crate::{
    abis::{read_request::ReadRequestContext, side_effect::Readable}, hash::silo_note_hash,
    merkle_tree::leaf_preimage::LeafPreimage, traits::{Empty, Hash}
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
    fn assert_match_read_request(self, read_request: ReadRequestContext) {
        // TODO(#2847): Read request value shouldn't have been siloed by apps.
        // let siloed_value = silo_note_hash(read_request.contract_address, read_request.value);
        // assert_eq(self.value, siloed_value, "Value of the note hash leaf does not match read request");
        assert_eq(self.value, read_request.value, "Value of the note hash leaf does not match read request");
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
