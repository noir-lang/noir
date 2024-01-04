use crate::{
    constants::{
        BLOCK_HEADER_LENGTH,
        GENERATOR_INDEX__BLOCK_HASH,
    },
    hash::pedersen_hash,
};
use crate::traits::Empty;

// docs:start:block-header
struct BlockHeader {
    note_hash_tree_root : Field,
    nullifier_tree_root : Field,
    contract_tree_root : Field,
    l1_to_l2_message_tree_root : Field,
    archive_root: Field,
    public_data_tree_root: Field,
    global_variables_hash: Field,
}
// docs:end:block-header

impl Empty for BlockHeader {
    fn empty() -> Self {
        BlockHeader::deserialize([0; BLOCK_HEADER_LENGTH])
    }
}

impl BlockHeader {
    pub fn assert_is_zero(self) {
        assert(self.note_hash_tree_root == 0);
        assert(self.nullifier_tree_root == 0);
        assert(self.contract_tree_root == 0);
        assert(self.l1_to_l2_message_tree_root == 0);
        assert(self.archive_root == 0);
        assert(self.public_data_tree_root == 0);
    }

    pub fn serialize(self) -> [Field; BLOCK_HEADER_LENGTH] {
        // This comment was copied from the cpp codebase.
        //
        // TODO(#3441): Note private_kernel_vk_tree_root, is not included yet as
        // it is not present in noir,
        [
            self.note_hash_tree_root,
            self.nullifier_tree_root,
            self.contract_tree_root,
            self.l1_to_l2_message_tree_root,
            self.archive_root,
            self.public_data_tree_root,
            self.global_variables_hash
        ]
    }

    pub fn deserialize(deserialized: [Field; BLOCK_HEADER_LENGTH]) -> Self {
        BlockHeader {
            note_hash_tree_root: deserialized[0],
            nullifier_tree_root: deserialized[1],
            contract_tree_root: deserialized[2],
            l1_to_l2_message_tree_root: deserialized[3],
            archive_root: deserialized[4],
            public_data_tree_root: deserialized[5],
            global_variables_hash: deserialized[6],
        }
    }
    
    pub fn note_hash_tree_root(self) -> Field {
        self.note_hash_tree_root
    }

    pub fn contract_tree_root(self) -> Field {
        self.contract_tree_root
    }

    pub fn block_hash(self) -> Field {
        // TODO(#3442): Unify the ordering in `BlockHeader::serialize` function and the ordering
        // in the block hash preimage --> This requires changes in the circuits.
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        pedersen_hash([
            self.global_variables_hash,
            self.note_hash_tree_root,
            self.nullifier_tree_root,
            self.contract_tree_root,
            self.l1_to_l2_message_tree_root,
            self.public_data_tree_root,
        ], GENERATOR_INDEX__BLOCK_HASH)
    }
}
