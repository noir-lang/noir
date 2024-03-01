use dep::protocol_types::constants::L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH;

// Checks if a msg is within the l1ToL2Msg tree
#[oracle(getL1ToL2MembershipWitness)]
fn get_l1_to_l2_membership_witness_oracle(_entry_key: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {}

unconstrained pub fn get_l1_to_l2_membership_witness(entry_key: Field) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {
    get_l1_to_l2_membership_witness_oracle(entry_key)
}
