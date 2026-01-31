use std::hash::pedersen_hash;

// This circuit proves valid minting without revealing the recipient
fn main(
    // Private inputs
    recipient_secret: Field,
    mint_amount: Field,
    nonce: Field,
    
    // Public inputs
    pub output_commitment: Field,
    pub mint_request_id: Field
) {
    // 1. Compute recipient address from secret
    let recipient_address = pedersen_hash([recipient_secret]);
    
    // 2. Verify output commitment
    let computed_commitment = pedersen_hash([recipient_address, mint_amount, nonce]);
    assert(computed_commitment == output_commitment);
    
    // 3. Ensure mint amount is positive
    let zero_field: Field = 0;
    assert(mint_amount != zero_field);
}
