use dep::types::{
    abis::{nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{
    MAX_NULLIFIERS_PER_TX, NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
    PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_TREE_HEIGHT
},
    merkle_tree::MembershipWitness
};

struct StateDiffHints {
    nullifier_predecessor_preimages: [NullifierLeafPreimage; MAX_NULLIFIERS_PER_TX],
    nullifier_predecessor_membership_witnesses: [MembershipWitness<NULLIFIER_TREE_HEIGHT>; MAX_NULLIFIERS_PER_TX],

    sorted_nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    sorted_nullifier_indexes: [u32; MAX_NULLIFIERS_PER_TX],

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the snapshots' `next_available_leaf_index` values (tree
    // snapshots of the relevant trees are stored in partial state reference).
    note_hash_subtree_sibling_path: [Field; NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH],
    nullifier_subtree_sibling_path: [Field; NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH],
    public_data_sibling_path: [Field; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH],
}
