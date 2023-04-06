use crate::merkle;

fn update_merkle_tree_leaf(
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
        hash_path
    ) == 1;
    // Constrain new leaf to be in new tree at same index
    constrain merkle::check_membership(
        new_root,
        new_leaf,
        index,
        hash_path
    ) == 1;
}

struct SparseMerkleTree {
    default_value: Field,
}

impl SparseMerkleTree {
    fn insert(self,
        old_root: Field,
        new_root: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) {
        update_merkle_tree_leaf(
            old_root,
            self.default_value,
            new_root,
            new_leaf,
            index,
            hash_path,
        );
    }

    fn delete(
        self,
        old_root: Field,
        old_leaf: Field,
        new_root: Field,
        index: Field,
        hash_path: [Field],
    ) {
        update_merkle_tree_leaf(
            old_root,
            old_leaf,
            new_root,
            self.default_value,
            index,
            hash_path,
        );
    }

    fn update(
        old_root: Field,
        old_leaf: Field,
        new_root: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) {
        update_merkle_tree_leaf(
            old_root,
            old_leaf,
            new_root,
            new_leaf,
            index,
            hash_path,
        );
    }
}
