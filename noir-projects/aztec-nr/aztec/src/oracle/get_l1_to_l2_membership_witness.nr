use dep::protocol_types::{address::AztecAddress};

global L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH: u64 = 17;

// Obtains membership witness (index and sibling path) for a message in the L1 to L2 message tree.
#[oracle(getL1ToL2MembershipWitness)]
fn get_l1_to_l2_membership_witness_oracle(
    _contract_address: AztecAddress,
    _message_hash: Field,
    _secret: Field
) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {}

unconstrained pub fn get_l1_to_l2_membership_witness(
    contract_address: AztecAddress,
    message_hash: Field,
    secret: Field
) -> [Field; L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH] {
    get_l1_to_l2_membership_witness_oracle(contract_address, message_hash, secret)
}
