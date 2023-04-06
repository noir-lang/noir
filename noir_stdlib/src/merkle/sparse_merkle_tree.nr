use crate::merkle;

struct SparseMerkleTree {
 default_value: Field,
 old_root: Field,
}

impl SparseMerkleTree {
    fn insert(
        new_root: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) -> {
        // Constrain to default leaf in old tree
        constrain merkle::check_membership(
            self.old_root,
            self.default_value,
            index,
            hash_path
        ) == 1;
        // Constrain to new leaf in new tree
        constrain merkle::check_membership(
            new_root,
            new_leaf,
            index,
            hash_path
        ) == 1;
    }

    fn delete(
        new_root: Field,
        old_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) -> {
        // Constrain to old leaf in old tree
        constrain merkle::check_membership(
            self.old_root,
            old_leaf,
            index,
            hash_path
        ) == 1;
        // Constrain to default leaf in new tree
        constrain merkle::check_membership(
            new_root,
            new_leaf,
            index,
            hash_path
        ) == 1;
    }

    fn update(
        new_root: Field,
        old_leaf: Field,
        new_leaf: Field,
        index: Field,
        hash_path: [Field],
    ) -> {
        // Constrain to old leaf in old tree
        constrain merkle::check_membership(
            old_root,
            old_leaf,
            index,
            hash_path
        ) == 1;
        // Constrain to new leaf in new tree
        constrain merkle::check_membership(
            new_root,
            new_leaf,
            index,
            hash_path
        ) == 1;
    }
}
