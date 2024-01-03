// docs:start:mint_public_content_hash_nr
use dep::protocol_types::{
    address::{
        AztecAddress,
        EthAddress,
    },
    hash::sha256_to_field,
};

// Computes a content hash of a deposit/mint_public message.
// Refer TokenPortal.sol for reference on L1.
pub fn get_mint_public_content_hash(owner: AztecAddress, amount: Field, canceller: EthAddress) -> Field {
    
    let mut hash_bytes: [u8; 100] = [0; 100];
    let recipient_bytes = owner.to_field().to_be_bytes(32);
    let amount_bytes = amount.to_be_bytes(32);
    let canceller_bytes = canceller.to_field().to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = recipient_bytes[i];
        hash_bytes[i + 36] = amount_bytes[i];
        hash_bytes[i + 68] = canceller_bytes[i];
    }

    // Function selector: 0xefc2aae6 keccak256('mint_public(bytes32,uint256,address)')
    hash_bytes[0] = 0xef;
    hash_bytes[1] = 0xc2;
    hash_bytes[2] = 0xaa;
    hash_bytes[3] = 0xe6;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
// docs:end:mint_public_content_hash_nr

// docs:start:get_mint_private_content_hash
// Computes a content hash of a deposit/mint_private message.
// Refer TokenPortal.sol for reference on L1.
pub fn get_mint_private_content_hash(secret_hash_for_redeeming_minted_notes: Field, amount: Field, canceller: EthAddress) -> Field {
    let mut hash_bytes: [u8; 100] = [0; 100];
    let secret_hash_bytes = secret_hash_for_redeeming_minted_notes.to_be_bytes(32);
    let amount_bytes = amount.to_be_bytes(32);
    let canceller_bytes = canceller.to_field().to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = secret_hash_bytes[i];
        hash_bytes[i + 36] = amount_bytes[i];
        hash_bytes[i + 68] = canceller_bytes[i];
    }

    // Function selector: 0xf512262e keccak256('mint_private(bytes32,uint256,address)')
    hash_bytes[0] = 0xf5;
    hash_bytes[1] = 0x12;
    hash_bytes[2] = 0x26;
    hash_bytes[3] = 0x2e;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
// docs:end:get_mint_private_content_hash

// docs:start:get_withdraw_content_hash
// Computes a content hash of a withdraw message.
pub fn get_withdraw_content_hash(recipient: EthAddress, amount: Field, callerOnL1: EthAddress) -> Field {
    // Compute the content hash
    // Compute sha256(selector || amount || recipient)
    // then convert to a single field element
    // add that to the l2 to l1 messages
    let mut hash_bytes: [u8; 100] = [0; 100];
    let recipient_bytes = recipient.to_field().to_be_bytes(32);
    let amount_bytes = amount.to_be_bytes(32);
    let callerOnL1_bytes = callerOnL1.to_field().to_be_bytes(32);

    //  0x69328dec, selector for "withdraw(address,uint256,address)"
    hash_bytes[0] = 0x69;
    hash_bytes[1] = 0x32;
    hash_bytes[2] = 0x8d;
    hash_bytes[3] = 0xec;

    for i in 0..32 {
        hash_bytes[i + 4] = recipient_bytes[i];
        hash_bytes[i + 36] = amount_bytes[i];
        hash_bytes[i + 68] = callerOnL1_bytes[i];
    }
    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
// docs:end:get_withdraw_content_hash
