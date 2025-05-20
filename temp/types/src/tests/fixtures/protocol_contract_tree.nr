use crate::{
    abis::protocol_contract_leaf_preimage::ProtocolContractLeafPreimage,
    constants::PROTOCOL_CONTRACT_TREE_HEIGHT,
    merkle_tree::{leaf_preimage::LeafPreimage, merkle_tree::MerkleTree},
    tests::fixtures::contracts::get_protocol_contract,
    traits::{Empty, ToField},
    utils::arrays::get_sorted_tuple::get_sorted_tuple,
};

global PROTOCOL_CONTRACT_TREE_WIDTH: u32 = 1 << PROTOCOL_CONTRACT_TREE_HEIGHT as u8;
global NUM_PROTOCOL_CONTRACTS_FOR_TESTING: u32 = 3;

pub fn get_protocol_contract_tree() -> (MerkleTree<PROTOCOL_CONTRACT_TREE_WIDTH>, [ProtocolContractLeafPreimage; PROTOCOL_CONTRACT_TREE_WIDTH]) {
    let contracts = [0; NUM_PROTOCOL_CONTRACTS_FOR_TESTING].mapi(|i, _| {
        get_protocol_contract(i + 1).address.to_field()
    });

    // Safety: This is only called in tests.
    let sorted = unsafe { get_sorted_tuple(contracts, |a, b| a.lt(b)) };

    let mut next_addresses = [0; NUM_PROTOCOL_CONTRACTS_FOR_TESTING];
    for i in 0..sorted.len() {
        next_addresses[sorted[i].original_index] = if i == NUM_PROTOCOL_CONTRACTS_FOR_TESTING - 1 {
            0
        } else {
            sorted[i + 1].elem
        };
    }

    let mut leaf_preimages = [ProtocolContractLeafPreimage::empty(); PROTOCOL_CONTRACT_TREE_WIDTH];
    for i in 0..contracts.len() {
        leaf_preimages[i + 1] =
            ProtocolContractLeafPreimage { address: contracts[i], next_address: next_addresses[i] };
    }

    (
        MerkleTree::new(
            leaf_preimages.map(|l: ProtocolContractLeafPreimage| l.as_leaf()),
        ), leaf_preimages,
    )
}
