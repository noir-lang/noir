mod l1_to_l2_message;
mod l1_to_l2_message_getter_data;

use l1_to_l2_message_getter_data::make_l1_to_l2_message_getter_data;

use crate::oracle::get_l1_to_l2_message::get_l1_to_l2_message_call;

use dep::std::merkle::compute_merkle_root;

use dep::protocol_types::address::{
    AztecAddress,
    EthAddress,
};

// Returns the nullifier for the message
pub fn process_l1_to_l2_message(
    l1_to_l2_root: Field,
    storage_contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    chain_id: Field,
    version: Field,
    msg_key: Field,
    content: Field,
    secret: Field
) -> Field {
    let returned_message = get_l1_to_l2_message_call(msg_key);
    let l1_to_l2_message_data = make_l1_to_l2_message_getter_data(returned_message, 0, secret);

    // Check that the returned message is actually the message we looked up
    let msg_hash = l1_to_l2_message_data.message.hash();
    assert(msg_hash == msg_key, "Message not matching requested key");

    // Check that the message is in the tree
    let root = compute_merkle_root(
        msg_hash,
        l1_to_l2_message_data.leaf_index,
        l1_to_l2_message_data.sibling_path
    );
    assert(root == l1_to_l2_root, "Message not in state");

    // Validate this is the target contract
    assert(l1_to_l2_message_data.message.recipient.eq(storage_contract_address), "Invalid recipient");

    // Validate the sender is the portal contract
    assert(l1_to_l2_message_data.message.sender.eq(portal_contract_address), "Invalid sender");

    // Validate the chain id is correct
    assert(l1_to_l2_message_data.message.chainId == chain_id, "Invalid Chainid");

    // Validate the version is correct
    assert(l1_to_l2_message_data.message.version == version, "Invalid Version");

    // Validate the message hash is correct
    assert(l1_to_l2_message_data.message.content == content, "Invalid Content");

    // Validate the message secret is correct
    l1_to_l2_message_data.message.validate_message_secret();

    // Compute Nullifier
    l1_to_l2_message_data.message.compute_nullifier()
}
