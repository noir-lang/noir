struct AppendOnlyTreeSnapshot {
    root : Field,
    next_available_leaf_index : u32
}

impl AppendOnlyTreeSnapshot{
    pub fn eq(self, other : AppendOnlyTreeSnapshot) -> bool {
        (self.root == other.root) & (self.next_available_leaf_index == other.next_available_leaf_index)
    }
}