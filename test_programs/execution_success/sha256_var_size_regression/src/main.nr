global NUM_HASHES = 2;

fn main(foo: [u8; 95], toggle: bool, enable: [bool; NUM_HASHES]) {
// -> pub [[u8; 32]; NUM_HASHES] {
    // let mut result = [[0; 32]; NUM_HASHES];
    // for i in 0..NUM_HASHES {
    //     let size: Field = 93 + toggle as Field * 2;
    //     if enable[i] {
    //         result[i] = std::sha256::sha256_var(foo, size as u64);
    //     }
    // }

    // result

    let size: Field = 93 + toggle as Field * 2;

    let new_hash = std::hash::sha256_var(foo, size as u64);
    println(f"new_hash: {new_hash}");

    let hash = old_sha256_var(foo, size as u64);
    println(f"hash: {hash}");

    let const_hash = std::hash::sha256_var(foo, 93);
    println(f"const_hash: {const_hash}");

    assert_eq(new_hash, const_hash);
    assert_eq(new_hash, hash);
}

unconstrained fn build_msg_block_iter<let N: u32>(msg: [u8; N], message_size: u64, mut msg_block: [u8; 64]) -> [u8; 64] {
    let mut msg_byte_ptr: u64 = 0; // Message byte pointer
    for k in 0..N {
        if k as u64 < message_size {
            msg_block[msg_byte_ptr] = msg[k];
            msg_byte_ptr = msg_byte_ptr + 1;

            if msg_byte_ptr == 64 {
                msg_byte_ptr = 0;
            }
        }
    }
    msg_block
}

// Variable size SHA-256 hash
pub fn old_sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    let mut msg_block: [u8; 64] = [0; 64];
    let mut h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut i: u64 = 0; // Message byte pointer
    for k in 0..N {
        if k as u64 < message_size {
            // Populate msg_block
            msg_block[i] = msg[k];
            i = i + 1;
            if i == 64 {
                // Enough to hash block
                h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), h);

                i = 0;
            }
        }
    }

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).    
    msg_block[i] = 1 << 7;
    i = i + 1;

    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if i >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        // if i < 64 {
        //     for _ in 57..64 {
        //         if i <= 63 {
        //             msg_block[i] = 0;
        //             i += 1;
        //         }
        //     }
        // }
        for _ in 57..64 {
            if i <= 63 {
                msg_block[i] = 0;
                i += 1;
            }
        }

        h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), h);
        i = 0;
    }

    let len = 8 * message_size;
    let len_bytes = (len as Field).to_le_bytes(8);
    for _ in 0..64 {
        // In any case, fill blocks up with zeros until the last 64 (i.e. until msg_byte_ptr = 56).
        if i < 56 {
            msg_block[i] = 0;
            i = i + 1;
        } else if i < 64 {
            for j in 0..8 {
                msg_block[63 - j] = len_bytes[j];
            }
            i += 8;
        }
    }

    hash_final_block(msg_block, h)
}

// Convert 64-byte array to array of 16 u32s
fn msg_u8_to_u32(msg: [u8; 64]) -> [u32; 16] {
    let mut msg32: [u32; 16] = [0; 16];

    for i in 0..16 {
        let mut msg_field: Field = 0;
        for j in 0..4 {
            msg_field = msg_field * 256 + msg[64 - 4*(i + 1) + j] as Field;
        }
        msg32[15 - i] = msg_field as u32;
    }

    msg32
}

fn hash_final_block(msg_block: [u8; 64], mut state: [u32; 8]) -> [u8; 32] {
    let mut out_h: [u8; 32] = [0; 32]; // Digest as sequence of bytes

    // Hash final padded block
    state = std::hash::sha256_compression(msg_u8_to_u32(msg_block), state);

    // Return final hash as byte array
    for j in 0..8 {
        let h_bytes = (state[7 - j] as Field).to_le_bytes(4);
        for k in 0..4 {
            out_h[31 - 4*j - k] = h_bytes[k];
        }
    }

    out_h
}