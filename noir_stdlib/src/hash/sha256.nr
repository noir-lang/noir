use crate::runtime::is_unconstrained;

// Implementation of SHA-256 mapping a byte array of variable length to
// 32 bytes.

// Deprecated in favour of `sha256_var`
// docs:start:sha256
pub fn sha256<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:sha256
{
    crate::sha256::digest(input)
}

#[foreign(sha256_compression)]
pub fn sha256_compression(_input: [u32; 16], _state: [u32; 8]) -> [u32; 8] {}

// SHA-256 hash function
#[no_predicates]
pub fn digest<let N: u32>(msg: [u8; N]) -> [u8; 32] {
    sha256_var(msg, N as u64)
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

unconstrained fn build_msg_block_iter<let N: u32>(msg: [u8; N], message_size: u32, msg_start: u32) -> ([u8; 64], u32) {
    let mut msg_block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    // We insert `BLOCK_SIZE` bytes (or up to the end of the message)
    let block_input = if msg_start + BLOCK_SIZE > message_size {
        if message_size < msg_start {
            // This function is sometimes called with `msg_start` past the end of the message.
            // In this case we return an empty block and zero pointer to signal that the result should be ignored.
            0
        } else {
            message_size - msg_start
        }
    } else {
        BLOCK_SIZE
    };
    for k in 0..block_input {
        msg_block[k] = msg[msg_start + k];
    }
    (msg_block, block_input)
}

// Verify the block we are compressing was appropriately constructed
fn verify_msg_block<let N: u32>(
    msg: [u8; N],
    message_size: u32,
    msg_block: [u8; 64],
    msg_start: u32
) -> u32 {
    let mut msg_byte_ptr: u32 = 0; // Message byte pointer
    let mut msg_end = msg_start + BLOCK_SIZE;
    if msg_end > N {
        msg_end = N;
    }

    for k in msg_start..msg_end {
        if k < message_size {
            assert_eq(msg_block[msg_byte_ptr], msg[k]);
            msg_byte_ptr = msg_byte_ptr + 1;
        }
    }

    msg_byte_ptr
}

global BLOCK_SIZE = 64;
global ZERO = 0;

// Variable size SHA-256 hash
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    let message_size = message_size as u32;
    let num_blocks = N / BLOCK_SIZE;
    let mut msg_block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let mut h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut msg_byte_ptr = 0; // Pointer into msg_block

    for i in 0..num_blocks {
        let msg_start = BLOCK_SIZE * i;
        let (new_msg_block, new_msg_byte_ptr) = unsafe {
            build_msg_block_iter(msg, message_size, msg_start)
        };
        if msg_start < message_size {
            msg_block = new_msg_block;
        }

        if !is_unconstrained() {
            // Verify the block we are compressing was appropriately constructed
            let new_msg_byte_ptr = verify_msg_block(msg, message_size, msg_block, msg_start);
            if msg_start < message_size {
                msg_byte_ptr = new_msg_byte_ptr;
            }
        } else if msg_start < message_size {
            msg_byte_ptr = new_msg_byte_ptr;
        }

        // If the block is filled, compress it.
        // An un-filled block is handled after this loop.
        if msg_byte_ptr == BLOCK_SIZE {
            h = sha256_compression(msg_u8_to_u32(msg_block), h);
        }
    }

    let modulo = N % BLOCK_SIZE;
    // Handle setup of the final msg block.
    // This case is only hit if the msg is less than the block size,
    // or our message cannot be evenly split into blocks.
    if modulo != 0 {
        let msg_start = BLOCK_SIZE * num_blocks;
        let (new_msg_block, new_msg_byte_ptr) = unsafe {
            build_msg_block_iter(msg, message_size, msg_start)
        };

        if msg_start < message_size {
            msg_block = new_msg_block;
        }

        if !is_unconstrained() {
            let new_msg_byte_ptr = verify_msg_block(msg, message_size, msg_block, msg_start);
            if msg_start < message_size {
                msg_byte_ptr = new_msg_byte_ptr;
            }
        } else if msg_start < message_size {
            msg_byte_ptr = new_msg_byte_ptr;
        }
    }

    if msg_byte_ptr == BLOCK_SIZE {
        msg_byte_ptr = 0;
    }

    // This variable is used to get around the compiler under-constrained check giving a warning.
    // We want to check against a constant zero, but if it does not come from the circuit inputs
    // or return values the compiler check will issue a warning.
    let zero = msg_block[0] - msg_block[0];

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).
    msg_block[msg_byte_ptr] = 1 << 7;
    let last_block = msg_block;
    msg_byte_ptr = msg_byte_ptr + 1;

    unsafe {
        let (new_msg_block, new_msg_byte_ptr) = pad_msg_block(msg_block, msg_byte_ptr);
        msg_block = new_msg_block;
        if crate::runtime::is_unconstrained() {
            msg_byte_ptr = new_msg_byte_ptr;
        }
    }

    if !crate::runtime::is_unconstrained() {
        for i in 0..BLOCK_SIZE {
            assert_eq(msg_block[i], last_block[i]);
        }

        // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
        // the 1 and 0s fill up the current block, which we then compress accordingly.
        // Not enough bits (64) to store length. Fill up with zeros.
        for _i in 57..BLOCK_SIZE {
            if msg_byte_ptr <= 63 & msg_byte_ptr >= 57 {
                assert_eq(msg_block[msg_byte_ptr], zero);
                msg_byte_ptr += 1;
            }
        }
    }

    if msg_byte_ptr >= 57 {
        h = sha256_compression(msg_u8_to_u32(msg_block), h);

        msg_byte_ptr = 0;
    }

    msg_block = unsafe {
        attach_len_to_msg_block(msg_block, msg_byte_ptr, message_size)
    };

    if !crate::runtime::is_unconstrained() {
        for i in 0..56 {
            let predicate = (i < msg_byte_ptr) as u8;
            let expected_byte = predicate * last_block[i];
            assert_eq(msg_block[i], expected_byte);
        }

        // We verify the message length was inserted correctly by reversing the byte decomposition.
        let len = 8 * message_size;
        let mut reconstructed_len: Field = 0;
        for i in 56..64 {
            reconstructed_len = 256 * reconstructed_len + msg_block[i] as Field;
        }
        assert_eq(reconstructed_len, len as Field);
    }

    hash_final_block(msg_block, h)
}

