mod get_commitment_getter_data;
mod l1_to_l2_message;
mod l1_to_l2_message_getter_data;

use l1_to_l2_message_getter_data::make_l1_to_l2_message_getter_data;

use crate::abi::PublicContextInputs;
use crate::oracle::create_nullifier::create_nullifier;
use crate::oracle::get_l1_to_l2_message::get_l1_to_l2_message_call;

// TODO(Maddiaa): Where to put this code?, a context would make sense but public calls dont _really_ have one? - come back to.
fn consume_l1_to_l2_message_public(pub_context: PublicContextInputs, msg_key: Field, content: Field, secret: Field) {
    let nullifier = process_l1_to_l2_message(pub_context.block_data.l1_to_l2_messages_tree_root, pub_context.call_context.storage_contract_address, msg_key, content, secret);

    // Push nullifier
    create_nullifier(nullifier);
}


// Returns the nullifier for the message
fn process_l1_to_l2_message(l1_to_l2_root: Field, storage_contract_address: Field, msg_key: Field, content: Field, secret: Field) -> Field{

    let returned_message = get_l1_to_l2_message_call(msg_key);
    let l1_to_l2_message_data = make_l1_to_l2_message_getter_data(returned_message, 0, secret);

    // Check tree roots against the inputs 
    assert(l1_to_l2_message_data.root == l1_to_l2_root);

    // Validate this is the target contract
    assert(l1_to_l2_message_data.message.recipient == storage_contract_address);

    // Validate the message hash is correct
    assert(l1_to_l2_message_data.message.content == content);

    // Validate the message secret is correct
    l1_to_l2_message_data.message.validate_message_secret();

    // Compute Nullifier
    l1_to_l2_message_data.message.compute_nullifier()
}