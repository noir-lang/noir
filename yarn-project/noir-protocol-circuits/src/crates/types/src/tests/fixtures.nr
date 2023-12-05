mod contract_functions;
mod contract_tree;
mod contracts;
mod note_hash_tree;
mod read_requests;

use crate::address::Address;
use crate::abis::block_header::BlockHeader;
use crate::block::Block;
use crate::point::Point;
use crate::tests::fixtures;

global MSG_SENDER = Address { inner: 27 };

global DEPLOYER_PUBLIC_KEY = Point { x: 123456789, y: 123456789 };

global BLOCK_HEADER = BlockHeader {
    archive_root: 0,
    block: Block {
        note_hash_tree_root: fixtures::note_hash_tree::ROOT,
        nullifier_tree_root: 0,
        contract_tree_root: fixtures::contract_tree::ROOT,
        l1_to_l2_messages_tree_root: 0,
        public_data_tree_root: 0,
        global_variables_hash: 0,
    },
    private_kernel_vk_tree_root: 0,
};