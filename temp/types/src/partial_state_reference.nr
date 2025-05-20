use crate::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    constants::PARTIAL_STATE_REFERENCE_LENGTH,
    traits::{Deserialize, Empty, Serialize},
};

pub struct PartialStateReference {
    pub note_hash_tree: AppendOnlyTreeSnapshot,
    pub nullifier_tree: AppendOnlyTreeSnapshot,
    pub public_data_tree: AppendOnlyTreeSnapshot,
}

impl Eq for PartialStateReference {
    fn eq(self, other: PartialStateReference) -> bool {
        self.note_hash_tree.eq(other.note_hash_tree)
            & self.nullifier_tree.eq(other.nullifier_tree)
            & self.public_data_tree.eq(other.public_data_tree)
    }
}

impl Serialize<PARTIAL_STATE_REFERENCE_LENGTH> for PartialStateReference {
    fn serialize(self) -> [Field; PARTIAL_STATE_REFERENCE_LENGTH] {
        let serialized_note_hash_tree = self.note_hash_tree.serialize();
        let serialized_nullifier_tree = self.nullifier_tree.serialize();
        let serialized_public_data_tree = self.public_data_tree.serialize();

        [
            serialized_note_hash_tree[0],
            serialized_note_hash_tree[1],
            serialized_nullifier_tree[0],
            serialized_nullifier_tree[1],
            serialized_public_data_tree[0],
            serialized_public_data_tree[1],
        ]
    }
}

impl Deserialize<PARTIAL_STATE_REFERENCE_LENGTH> for PartialStateReference {
    fn deserialize(serialized: [Field; PARTIAL_STATE_REFERENCE_LENGTH]) -> PartialStateReference {
        PartialStateReference {
            note_hash_tree: AppendOnlyTreeSnapshot::deserialize([serialized[0], serialized[1]]),
            nullifier_tree: AppendOnlyTreeSnapshot::deserialize([serialized[2], serialized[3]]),
            public_data_tree: AppendOnlyTreeSnapshot::deserialize([serialized[4], serialized[5]]),
        }
    }
}

impl Empty for PartialStateReference {
    fn empty() -> Self {
        Self {
            note_hash_tree: AppendOnlyTreeSnapshot::zero(),
            nullifier_tree: AppendOnlyTreeSnapshot::zero(),
            public_data_tree: AppendOnlyTreeSnapshot::zero(),
        }
    }
}

#[test]
fn serialization_of_empty() {
    let partial = PartialStateReference::empty();
    let _serialized = partial.serialize();
    let _deserialized = PartialStateReference::deserialize(_serialized);
}
