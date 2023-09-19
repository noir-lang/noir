use dep::std::hash::{pedersen_with_separator, sha256};
use dep::aztec::constants_gen::{
    GENERATOR_INDEX__SIGNATURE_PAYLOAD,
    GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET,
};

fn compute_secret_hash(secret: Field) -> Field {
    // TODO(#1205) This is probably not the right index to use
    pedersen_with_separator([secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET)[0]
}

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

    let content_sha256 = sha256(hash_bytes);

    // // Convert the content_sha256 to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (content_sha256[15 - i] as Field) * v;
        low = low + (content_sha256[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    // Abuse that a % p + b % p = (a + b) % p and that low < p
    let content_hash = low + high * v;
    content_hash
}

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

    let content_sha256 = sha256(hash_bytes);

    // // Convert the content_sha256 to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (content_sha256[15 - i] as Field) * v;
        low = low + (content_sha256[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    // Abuse that a % p + b % p = (a + b) % p and that low < p
    let content_hash = low + high * v;
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
    let content_sha256 = sha256(hash_bytes);

    // Convert the content_sha256 to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (content_sha256[15 - i] as Field) * v;
        low = low + (content_sha256[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    // Abuse that a % p + b % p = (a + b) % p and that low < p
    let content = low + high * v;
    content
}