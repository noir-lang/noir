// Sha256 example
// 
// Calls Sha256 from the standard library.
//
// The Compiler sees this special function and creates an ACIR gate
//
// The ACIR SHA256 gate is passed to PLONK who should
// know how to create the necessary constraints.
//
// Not yet here: For R1CS, it is more about manipulating arithmetic gates to get performance
// This can be done in ACIR!

fn main(x: Field, result: [u8; 32]) {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
    // The padding is taken care of by the program
    // docs:start:sha256_var
    let digest = sha256_var([x as u8], 1);
    println(digest);
    println(result);
    let current_digest = std::hash::sha256_var([x as u8], 1);
    println(current_digest);
    // docs:end:sha256_var
    assert(digest == result);
}

unconstrained fn build_msg_block_iter<let N: u32>(msg: [u8; N], message_size: u64, mut msg_block: [u8; 64]) -> [u8; 64] {
    let mut msg_byte_ptr: u64 = 0; // Message byte pointer
    // let mut msg_blocks: [[u8; 64]; N] = [[0; 64]; N];
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

fn verify_msg_block<let N: u32>(msg: [u8; N], message_size: u64, msg_block: [u8; 64]) {
    let mut i: u64 = 0; // Message byte pointer
    for k in 0..N {
        if k as u64 < message_size {
            assert_eq(msg_block[i], msg[k]);
            i = i + 1;
            if i == 64 {
                // Enough to hash block
                // TODO: need to accurately constrain each compression step
                // my_h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), my_h);
                i = 0;
            }
        } else {
            // Need to assert over the msg block in the else case as well 
            if N < 64 {
                assert_eq(msg_block[i], 0);
            } else {
                assert_eq(msg_block[i], msg[k]);
            }
        }
    }
}

// Variable size SHA-256 hash
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    let num_blocks = N / 64;
    let mut msg_block: [u8; 64] = [0; 64];
    let mut h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut msg_byte_ptr = 0;

    let mut i: u64 = 0; // Message byte pointer
    let mut my_h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value

    if num_blocks == 0 {
        println("got here");
        unsafe {
            let new_msg_block  = build_msg_block_iter(msg, message_size, msg_block);
            msg_block = new_msg_block;
        }

        // my_h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), my_h);
    }

        // Verify the block we are compressing was appropriately constructed
        for k in 0..N {
            if k as u64 < message_size {
                assert_eq(msg_block[i], msg[k]);
                i = i + 1;
                if i == 64 {
                    i = 0;
                }
            } else {
                // Assert over the msg block in the else case as well 
                if N < 64 {
                    assert_eq(msg_block[i], 0);
                } else {
                    assert_eq(msg_block[i], msg[k]);
                }
            }
        }

    for _ in 0..num_blocks {
        println("got here");
        unsafe {
            let new_msg_block  = build_msg_block_iter(msg, message_size, msg_block);
            msg_block = new_msg_block;
        }
        // Verify the block we are compressing was appropriately constructed
        for k in 0..N {
            if k as u64 < message_size {
                assert_eq(msg_block[i], msg[k]);
                i = i + 1;
                if i == 64 {
                    i = 0;
                }
            } else {
                // Assert over the msg block in the else case as well 
                if N < 64 {
                    assert_eq(msg_block[i], 0);
                } else {
                    assert_eq(msg_block[i], msg[k]);
                }
            }
        }
        // Hash the block
        my_h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), my_h);
    }
    h = my_h;
    println(h);

    msg_byte_ptr = i;

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).    
    msg_block[msg_byte_ptr] = 1 << 7;
    msg_byte_ptr = msg_byte_ptr + 1;

    unsafe {
        let (new_msg_block, _) = fixup_msg_block(msg_block, msg_byte_ptr, h);
        msg_block = new_msg_block;
    }
    println(h);

    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if msg_byte_ptr >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        for _i in 57..64 {
            if msg_byte_ptr <= 63 {
                assert_eq(msg_block[msg_byte_ptr], 0);
                msg_byte_ptr += 1;
            }
        }

        h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), h);

        msg_byte_ptr = 0;
    }

    // println(h);
    // println(msg_block);
    // let old_h = h;
    // h = std::hash::sha256_compression(transformed_msg, h);

    // println(msg_byte_ptr);
    unsafe {
        msg_block = setup_final_msg_block(msg_block, msg_byte_ptr, message_size);
    }
    // println(msg_block);

    let len = 8 * message_size;
    let len_bytes = (len as Field).to_le_bytes(8);
    // In any case, fill blocks up with zeros until the last 64 (i.e. until i = 56).
    for _ in 0..64 {
        if msg_byte_ptr < 56 {
            assert_eq(msg_block[msg_byte_ptr], 0);
            msg_byte_ptr = msg_byte_ptr + 1;
        } else {
            msg_byte_ptr += 8;
        }
    }

    let mut block_idx = 0;
    for i in 56..64 {
        assert_eq(msg_block[63 - block_idx], len_bytes[i - 56]);
        block_idx = block_idx + 1;
    }

    println("about to hash final block");
    println(msg_block);
    println(h);
    hash_final_block(msg_block, h)
}

unconstrained fn fixup_msg_block<let N: u32>(mut msg_block: [u8; 64], mut msg_byte_ptr: u64, mut h: [u32; 8]) -> ([u8; 64], [u32; 8]) {
    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if msg_byte_ptr >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        if msg_byte_ptr < 64 {
            for _ in 57..64 {
                if msg_byte_ptr <= 63 {
                    msg_block[msg_byte_ptr] = 0;
                    msg_byte_ptr += 1;
                }
            }
        }
        h = std::hash::sha256_compression(msg_u8_to_u32(msg_block), h);
        // msg_byte_ptr = 0;
    }
    (msg_block, h)
}

unconstrained fn setup_final_msg_block<let N: u32>(mut msg_block: [u8; 64], mut msg_byte_ptr: u64, message_size: u64) -> [u8; 64] {
    let len = 8 * message_size;
    let len_bytes = (len as Field).to_le_bytes(8);
    for _i in 0..64 {
        // In any case, fill blocks up with zeros until the last 64 (i.e. until i = 56).
        if msg_byte_ptr < 56 {
            msg_block[msg_byte_ptr] = 0;
            msg_byte_ptr = msg_byte_ptr + 1;
        } else if msg_byte_ptr < 64 {
            for j in 0..8 {
                msg_block[63 - j] = len_bytes[j];
            }
            msg_byte_ptr += 8;
        }
    }
    msg_block
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