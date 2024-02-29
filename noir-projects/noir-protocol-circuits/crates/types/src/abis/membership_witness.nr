use crate::constants::{
    CONTRACT_TREE_HEIGHT, FUNCTION_TREE_HEIGHT, NULLIFIER_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT,
    ROLLUP_VK_TREE_HEIGHT, ARCHIVE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT
};

struct MembershipWitness<N> {
    leaf_index: Field,
    sibling_path: [Field; N]
}

// TODO(Kev): Instead of doing `MembershipWitness<FUNCTION_TREE_HEIGHT>` we are forced 
// to  do this new struct because the typescript bindings generator
// does not have logic to monomorphize these properly. See the file named
// `typechain-type-alias` in the folder `bug-collecting-crate`
struct FunctionLeafMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; FUNCTION_TREE_HEIGHT]
}

struct ContractLeafMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; CONTRACT_TREE_HEIGHT]
}

struct VKMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; ROLLUP_VK_TREE_HEIGHT]
}

struct NullifierMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; NULLIFIER_TREE_HEIGHT]
}

struct PublicDataMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; PUBLIC_DATA_TREE_HEIGHT]
}

struct ArchiveRootMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; ARCHIVE_HEIGHT]
}

struct ReadRequestMembershipWitness {
    leaf_index: Field,
    sibling_path: [Field; NOTE_HASH_TREE_HEIGHT],
    // whether or not the read request corresponds to a pending note hash
    // In case we change the default to true, we have to adapt is_empty() method
    // hint to point kernel to the commitment this rr corresponds to
    is_transient: bool,
    hint_to_note_hash: Field,
}
