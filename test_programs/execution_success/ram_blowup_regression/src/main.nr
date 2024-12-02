global TX_EFFECTS_HASH_INPUT_FIELDS: u32 = 256;

// Convert a 32 byte array to a field element by truncating the final byte
pub fn field_from_bytes_32_trunc(bytes32: [u8; 32]) -> Field {
    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..15 {
        // covers bytes 16..30 (31 is truncated and ignored)
        low = low + (bytes32[15 + 15 - i] as Field) * v;
        v = v * 256;
        // covers bytes 0..14
        high = high + (bytes32[14 - i] as Field) * v;
    }
    // covers byte 15
    low = low + (bytes32[15] as Field) * v;

    low + high * v
}

pub fn sha256_to_field<let N: u32>(bytes_to_hash: [u8; N]) -> Field {
    let sha256_hashed = std::hash::sha256(bytes_to_hash);
    let hash_in_a_field = field_from_bytes_32_trunc(sha256_hashed);

    hash_in_a_field
}

fn main(tx_effects_hash_input: [Field; TX_EFFECTS_HASH_INPUT_FIELDS]) -> pub Field {
    let mut hash_input_flattened = [0; TX_EFFECTS_HASH_INPUT_FIELDS * 32];
    for offset in 0..TX_EFFECTS_HASH_INPUT_FIELDS {
        let input_as_bytes: [u8; 32] = tx_effects_hash_input[offset].to_be_bytes();
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = sha256_to_field(hash_input_flattened);
    sha_digest
}
