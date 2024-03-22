mod l1_to_l2_message;

use crate::oracle::get_l1_to_l2_membership_witness::get_l1_to_l2_membership_witness;

use dep::std::merkle::compute_merkle_root;
use crate::messaging::l1_to_l2_message::L1ToL2Message;
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
    let msg = L1ToL2Message::new(
        portal_contract_address,
        chain_id,
        storage_contract_address,
        version,
        content,
        secret
    );
    let message_hash = msg.hash();

    let returned_message = get_l1_to_l2_membership_witness(message_hash);
    let leaf_index = returned_message[0];
    let sibling_path = arr_copy_slice(returned_message, [0; L1_TO_L2_MSG_TREE_HEIGHT], 1);

    // Check that the message is in the tree
    // This is implicitly checking that the values of the message are correct
    let root = compute_merkle_root(message_hash, leaf_index, sibling_path);
    assert(root == l1_to_l2_root, "Message not in state");

    msg.compute_nullifier()
}
