use dep::std;

fn main(
    device_secret: Field,           // Private input
    commitment: pub Field,         // Public: demo commitment for device_secret
    challenge: pub Field,          // Public: Prevent replay/ensure freshness
    user_id_hash: pub Field        // Public: Anchors to specific user
) -> pub Field {                   // Return Nullifier
    // 1. Check commitment
    let computed_commitment = device_secret * device_secret + user_id_hash;
    assert(computed_commitment == commitment);

    // 2. Generate Nullifier to prevent proof reuse for the same challenge
    let nullifier = device_secret * challenge + user_id_hash;
    
    nullifier
}

#[test]
fn test_recovery() {
    let secret = 12345;
    let challenge = 67890;
    let user_id = 1;
    let commitment = secret * secret + user_id;
    
    let nullifier = main(secret, commitment, challenge, user_id);
    assert(nullifier != 0);
}
