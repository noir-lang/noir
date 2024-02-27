use crate::messaging::l1_to_l2_message::L1ToL2Message;
use dep::protocol_types::{constants::{L1_TO_L2_MSG_TREE_HEIGHT, L1_TO_L2_MESSAGE_LENGTH}, utils::arr_copy_slice};

struct L1ToL2MessageGetterData {
    message: L1ToL2Message,
    sibling_path: [Field; L1_TO_L2_MSG_TREE_HEIGHT],
    leaf_index: Field
}

pub fn l1_to_l2_message_getter_len() -> u64 {
    L1_TO_L2_MESSAGE_LENGTH + 1 + L1_TO_L2_MSG_TREE_HEIGHT
}

pub fn make_l1_to_l2_message_getter_data<N>(
    fields: [Field; N],
    start: u64,
    secret: Field
) -> L1ToL2MessageGetterData {
    L1ToL2MessageGetterData {
        message: L1ToL2Message::deserialize(
            arr_copy_slice(fields, [0; L1_TO_L2_MESSAGE_LENGTH], start),
            secret,
            fields[start + L1_TO_L2_MESSAGE_LENGTH]
        ),
        leaf_index: fields[start + L1_TO_L2_MESSAGE_LENGTH],
        sibling_path: arr_copy_slice(
            fields,
            [0; L1_TO_L2_MSG_TREE_HEIGHT],
            L1_TO_L2_MESSAGE_LENGTH + 1
        )
    }
}
