use std::hash::pedersen_hash;

// This circuit proves:
// 1. Sender knows the secret key for the input commitment
// 2. Sender has sufficient balance
// 3. Creates valid output commitments without revealing amounts or addresses

fn main(
    // Private inputs (witness)
    sender_secret: Field,
    sender_balance: Field,
    transfer_amount: Field,
    recipient_address: Field,
    sender_nonce: Field,
    
    // Public inputs
    pub input_commitment: Field,
    pub output_commitment_sender: Field,
    pub output_commitment_recipient: Field,
    pub nullifier: Field,
    pub new_nonce: Field
) {
    // 1. Verify sender owns the input commitment
    // Commitment = Hash(address, balance, nonce)
    let sender_address = pedersen_hash([sender_secret]);
    let computed_commitment = pedersen_hash([sender_address, sender_balance, sender_nonce]);
    assert(computed_commitment == input_commitment);
    
    // 2. Verify sender has sufficient balance
    assert(sender_balance.lt(transfer_amount) == false);
    
    // 3. Compute nullifier to prevent double-spending
    // Nullifier = Hash(secret, nonce)
    let computed_nullifier = pedersen_hash([sender_secret, sender_nonce]);
    assert(computed_nullifier == nullifier);
    
    // 4. Verify output commitments are correctly formed
    let new_sender_balance = sender_balance - transfer_amount;
    let computed_sender_output = pedersen_hash([sender_address, new_sender_balance, new_nonce]);
    assert(computed_sender_output == output_commitment_sender);
    
    // For recipient, we assume they're receiving for the first time or we know their current state
    let zero_field: Field = 0;
    let computed_recipient_output = pedersen_hash([recipient_address, transfer_amount, zero_field]);
    assert(computed_recipient_output == output_commitment_recipient);
}