unconstrained fn pad_msg_block(
    mut msg_block: [u8; 64],
    mut msg_byte_ptr: u32
) -> ([u8; BLOCK_SIZE], u32) {
    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if msg_byte_ptr >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        for i in msg_byte_ptr..BLOCK_SIZE {
            msg_block[i] = 0;
        }
        (msg_block, BLOCK_SIZE)
    } else {
        (msg_block, msg_byte_ptr)
    }
}

unconstrained fn attach_len_to_msg_block(mut msg_block: [u8; BLOCK_SIZE], msg_byte_ptr: u32, message_size: u32) -> [u8; BLOCK_SIZE] {
    // We assume that `msg_byte_ptr` is less than 57 because if not then it is reset to zero before calling this function.
    // In any case, fill blocks up with zeros until the last 64 (i.e. until msg_byte_ptr = 56).

    for i in msg_byte_ptr..56 {
        msg_block[i] = 0;
    }

    let len = 8 * message_size;
    let len_bytes: [u8; 8] = (len as Field).to_be_bytes();
    for i in 0..8 {
        msg_block[56 + i] = len_bytes[i];
    }
    msg_block
}

fn hash_final_block(msg_block: [u8; BLOCK_SIZE], mut state: [u32; 8]) -> [u8; 32] {
    let mut out_h: [u8; 32] = [0; 32]; // Digest as sequence of bytes

    // Hash final padded block
    state = sha256_compression(msg_u8_to_u32(msg_block), state);

    // Return final hash as byte array
    for j in 0..8 {
        let h_bytes: [u8; 4] = (state[7 - j] as Field).to_le_bytes();
        for k in 0..4 {
            out_h[31 - 4*j - k] = h_bytes[k];
        }
    }

    out_h
}

mod tests {
    use super::sha256_var;

