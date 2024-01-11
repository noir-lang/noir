use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;

struct PartialStateReference {
    note_hash_tree: AppendOnlyTreeSnapshot,
    nullifier_tree: AppendOnlyTreeSnapshot,
    contract_tree: AppendOnlyTreeSnapshot,
    public_data_tree: AppendOnlyTreeSnapshot,
}

impl PartialStateReference {
    fn eq(self, other: PartialStateReference) -> bool {
        self.note_hash_tree.eq(other.note_hash_tree) &
        self.nullifier_tree.eq(other.nullifier_tree) &
        self.contract_tree.eq(other.contract_tree) &
        self.public_data_tree.eq(other.public_data_tree)
    }
}
