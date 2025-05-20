use crate::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    constants::TREE_SNAPSHOTS_LENGTH,
    partial_state_reference::PartialStateReference,
    traits::{Deserialize, Empty, Serialize},
};

pub struct TreeSnapshots {
    pub l1_to_l2_message_tree: AppendOnlyTreeSnapshot,
    pub note_hash_tree: AppendOnlyTreeSnapshot,
    pub nullifier_tree: AppendOnlyTreeSnapshot,
    pub public_data_tree: AppendOnlyTreeSnapshot,
}

impl TreeSnapshots {
    pub fn to_partial_state_reference(self) -> PartialStateReference {
        PartialStateReference {
            note_hash_tree: self.note_hash_tree,
            nullifier_tree: self.nullifier_tree,
            public_data_tree: self.public_data_tree,
        }
    }
}

impl Eq for TreeSnapshots {
    fn eq(self, other: TreeSnapshots) -> bool {
        self.l1_to_l2_message_tree.eq(other.l1_to_l2_message_tree)
            & self.note_hash_tree.eq(other.note_hash_tree)
            & self.nullifier_tree.eq(other.nullifier_tree)
            & self.public_data_tree.eq(other.public_data_tree)
    }
}

impl Serialize<TREE_SNAPSHOTS_LENGTH> for TreeSnapshots {
    fn serialize(self) -> [Field; TREE_SNAPSHOTS_LENGTH] {
        let serialized_l1_to_l2_message_tree = self.l1_to_l2_message_tree.serialize();
        let serialized_note_hash_tree = self.note_hash_tree.serialize();
        let serialized_nullifier_tree = self.nullifier_tree.serialize();
        let serialized_public_data_tree = self.public_data_tree.serialize();

        [
            serialized_l1_to_l2_message_tree[0],
            serialized_l1_to_l2_message_tree[1],
            serialized_note_hash_tree[0],
            serialized_note_hash_tree[1],
            serialized_nullifier_tree[0],
            serialized_nullifier_tree[1],
            serialized_public_data_tree[0],
            serialized_public_data_tree[1],
        ]
    }
}

impl Deserialize<TREE_SNAPSHOTS_LENGTH> for TreeSnapshots {
    fn deserialize(serialized: [Field; TREE_SNAPSHOTS_LENGTH]) -> TreeSnapshots {
        TreeSnapshots {
            l1_to_l2_message_tree: AppendOnlyTreeSnapshot::deserialize([
                serialized[0],
                serialized[1],
            ]),
            note_hash_tree: AppendOnlyTreeSnapshot::deserialize([serialized[2], serialized[3]]),
            nullifier_tree: AppendOnlyTreeSnapshot::deserialize([serialized[4], serialized[5]]),
            public_data_tree: AppendOnlyTreeSnapshot::deserialize([serialized[6], serialized[7]]),
        }
    }
}

impl Empty for TreeSnapshots {
    fn empty() -> Self {
        Self {
            l1_to_l2_message_tree: AppendOnlyTreeSnapshot::zero(),
            note_hash_tree: AppendOnlyTreeSnapshot::zero(),
            nullifier_tree: AppendOnlyTreeSnapshot::zero(),
            public_data_tree: AppendOnlyTreeSnapshot::zero(),
        }
    }
}

#[test]
fn serialization_of_empty() {
    let item = TreeSnapshots::empty();
    let serialized = item.serialize();
    let deserialized = TreeSnapshots::deserialize(serialized);
    assert_eq(item, deserialized);
}