    #[test]
    fn smoke_test() {
        let input = [0xbd];
        let result = [
            0x68, 0x32, 0x57, 0x20, 0xaa, 0xbd, 0x7c, 0x82, 0xf3, 0x0f, 0x55, 0x4b, 0x31, 0x3d, 0x05, 0x70, 0xc9, 0x5a, 0xcc, 0xbb, 0x7d, 0xc4, 0xb5, 0xaa, 0xe1, 0x12, 0x04, 0xc0, 0x8f, 0xfe, 0x73, 0x2b
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_just_over_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117, 101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99, 111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116
        ];
        let result = [
            91, 122, 146, 93, 52, 109, 133, 148, 171, 61, 156, 70, 189, 238, 153, 7, 222, 184, 94, 24, 65, 114, 192, 244, 207, 199, 87, 232, 192, 224, 171, 207
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_multiple_over_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117, 101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99, 111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116, 61, 117, 115, 45, 97, 115, 99, 105, 105, 13, 10, 109, 105, 109, 101, 45, 118, 101, 114, 115, 105, 111, 110, 58, 49, 46, 48, 32, 40, 77, 97, 99, 32, 79, 83, 32, 88, 32, 77, 97, 105, 108, 32, 49, 54, 46, 48, 32, 92, 40, 51, 55, 51, 49, 46, 53, 48, 48, 46, 50, 51, 49, 92, 41, 41, 13, 10, 115, 117, 98, 106, 101, 99, 116, 58, 72, 101, 108, 108, 111, 13, 10, 109, 101, 115, 115, 97, 103, 101, 45, 105, 100, 58, 60, 56, 70, 56, 49, 57, 68, 51, 50, 45, 66, 54, 65, 67, 45, 52, 56, 57, 68, 45, 57, 55, 55, 70, 45, 52, 51, 56, 66, 66, 67, 52, 67, 65, 66, 50, 55, 64, 109, 101, 46, 99, 111, 109, 62, 13, 10, 100, 97, 116, 101, 58, 83, 97, 116, 44, 32, 50, 54, 32, 65, 117, 103, 32, 50, 48, 50, 51, 32, 49, 50, 58, 50, 53, 58, 50, 50, 32, 43, 48, 52, 48, 48, 13, 10, 116, 111, 58, 122, 107, 101, 119, 116, 101, 115, 116, 64, 103, 109, 97, 105, 108, 46, 99, 111, 109, 13, 10, 100, 107, 105, 109, 45, 115, 105, 103, 110, 97, 116, 117, 114, 101, 58, 118, 61, 49, 59, 32, 97, 61, 114, 115, 97, 45, 115, 104, 97, 50, 53, 54, 59, 32, 99, 61, 114, 101, 108, 97, 120, 101, 100, 47, 114, 101, 108, 97, 120, 101, 100, 59, 32, 100, 61, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 59, 32, 115, 61, 49, 97, 49, 104, 97, 105, 59, 32, 116, 61, 49, 54, 57, 51, 48, 51, 56, 51, 51, 55, 59, 32, 98, 104, 61, 55, 120, 81, 77, 68, 117, 111, 86, 86, 85, 52, 109, 48, 87, 48, 87, 82, 86, 83, 114, 86, 88, 77, 101, 71, 83, 73, 65, 83, 115, 110, 117, 99, 75, 57, 100, 74, 115, 114, 99, 43, 118, 85, 61, 59, 32, 104, 61, 102, 114, 111, 109, 58, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 77, 105, 109, 101, 45, 86, 101, 114, 115, 105, 111, 110, 58, 83, 117, 98, 106, 101, 99
        ];
        let result = [
            116, 90, 151, 31, 78, 22, 138, 180, 211, 189, 69, 76, 227, 200, 155, 29, 59, 123, 154, 60, 47, 153, 203, 129, 157, 251, 48, 2, 79, 11, 65, 47
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_just_under_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117, 101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99, 111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 59
        ];
        let result = [
            143, 140, 76, 173, 222, 123, 102, 68, 70, 149, 207, 43, 39, 61, 34, 79, 216, 252, 213, 165, 74, 16, 110, 74, 29, 64, 138, 167, 30, 1, 9, 119
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_big_not_block_multiple() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117, 101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99, 111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116, 61, 117, 115, 45, 97, 115, 99, 105, 105, 13, 10, 109, 105, 109, 101, 45, 118, 101, 114, 115, 105, 111, 110, 58, 49, 46, 48, 32, 40, 77, 97, 99, 32, 79, 83, 32, 88, 32, 77, 97, 105, 108, 32, 49, 54, 46, 48, 32, 92, 40, 51, 55, 51, 49, 46, 53, 48, 48, 46, 50, 51, 49, 92, 41, 41, 13, 10, 115, 117, 98, 106, 101, 99, 116, 58, 72, 101, 108, 108, 111, 13, 10, 109, 101, 115, 115, 97, 103, 101, 45, 105, 100, 58, 60, 56, 70, 56, 49, 57, 68, 51, 50, 45, 66, 54, 65, 67, 45, 52, 56, 57, 68, 45, 57, 55, 55, 70, 45, 52, 51, 56, 66, 66, 67, 52, 67, 65, 66, 50, 55, 64, 109, 101, 46, 99, 111, 109, 62, 13, 10, 100, 97, 116, 101, 58, 83, 97, 116, 44, 32, 50, 54, 32, 65, 117, 103, 32, 50, 48, 50, 51, 32, 49, 50, 58, 50, 53, 58, 50, 50, 32, 43, 48, 52, 48, 48, 13, 10, 116, 111, 58, 122, 107, 101, 119, 116, 101, 115, 116, 64, 103, 109, 97, 105, 108, 46, 99, 111, 109, 13, 10, 100, 107, 105, 109, 45, 115, 105, 103, 110, 97, 116, 117, 114, 101, 58, 118, 61, 49, 59, 32, 97, 61, 114, 115, 97, 45, 115, 104, 97, 50, 53, 54, 59, 32, 99, 61, 114, 101, 108, 97, 120, 101, 100, 47, 114, 101, 108, 97, 120, 101, 100, 59, 32, 100, 61, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 59, 32, 115, 61, 49, 97, 49, 104, 97, 105, 59, 32, 116, 61, 49, 54, 57, 51, 48, 51, 56, 51, 51, 55, 59, 32, 98, 104, 61, 55, 120, 81, 77, 68, 117, 111, 86, 86, 85, 52, 109, 48, 87, 48, 87, 82, 86, 83, 114, 86, 88, 77, 101, 71, 83, 73, 65, 83, 115, 110, 117, 99, 75, 57, 100, 74, 115, 114, 99, 43, 118, 85, 61, 59, 32, 104, 61, 102, 114, 111, 109, 58, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 77, 105, 109, 101, 45, 86, 101, 114, 115, 105, 111, 110, 58, 83, 117, 98, 106, 101, 99, 116, 58, 77, 101, 115, 115, 97, 103, 101, 45, 73, 100, 58, 68, 97, 116, 101, 58, 116, 111, 59, 32, 98, 61
        ];
        let result = [
            112, 144, 73, 182, 208, 98, 9, 238, 54, 229, 61, 145, 222, 17, 72, 62, 148, 222, 186, 55, 192, 82, 220, 35, 66, 47, 193, 200, 22, 38, 26, 186
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_big_with_padding() {
        let input = [
            48, 130, 1, 37, 2, 1, 0, 48, 11, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 48, 130, 1, 17, 48, 37, 2, 1, 1, 4, 32, 176, 223, 31, 133, 108, 84, 158, 102, 70, 11, 165, 175, 196, 12, 201, 130, 25, 131, 46, 125, 156, 194, 28, 23, 55, 133, 157, 164, 135, 136, 220, 78, 48, 37, 2, 1, 2, 4, 32, 190, 82, 180, 235, 222, 33, 79, 50, 152, 136, 142, 35, 116, 224, 6, 242, 156, 141, 128, 248, 10, 61, 98, 86, 248, 45, 207, 210, 90, 232, 175, 38, 48, 37, 2, 1, 3, 4, 32, 0, 194, 104, 108, 237, 246, 97, 230, 116, 198, 69, 110, 26, 87, 17, 89, 110, 199, 108, 250, 36, 21, 39, 87, 110, 102, 250, 213, 174, 131, 171, 174, 48, 37, 2, 1, 11, 4, 32, 136, 155, 87, 144, 111, 15, 152, 127, 85, 25, 154, 81, 20, 58, 51, 75, 193, 116, 234, 0, 60, 30, 29, 30, 183, 141, 72, 247, 255, 203, 100, 124, 48, 37, 2, 1, 12, 4, 32, 41, 234, 106, 78, 31, 11, 114, 137, 237, 17, 92, 71, 134, 47, 62, 78, 189, 233, 201, 214, 53, 4, 47, 189, 201, 133, 6, 121, 34, 131, 64, 142, 48, 37, 2, 1, 13, 4, 32, 91, 222, 210, 193, 62, 222, 104, 82, 36, 41, 138, 253, 70, 15, 148, 208, 156, 45, 105, 171, 241, 195, 185, 43, 217, 162, 146, 201, 222, 89, 238, 38, 48, 37, 2, 1, 14, 4, 32, 76, 123, 216, 13, 51, 227, 72, 245, 59, 193, 238, 166, 103, 49, 23, 164, 171, 188, 194, 197, 156, 187, 249, 28, 198, 95, 69, 15, 182, 56, 54, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
        let result = [
            32, 85, 108, 174, 127, 112, 178, 182, 8, 43, 134, 123, 192, 211, 131, 66, 184, 240, 212, 181, 240, 180, 106, 195, 24, 117, 54, 129, 19, 10, 250, 53
        ];
        let message_size = 297;
        assert_eq(sha256_var(input, message_size), result);
    }

    #[test]
    fn msg_big_no_padding() {
        let input = [
            48, 130, 1, 37, 2, 1, 0, 48, 11, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 48, 130, 1, 17, 48, 37, 2, 1, 1, 4, 32, 176, 223, 31, 133, 108, 84, 158, 102, 70, 11, 165, 175, 196, 12, 201, 130, 25, 131, 46, 125, 156, 194, 28, 23, 55, 133, 157, 164, 135, 136, 220, 78, 48, 37, 2, 1, 2, 4, 32, 190, 82, 180, 235, 222, 33, 79, 50, 152, 136, 142, 35, 116, 224, 6, 242, 156, 141, 128, 248, 10, 61, 98, 86, 248, 45, 207, 210, 90, 232, 175, 38, 48, 37, 2, 1, 3, 4, 32, 0, 194, 104, 108, 237, 246, 97, 230, 116, 198, 69, 110, 26, 87, 17, 89, 110, 199, 108, 250, 36, 21, 39, 87, 110, 102, 250, 213, 174, 131, 171, 174, 48, 37, 2, 1, 11, 4, 32, 136, 155, 87, 144, 111, 15, 152, 127, 85, 25, 154, 81, 20, 58, 51, 75, 193, 116, 234, 0, 60, 30, 29, 30, 183, 141, 72, 247, 255, 203, 100, 124, 48, 37, 2, 1, 12, 4, 32, 41, 234, 106, 78, 31, 11, 114, 137, 237, 17, 92, 71, 134, 47, 62, 78, 189, 233, 201, 214, 53, 4, 47, 189, 201, 133, 6, 121, 34, 131, 64, 142, 48, 37, 2, 1, 13, 4, 32, 91, 222, 210, 193, 62, 222, 104, 82, 36, 41, 138, 253, 70, 15, 148, 208, 156, 45, 105, 171, 241, 195, 185, 43, 217, 162, 146, 201, 222, 89, 238, 38, 48, 37, 2, 1, 14, 4, 32, 76, 123, 216, 13, 51, 227, 72, 245, 59, 193, 238, 166, 103, 49, 23, 164, 171, 188, 194, 197, 156, 187, 249, 28, 198, 95, 69, 15, 182, 56, 54, 38
        ];
        let result = [
            32, 85, 108, 174, 127, 112, 178, 182, 8, 43, 134, 123, 192, 211, 131, 66, 184, 240, 212, 181, 240, 180, 106, 195, 24, 117, 54, 129, 19, 10, 250, 53
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }
}
