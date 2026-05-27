use dep::std;

fn main(
    device_secret: Field,           // Private input
    commitment: pub Field,         // Public: Poseidon2(device_secret)
    challenge: pub Field,          // Public: Prevent replay/ensure freshness
    user_id_hash: pub Field        // Public: Anchors to specific user
) -> pub Field {                   // Return Nullifier
    // 1. Check commitment
    let computed_commitment = std::hash::poseidon::bn254::hash_1([device_secret]);
    assert(computed_commitment == commitment);

    // 2. Generate Nullifier to prevent proof reuse for the same challenge
    let nullifier = std::hash::poseidon::bn254::hash_2([device_secret, challenge]);
    
    nullifier
}

#[test]
fn test_recovery() {
    let secret = 12345;
    let challenge = 67890;
    let user_id = 1;
    let commitment = std::hash::poseidon::bn254::hash_1([secret]);
    
    let nullifier = main(secret, commitment, challenge, user_id);
    assert(nullifier != 0);
}
