use crate::{
    hash::{compute_secret_hash, compute_message_hash, compute_message_nullifier},
    oracle::get_l1_to_l2_membership_witness::get_l1_to_l2_membership_witness
};

use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::{constants::L1_TO_L2_MSG_TREE_HEIGHT, address::{AztecAddress, EthAddress}, utils::arr_copy_slice};

pub fn process_l1_to_l2_message(
    l1_to_l2_root: Field,
    storage_contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    chain_id: Field,
    version: Field,
    content: Field,
    secret: Field
) -> Field {
    let secret_hash = compute_secret_hash(secret);
    let message_hash = compute_message_hash(
        portal_contract_address,
        chain_id,
        storage_contract_address,
        version,
        content,
        secret_hash
    );

    let returned_message = get_l1_to_l2_membership_witness(storage_contract_address, message_hash, secret);
    let leaf_index = returned_message[0];
    let sibling_path = arr_copy_slice(returned_message, [0; L1_TO_L2_MSG_TREE_HEIGHT], 1);

    // Check that the message is in the tree
    // This is implicitly checking that the values of the message are correct
    let root = compute_merkle_root(message_hash, leaf_index, sibling_path);
    assert(root == l1_to_l2_root, "Message not in state");

    compute_message_nullifier(message_hash, secret, leaf_index)
}
