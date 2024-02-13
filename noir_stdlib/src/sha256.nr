// Implementation of SHA-256 mapping a byte array of variable length to
// 32 bytes.

// Convert 64-byte array to array of 16 u32s
fn msg_u8_to_u32(msg: [u8; 64]) -> [u32; 16] {
    let mut msg32: [u32; 16] = [0; 16];

    for i in 0..16 {
        for j in 0..4 {
            msg32[15 - i] = (msg32[15 - i] << 8) + msg[64 - 4*(i + 1) + j] as u32;
        }
    }

    msg32
}
// SHA-256 hash function
pub fn digest<N>(msg: [u8; N]) -> [u8; 32] {
    let mut msg_block: [u8; 64] = [0; 64];
    let mut h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut out_h: [u8; 32] = [0; 32]; // Digest as sequence of bytes
    let mut i: u64 = 0; // Message byte pointer
    for k in 0..N {
        // Populate msg_block
        msg_block[i as Field] = msg[k];
        i = i + 1;
        if i == 64 {
            // Enough to hash block
            h = crate::hash::sha256_compression(msg_u8_to_u32(msg_block), h);

            i = 0;
        }
    }
    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).
    msg_block[i as Field] = 1 << 7;
    i = i + 1;
    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if i >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        if i < 64 {
            for _i in 57..64 {
                if i <= 63 {
                    msg_block[i as Field] = 0;
                    i += 1;
                }
            }
        }
        h = crate::hash::sha256_compression(msg_u8_to_u32(msg_block), h);

        i = 0;
    }

    for _i in 0..64 {
        // In any case, fill blocks up with zeros until the last 64 (i.e. until i = 56).
        if i < 56 {
            msg_block[i as Field] = 0;
            i = i + 1;
        } else if i < 64 {
            let mut len = 8 * msg.len() as u64;
            for j in 0..8 {
                msg_block[63 - j] = len as u8;
                len >>= 8;
            }
            i += 8;
        }
    }
    // Hash final padded block
    h = crate::hash::sha256_compression(msg_u8_to_u32(msg_block), h);

    // Return final hash as byte array
    for j in 0..8 {
        for k in 0..4 {
            out_h[31 - 4*j - k] = h[7 - j] as u8;
            h[7-j] >>= 8;
        }
    }

    out_h
}
