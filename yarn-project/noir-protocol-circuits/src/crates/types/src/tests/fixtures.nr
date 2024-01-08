mod contract_functions;
mod contract_tree;
mod contracts;
mod note_hash_tree;
mod read_requests;

use crate::address::AztecAddress;
use crate::abis::block_header::BlockHeader;
use crate::grumpkin_point::GrumpkinPoint;
use crate::tests::fixtures;

global MSG_SENDER = AztecAddress { inner: 27 };

global DEPLOYER_PUBLIC_KEY = GrumpkinPoint { x: 123456789, y: 123456789 };

global BLOCK_HEADER = BlockHeader {
    note_hash_tree_root: fixtures::note_hash_tree::ROOT,
    nullifier_tree_root: 0,
    contract_tree_root: fixtures::contract_tree::ROOT,
    l1_to_l2_message_tree_root: 0,
    archive_root: 0,
    public_data_tree_root: 0,
    global_variables_hash: 0,
};
