use crate::messaging::l1_message::L1_TO_L2_MESSAGE_LEN;
use crate::messaging::l1_message::L1ToL2Message;
use crate::utils::arr_copy_slice;

struct L1ToL2MessageGetterData {
    message: L1ToL2Message,
    sibling_path: [Field; crate::L1_TO_L2_MESSAGES_TREE_HEIGHT],
    leaf_index: Field,
    root: Field,
}

fn l1_to_l2_message_getter_len() -> comptime Field {
    L1_TO_L2_MESSAGE_LEN + 1 + crate::L1_TO_L2_MESSAGES_TREE_HEIGHT + 1
}

fn make_l1_to_l2_message_getter_data(fields: [Field], start: comptime Field, secret: Field) -> L1ToL2MessageGetterData {
    L1ToL2MessageGetterData {
        message: L1ToL2Message::deserialize(arr_copy_slice(fields, [0; L1_TO_L2_MESSAGE_LEN], start), secret, fields[start + L1_TO_L2_MESSAGE_LEN]),
        leaf_index: fields[start + L1_TO_L2_MESSAGE_LEN],
        sibling_path: arr_copy_slice(fields, [0; crate::L1_TO_L2_MESSAGES_TREE_HEIGHT], L1_TO_L2_MESSAGE_LEN + 1),
        root: fields[start + L1_TO_L2_MESSAGE_LEN + crate::L1_TO_L2_MESSAGES_TREE_HEIGHT + 1],
    }
}