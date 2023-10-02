use dep::std::hash::pedersen_with_separator;
// docs:start:mint_public_content_hash_nr
use dep::aztec::hash::{sha256_to_field};

// Computes a content hash of a deposit/mint_public message.
// Refer TokenPortal.sol for reference on L1.
fn get_mint_public_content_hash(owner_address: Field, amount: Field, canceller: Field) -> Field {
    
    let mut hash_bytes: [u8; 100] = [0; 100];
    let amount_bytes = amount.to_be_bytes(32);
    let recipient_bytes = owner_address.to_be_bytes(32);
    let canceller_bytes = canceller.to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = amount_bytes[i];
        hash_bytes[i + 36] = recipient_bytes[i];
        hash_bytes[i + 68] = canceller_bytes[i];
    }

    // Function selector: 0x63c9440d keccak256('mint_public(uint256,bytes32,address)')
    hash_bytes[0] = 0x63;
    hash_bytes[1] = 0xc9;
    hash_bytes[2] = 0x44;
    hash_bytes[3] = 0x0d;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
// docs:end:mint_public_content_hash_nr

// Computes a content hash of a deposit/mint_private message.
// Refer TokenPortal.sol for reference on L1.
fn get_mint_private_content_hash(amount: Field, secret_hash_for_redeeming_minted_notes: Field, canceller: Field) -> Field {
    let mut hash_bytes: [u8; 100] = [0; 100];
    let amount_bytes = amount.to_be_bytes(32);
    let secret_hash_bytes = secret_hash_for_redeeming_minted_notes.to_be_bytes(32);
    let canceller_bytes = canceller.to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = amount_bytes[i];
        hash_bytes[i + 36] = secret_hash_bytes[i];
        hash_bytes[i + 68] = canceller_bytes[i];
    }

    // Function selector: 0x25d46b0f keccak256('mint_private(uint256,bytes32,address)')
    hash_bytes[0] = 0x25;
    hash_bytes[1] = 0xd4;
    hash_bytes[2] = 0x6b;
    hash_bytes[3] = 0x0f;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}

// Computes a content hash of a withdraw message.
fn get_withdraw_content_hash(recipient: Field, amount: Field, callerOnL1: Field) -> Field {
    // Compute the content hash
    // Compute sha256(selector || amount || recipient)
    // then convert to a single field element
    // add that to the l2 to l1 messages
    let mut hash_bytes: [u8; 100] = [0; 100];
    let amount_bytes = amount.to_be_bytes(32);
    let recipient_bytes = recipient.to_be_bytes(32);
    let callerOnL1_bytes = callerOnL1.to_be_bytes(32);

    //  0xb460af94, selector for "withdraw(uint256,address,address)"
    hash_bytes[0] = 0xb4;
    hash_bytes[1] = 0x60;
    hash_bytes[2] = 0xaf;
    hash_bytes[3] = 0x94;

    for i in 0..32 {
        hash_bytes[i + 4] = amount_bytes[i];
        hash_bytes[i + 36] = recipient_bytes[i];
        hash_bytes[i + 68] = callerOnL1_bytes[i];
    }
    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}