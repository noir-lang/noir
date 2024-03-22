use dep::protocol_types::constants::L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH;

// Obtains membership witness (index and sibling path) for a message in the L1 to L2 message tree.
#[oracle(getL1ToL2MembershipWitness)]
fn get_l1_to_l2_membership_witness_oracle(_message_hash: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {}

unconstrained pub fn get_l1_to_l2_membership_witness(message_hash: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {
    get_l1_to_l2_membership_witness_oracle(message_hash)
}
