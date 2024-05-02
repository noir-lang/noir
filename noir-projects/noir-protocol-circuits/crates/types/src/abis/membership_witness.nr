use crate::{
    constants::{
    FUNCTION_TREE_HEIGHT, NULLIFIER_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT, ROLLUP_VK_TREE_HEIGHT,
    ARCHIVE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT
},
    traits::Empty
};

// TODO(Kev): Instead of doing `MembershipWitness<FUNCTION_TREE_HEIGHT>` we are forced 
// to  do this new struct because the typescript bindings generator
// does not have logic to monomorphize these properly. See the file named
// `typechain-type-alias` in the folder `bug-collecting-crate`
struct FunctionLeafMembershipWitness{
    leaf_index: Field,
    sibling_path: [Field; FUNCTION_TREE_HEIGHT]
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

struct NoteHashMembershipWitness {
    leaf_index: Field,
    sibling_path: [Field; NOTE_HASH_TREE_HEIGHT],
}

impl Empty for VKMembershipWitness {
    fn empty() -> Self {
        VKMembershipWitness {
            leaf_index: 0,
            sibling_path: [0; ROLLUP_VK_TREE_HEIGHT]
        }
    }
}

impl Empty for NullifierMembershipWitness {
    fn empty() -> Self {
        NullifierMembershipWitness {
            leaf_index: 0,
            sibling_path: [0; NULLIFIER_TREE_HEIGHT]
        }
    }
}

impl Empty for PublicDataMembershipWitness {
    fn empty() -> Self {
        PublicDataMembershipWitness {
            leaf_index: 0,
            sibling_path: [0; PUBLIC_DATA_TREE_HEIGHT]
        }
    }
}

impl Empty for NoteHashMembershipWitness {
    fn empty() -> Self {
        NoteHashMembershipWitness {
            leaf_index: 0,
            sibling_path: [0; NOTE_HASH_TREE_HEIGHT]
        }
    }
}
