fn main(hashed_message: [u8; 32], pub_key_x: [u8; 32], pub_key_y: [u8; 32], signature: [u8; 64]) {
    let signature = normalize_ecdsa_signature(signature);
    let valid_signature =
        std::ecdsa_secp256r1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
    assert(valid_signature);
}

// Normalizes an ECDSA signature to low-S form for secp256r1
//
// If `s > (order - 1) / 2`, replaces `s` with `order - s`.
fn normalize_ecdsa_signature(signature: [u8; 64]) -> [u8; 64] {
    // secp256r1 curve order
    let order: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xBC, 0xE6, 0xFA, 0xAD, 0xA7, 0x17, 0x9E, 0x84, 0xF3, 0xB9, 0xCA, 0xC2, 0xFC, 0x63,
        0x25, 0x51,
    ];
    // (order - 1) / 2
    let half_order: [u8; 32] = [
        0x7F, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0x00, 0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xDE, 0x73, 0x7D, 0x56, 0xD3, 0x8B, 0xCF, 0x42, 0x79, 0xDC, 0xE5, 0x61, 0x7E, 0x31,
        0x92, 0xA8,
    ];

    // Check if s > half_order (big-endian comparison)
    let mut s_is_high = false;
    let mut stop = false;
    for i in 0..32 {
        if !stop {
            if signature[32 + i] > half_order[i] {
                s_is_high = true;
                stop = true;
            } else if signature[32 + i] < half_order[i] {
                stop = true;
            }
        }
    }

    let mut result = signature;
    if s_is_high {
        // Compute order - s
        let mut borrow: u32 = 0;
        for j in 0..32 {
            let i = 31 - j;
            let a = order[i] as u32;
            let b = signature[32 + i] as u32 + borrow;
            if a >= b {
                result[32 + i] = (a - b) as u8;
                borrow = 0;
            } else {
                result[32 + i] = (256 + a - b) as u8;
                borrow = 1;
            }
        }
    }
    result
}
