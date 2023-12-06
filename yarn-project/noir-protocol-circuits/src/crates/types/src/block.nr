use crate::constants::GENERATOR_INDEX__BLOCK_HASH;

// This is not in the cpp code.
struct Block {
    // Private data
    note_hash_tree_root : Field,
    nullifier_tree_root : Field,
    contract_tree_root : Field,
    l1_to_l2_messages_tree_root : Field,

    // Public data
    public_data_tree_root : Field,
    global_variables_hash : Field,
}


impl Block {
    fn assert_is_zero(self) {
        assert(self.note_hash_tree_root == 0);
        assert(self.nullifier_tree_root == 0);
        assert(self.contract_tree_root == 0);
        assert(self.l1_to_l2_messages_tree_root == 0);
        assert(self.public_data_tree_root == 0);
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.global_variables_hash,
            self.note_hash_tree_root,
            self.nullifier_tree_root,
            self.contract_tree_root,
            self.l1_to_l2_messages_tree_root,
            self.public_data_tree_root,
        ], GENERATOR_INDEX__BLOCK_HASH)
    }
}