pub struct AppendOnlyTreeSnapshot {
    pub root: Field,
    // TODO(Alvaro) change this to a u64
    pub next_available_leaf_index: u32,
}

pub global APPEND_ONLY_TREE_SNAPSHOT_LENGTH: u32 = 2;

impl AppendOnlyTreeSnapshot {
    pub fn serialize(self) -> [Field; APPEND_ONLY_TREE_SNAPSHOT_LENGTH] {
        [self.root, self.next_available_leaf_index as Field]
    }

    pub fn deserialize(
        serialized: [Field; APPEND_ONLY_TREE_SNAPSHOT_LENGTH],
    ) -> AppendOnlyTreeSnapshot {
        AppendOnlyTreeSnapshot {
            root: serialized[0],
            next_available_leaf_index: serialized[1] as u32,
        }
    }

    pub fn zero() -> Self {
        Self { root: 0, next_available_leaf_index: 0 }
    }
}

impl Eq for AppendOnlyTreeSnapshot {
    fn eq(self, other: AppendOnlyTreeSnapshot) -> bool {
        (self.root == other.root)
            & (self.next_available_leaf_index == other.next_available_leaf_index)
    }
}
