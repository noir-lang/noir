
global L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH = 18; // 8 fields + 8 deep hash path + 1 root + 1 index

// Checks if a msg is within the l1ToL2Msg tree
#[oracle(getL1ToL2Message)]
fn get_l1_to_l2_msg_oracle(_msg_key: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {}

unconstrained fn get_l1_to_l2_message_call(msg_key: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {
    get_l1_to_l2_msg_oracle(msg_key)
}