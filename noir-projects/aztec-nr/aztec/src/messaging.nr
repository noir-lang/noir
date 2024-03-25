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
    let mut msg = L1ToL2Message::new(
        portal_contract_address,
        chain_id,
        storage_contract_address,
        version,
        content,
        secret
    );
    let message_hash = msg.hash();

    let returned_message = get_l1_to_l2_membership_witness(storage_contract_address, message_hash, secret);
    let leaf_index = returned_message[0];
    let sibling_path = arr_copy_slice(returned_message, [0; L1_TO_L2_MSG_TREE_HEIGHT], 1);

    // Check that the message is in the tree
    // This is implicitly checking that the values of the message are correct
    let root = compute_merkle_root(message_hash, leaf_index, sibling_path);
    assert(root == l1_to_l2_root, "Message not in state");

    // Note: Had to add this line to make it work (wasted an hour debugging this). L1ToL2Message noir struct is
    // an abomination and it would not have ever got to that state if people followed the logical rule of 
    // "deserialize(serialize(object)) == object" always being true (there are a few params which are not
    // serialzied). Will refactor that in a separate PR.
    // TODO(#5420)
    msg.tree_index = leaf_index;

    msg.compute_nullifier()
}
