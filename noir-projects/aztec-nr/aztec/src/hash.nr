use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET, GENERATOR_INDEX__NULLIFIER},
    hash::{pedersen_hash, silo_nullifier}
};

use dep::protocol_types::hash::{hash_args, hash_args_array, sha256_to_field};

pub fn compute_secret_hash(secret: Field) -> Field {
    // TODO(#1205) This is probably not the right index to use
    pedersen_hash([secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET)
}

pub fn compute_message_hash(
    sender: EthAddress,
    chain_id: Field,
    recipient: AztecAddress,
    version: Field,
    content: Field,
    secret_hash: Field
) -> Field {
    let mut hash_bytes = [0 as u8; 192];
    let sender_bytes = sender.to_field().to_be_bytes(32);
    let chain_id_bytes = chain_id.to_be_bytes(32);
    let recipient_bytes = recipient.to_field().to_be_bytes(32);
    let version_bytes = version.to_be_bytes(32);
    let content_bytes = content.to_be_bytes(32);
    let secret_hash_bytes = secret_hash.to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i] = sender_bytes[i];
        hash_bytes[i + 32] = chain_id_bytes[i];
        hash_bytes[i + 64] = recipient_bytes[i];
        hash_bytes[i + 96] = version_bytes[i];
        hash_bytes[i + 128] = content_bytes[i];
        hash_bytes[i + 160] = secret_hash_bytes[i];
    }

    sha256_to_field(hash_bytes)
}

// The nullifier of a l1 to l2 message is the hash of the message salted with the secret and index of the message hash
// in the L1 to L2 message tree
pub fn compute_message_nullifier(message_hash: Field, secret: Field, leaf_index: Field) -> Field {
    pedersen_hash(
        [message_hash, secret, leaf_index],
        GENERATOR_INDEX__NULLIFIER
    )
}

pub fn compute_siloed_nullifier(address: AztecAddress, nullifier: Field) -> Field {
    silo_nullifier(address, nullifier)
}
