use crate::merkle;

struct SparseMerkleTree {
    default_value: Field,
}

impl SparseMerkleTree {
    /// Constrains that the new leaf is located in the new tree at an index
    /// that previously held the default value in the old tree.
    fn insert(self,
        old_root: Field,
        new_root: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) {
        SparseMerkleTree::update(
            old_root,
            self.default_value,
            new_root,
            new_leaf,
            index,
            hash_path,
        );
    }

    /// Constrains that the new tree holds the default value at the same
    /// index that the old tree held the old leaf.
    fn delete(
        self,
        old_root: Field,
        old_leaf: Field,
        new_root: Field,
        index: Field,
        hash_path: [Field],
    ) {
        SparseMerkleTree::update(
            old_root,
            old_leaf,
            new_root,
            self.default_value,
            index,
            hash_path,
        );
    }

    /// Constrains that the changed leaf's before and after values are
    /// consistent with the given merkle roots, and that the leaf at the
    /// specified index is the only difference between the trees.
    fn update(
        old_root: Field,
        old_leaf: Field,
        new_root: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) {
        // Constrain old leaf to be in old tree
        constrain merkle::check_membership(
            old_root,
            old_leaf,
            index,
            hash_path,
        ) == 1;
        // Constrain new leaf to be in new tree at same index
        constrain merkle::check_membership(
            new_root,
            new_leaf,
            index,
            hash_path,
        ) == 1;
    }
}
