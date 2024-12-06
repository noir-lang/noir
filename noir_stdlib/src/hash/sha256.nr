use crate::runtime::is_unconstrained;

// Implementation of SHA-256 mapping a byte array of variable length to
// 32 bytes.

// A message block is up to 64 bytes taken from the input.
global BLOCK_SIZE: u32 = 64;

// The first index in the block where the 8 byte message size will be written.
global MSG_SIZE_PTR: u32 = 56;

// Size of the message block when packed as 4-byte integer array.
global INT_BLOCK_SIZE: u32 = 16;

// A `u32` integer consists of 4 bytes.
global INT_SIZE: u32 = 4;

// Index of the integer in the `INT_BLOCK` where the length is written.
global INT_SIZE_PTR: u32 = MSG_SIZE_PTR / INT_SIZE;

// Magic numbers for bit shifting.
// Works with actual bit shifting as well as the compiler turns them into * and /
// but circuit execution appears to be 10% faster this way.
global TWO_POW_8: u32 = 256;
global TWO_POW_16: u32 = TWO_POW_8 * 256;
global TWO_POW_24: u32 = TWO_POW_16 * 256;
global TWO_POW_32: u64 = TWO_POW_24 as u64 * 256;

// Index of a byte in a 64 byte block; ie. 0..=63
type BLOCK_BYTE_PTR = u32;

// The foreign function to compress blocks works on 16 pieces of 4-byte integers, instead of 64 bytes.
type INT_BLOCK = [u32; INT_BLOCK_SIZE];

// A message block is a slice of the original message of a fixed size,
// potentially padded with zeros, with neighbouring 4 bytes packed into integers.
type MSG_BLOCK = INT_BLOCK;

// The hash is 32 bytes.
type HASH = [u8; 32];

// The state accumulates the blocks.
// Its overall size is the same as the `HASH`.
type STATE = [u32; 8];

// Deprecated in favour of `sha256_var`
// docs:start:sha256
pub fn sha256<let N: u32>(input: [u8; N]) -> HASH
// docs:end:sha256
{
    digest(input)
}

#[foreign(sha256_compression)]
pub fn sha256_compression(_input: INT_BLOCK, _state: STATE) -> STATE {}

// SHA-256 hash function
#[no_predicates]
pub fn digest<let N: u32>(msg: [u8; N]) -> HASH {
    sha256_var(msg, N as u64)
}

// Variable size SHA-256 hash
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> HASH {
    let message_size = message_size as u32;
    let num_blocks = N / BLOCK_SIZE;
    let mut msg_block: MSG_BLOCK = [0; INT_BLOCK_SIZE];
    // Intermediate hash, starting with the canonical initial value
    let mut h: STATE = [
        1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635,
        1541459225,
    ];
    // Pointer into msg_block on a 64 byte scale
    let mut msg_byte_ptr = 0;
    for i in 0..num_blocks {
        let msg_start = BLOCK_SIZE * i;
        let (new_msg_block, new_msg_byte_ptr) =
            unsafe { build_msg_block(msg, message_size, msg_start) };

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
        if (msg_start < message_size) & (msg_byte_ptr == BLOCK_SIZE) {
            h = sha256_compression(msg_block, h);
        }
    }

    let modulo = N % BLOCK_SIZE;
    // Handle setup of the final msg block.
    // This case is only hit if the msg is less than the block size,
    // or our message cannot be evenly split into blocks.
    if modulo != 0 {
        let msg_start = BLOCK_SIZE * num_blocks;
        let (new_msg_block, new_msg_byte_ptr) =
            unsafe { build_msg_block(msg, message_size, msg_start) };

        if msg_start < message_size {
            msg_block = new_msg_block;
        }

        if !is_unconstrained() {
            let new_msg_byte_ptr = verify_msg_block(msg, message_size, msg_block, msg_start);
            if msg_start < message_size {
                msg_byte_ptr = new_msg_byte_ptr;
                verify_msg_block_padding(msg_block, msg_byte_ptr);
            }
        } else if msg_start < message_size {
            msg_byte_ptr = new_msg_byte_ptr;
        }
    }

    // If we had modulo == 0 then it means the last block was full,
    // and we can reset the pointer to zero to overwrite it.
    if msg_byte_ptr == BLOCK_SIZE {
        msg_byte_ptr = 0;
    }

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).
    // Here we rely on the fact that everything beyond the available input is set to 0.
    msg_block = update_block_item(
        msg_block,
        msg_byte_ptr,
        |msg_item| set_item_byte_then_zeros(msg_item, msg_byte_ptr, 1 << 7),
    );
    msg_byte_ptr = msg_byte_ptr + 1;
    let last_block = msg_block;

    // If we don't have room to write the size, compress the block and reset it.
    if msg_byte_ptr > MSG_SIZE_PTR {
        h = sha256_compression(msg_block, h);
        // `attach_len_to_msg_block` will zero out everything after the `msg_byte_ptr`.
        msg_byte_ptr = 0;
    }

    msg_block = unsafe { attach_len_to_msg_block(msg_block, msg_byte_ptr, message_size) };

    if !is_unconstrained() {
        verify_msg_len(msg_block, last_block, msg_byte_ptr, message_size);
    }

    hash_final_block(msg_block, h)
}

// Take `BLOCK_SIZE` number of bytes from `msg` starting at `msg_start`.
// Returns the block and the length that has been copied rather than padded with zeros.
unconstrained fn build_msg_block<let N: u32>(
    msg: [u8; N],
    message_size: u32,
    msg_start: u32,
) -> (MSG_BLOCK, BLOCK_BYTE_PTR) {
    let mut msg_block: MSG_BLOCK = [0; INT_BLOCK_SIZE];

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

    // Figure out the number of items in the int array that we have to pack.
    // e.g. if the input is [0,1,2,3,4,5] then we need to pack it as 2 items: [0123, 4500]
    let mut int_input = block_input / INT_SIZE;
    if block_input % INT_SIZE != 0 {
        int_input = int_input + 1;
    };

    for i in 0..int_input {
        let mut msg_item: u32 = 0;
        // Always construct the integer as 4 bytes, even if it means going beyond the input.
        for j in 0..INT_SIZE {
            let k = i * INT_SIZE + j;
            let msg_byte = if k < block_input {
                msg[msg_start + k]
            } else {
                0
            };
            msg_item = lshift8(msg_item, 1) + msg_byte as u32;
        }
        msg_block[i] = msg_item;
    }

    // Returning the index as if it was a 64 byte array.
    // We have to project it down to 16 items and bit shifting to get a byte back if we need it.
    (msg_block, block_input)
}

// Verify the block we are compressing was appropriately constructed by `build_msg_block`
// and matches the input data. Returns the index of the first unset item.
// If `message_size` is less than `msg_start` then this is called with the old non-empty block;
// in that case we can skip verification, ie. no need to check that everything is zero.
fn verify_msg_block<let N: u32>(
    msg: [u8; N],
    message_size: u32,
    msg_block: MSG_BLOCK,
    msg_start: u32,
) -> BLOCK_BYTE_PTR {
    let mut msg_byte_ptr = 0;
    let mut msg_end = msg_start + BLOCK_SIZE;
    if msg_end > N {
        msg_end = N;
    }
    // We might have to go beyond the input to pad the fields.
    if msg_end % INT_SIZE != 0 {
        msg_end = msg_end + INT_SIZE - msg_end % INT_SIZE;
    }

    // Reconstructed packed item.
    let mut msg_item: u32 = 0;

    // Inclusive at the end so that we can compare the last item.
    let mut i: u32 = 0;
    for k in msg_start..=msg_end {
        if k % INT_SIZE == 0 {
            // If we consumed some input we can compare against the block.
            if (msg_start < message_size) & (k > msg_start) {
                assert_eq(msg_block[i], msg_item as u32);
                i = i + 1;
                msg_item = 0;
            }
        }
        // Shift the accumulator
        msg_item = lshift8(msg_item, 1);
        // If we have input to consume, add it at the rightmost position.
        if k < message_size & k < msg_end {
            msg_item = msg_item + msg[k] as u32;
            msg_byte_ptr = msg_byte_ptr + 1;
        }
    }

    msg_byte_ptr
}

// Verify the block we are compressing was appropriately padded with zeros by `build_msg_block`.
// This is only relevant for the last, potentially partially filled block.
fn verify_msg_block_padding(msg_block: MSG_BLOCK, msg_byte_ptr: BLOCK_BYTE_PTR) {
    // Check all the way to the end of the block.
    verify_msg_block_zeros(msg_block, msg_byte_ptr, INT_BLOCK_SIZE);
}

// Verify that a region of ints in the message block are (partially) zeroed,
// up to an (exclusive) maximum which can either be the end of the block
// or just where the size is to be written.
fn verify_msg_block_zeros(
    msg_block: MSG_BLOCK,
    mut msg_byte_ptr: BLOCK_BYTE_PTR,
    max_int_byte_ptr: u32,
) {
    // This variable is used to get around the compiler under-constrained check giving a warning.
    // We want to check against a constant zero, but if it does not come from the circuit inputs
    // or return values the compiler check will issue a warning.
    let zero = msg_block[0] - msg_block[0];

    // First integer which is supposed to be (partially) zero.
    let mut int_byte_ptr = msg_byte_ptr / INT_SIZE;

    // Check partial zeros.
    let modulo = msg_byte_ptr % INT_SIZE;
    if modulo != 0 {
        let zeros = INT_SIZE - modulo;
        let mask = if zeros == 3 {
            TWO_POW_24
        } else if zeros == 2 {
            TWO_POW_16
        } else {
            TWO_POW_8
        };
        assert_eq(msg_block[int_byte_ptr] % mask, zero);
        int_byte_ptr = int_byte_ptr + 1;
    }

    // Check the rest of the items.
    for i in 0..max_int_byte_ptr {
        if i >= int_byte_ptr {
            assert_eq(msg_block[i], zero);
        }
    }
}

// Verify that up to the byte pointer the two blocks are equal.
// At the byte pointer the new block can be partially zeroed.
fn verify_msg_block_equals_last(
    msg_block: MSG_BLOCK,
    last_block: MSG_BLOCK,
    mut msg_byte_ptr: BLOCK_BYTE_PTR,
) {
    // msg_byte_ptr is the position at which they are no longer have to be the same.
    // First integer which is supposed to be (partially) zero contains that pointer.
    let mut int_byte_ptr = msg_byte_ptr / INT_SIZE;

    // Check partial zeros.
    let modulo = msg_byte_ptr % INT_SIZE;
    if modulo != 0 {
        // Reconstruct the partially zero item from the last block.
        let last_field = last_block[int_byte_ptr];
        let mut msg_item: u32 = 0;
        // Reset to where they are still equal.
        msg_byte_ptr = msg_byte_ptr - modulo;
        for i in 0..INT_SIZE {
            msg_item = lshift8(msg_item, 1);
            if i < modulo {
                msg_item = msg_item + get_item_byte(last_field, msg_byte_ptr) as u32;
                msg_byte_ptr = msg_byte_ptr + 1;
            }
        }
        assert_eq(msg_block[int_byte_ptr], msg_item);
    }

    for i in 0..INT_SIZE_PTR {
        if i < int_byte_ptr {
            assert_eq(msg_block[i], last_block[i]);
        }
    }
}

// Apply a function on the block item which the pointer indicates.
fn update_block_item<Env>(
    mut msg_block: MSG_BLOCK,
    msg_byte_ptr: BLOCK_BYTE_PTR,
    f: fn[Env](u32) -> u32,
) -> MSG_BLOCK {
    let i = msg_byte_ptr / INT_SIZE;
    msg_block[i] = f(msg_block[i]);
    msg_block
}

// Set the rightmost `zeros` number of bytes to 0.
fn set_item_zeros(item: u32, zeros: u8) -> u32 {
    lshift8(rshift8(item, zeros), zeros)
}

// Replace one byte in the item with a value, and set everything after it to zero.
fn set_item_byte_then_zeros(msg_item: u32, msg_byte_ptr: BLOCK_BYTE_PTR, msg_byte: u8) -> u32 {
    let zeros = INT_SIZE - msg_byte_ptr % INT_SIZE;
    let zeroed_item = set_item_zeros(msg_item, zeros as u8);
    let new_item = byte_into_item(msg_byte, msg_byte_ptr);
    zeroed_item + new_item
}

// Get a byte of a message item according to its overall position in the `BLOCK_SIZE` space.
fn get_item_byte(mut msg_item: u32, msg_byte_ptr: BLOCK_BYTE_PTR) -> u8 {
    // How many times do we have to shift to the right to get to the position we want?
    let max_shifts = INT_SIZE - 1;
    let shifts = max_shifts - msg_byte_ptr % INT_SIZE;
    msg_item = rshift8(msg_item, shifts as u8);
    // At this point the byte we want is in the rightmost position.
    msg_item as u8
}

// Project a byte into a position in a field based on the overall block pointer.
// For example putting 1 into pointer 5 would be 100, because overall we would
// have [____, 0100] with indexes [0123,4567].
fn byte_into_item(msg_byte: u8, msg_byte_ptr: BLOCK_BYTE_PTR) -> u32 {
    let mut msg_item = msg_byte as u32;
    // How many times do we have to shift to the left to get to the position we want?
    let max_shifts = INT_SIZE - 1;
    let shifts = max_shifts - msg_byte_ptr % INT_SIZE;
    lshift8(msg_item, shifts as u8)
}

// Construct a field out of 4 bytes.
fn make_item(b0: u8, b1: u8, b2: u8, b3: u8) -> u32 {
    let mut item = b0 as u32;
    item = lshift8(item, 1) + b1 as u32;
    item = lshift8(item, 1) + b2 as u32;
    item = lshift8(item, 1) + b3 as u32;
    item
}

// Shift by 8 bits to the left between 0 and 4 times.
// Checks `is_unconstrained()` to just use a bitshift if we're running in an unconstrained context,
// otherwise multiplies by 256.
fn lshift8(item: u32, shifts: u8) -> u32 {
    if is_unconstrained() {
        if item == 0 {
            0
        } else {
            // Brillig wouldn't shift 0<<4 without overflow.
            item << (8 * shifts)
        }
    } else {
        // We can do a for loop up to INT_SIZE or an if-else.
        if shifts == 0 {
            item
        } else if shifts == 1 {
            item * TWO_POW_8
        } else if shifts == 2 {
            item * TWO_POW_16
        } else if shifts == 3 {
            item * TWO_POW_24
        } else {
            // Doesn't make sense, but it's most likely called on 0 anyway.
            0
        }
    }
}

// Shift by 8 bits to the right between 0 and 4 times.
// Checks `is_unconstrained()` to just use a bitshift if we're running in an unconstrained context,
// otherwise divides by 256.
fn rshift8(item: u32, shifts: u8) -> u32 {
    if is_unconstrained() {
        item >> (8 * shifts)
    } else {
        // Division wouldn't work on `Field`.
        if shifts == 0 {
            item
        } else if shifts == 1 {
            item / TWO_POW_8
        } else if shifts == 2 {
            item / TWO_POW_16
        } else if shifts == 3 {
            item / TWO_POW_24
        } else {
            0
        }
    }
}

// Zero out all bytes between the end of the message and where the length is appended,
// then write the length into the last 8 bytes of the block.
unconstrained fn attach_len_to_msg_block(
    mut msg_block: MSG_BLOCK,
    mut msg_byte_ptr: BLOCK_BYTE_PTR,
    message_size: u32,
) -> MSG_BLOCK {
    // We assume that `msg_byte_ptr` is less than 57 because if not then it is reset to zero before calling this function.
    // In any case, fill blocks up with zeros until the last 64 bits (i.e. until msg_byte_ptr = 56).
    // There can be one item which has to be partially zeroed.
    let modulo = msg_byte_ptr % INT_SIZE;
    if modulo != 0 {
        // Index of the block in which we find the item we need to partially zero.
        let i = msg_byte_ptr / INT_SIZE;
        let zeros = INT_SIZE - modulo;
        msg_block[i] = set_item_zeros(msg_block[i], zeros as u8);
        msg_byte_ptr = msg_byte_ptr + zeros;
    }

    // The rest can be zeroed without bit shifting anything.
    for i in (msg_byte_ptr / INT_SIZE)..INT_SIZE_PTR {
        msg_block[i] = 0;
    }

    // Set the last two 4 byte ints as the first/second half of the 8 bytes of the length.
    let len = 8 * message_size;
    let len_bytes: [u8; 8] = (len as Field).to_be_bytes();
    for i in 0..=1 {
        let shift = i * 4;
        msg_block[INT_SIZE_PTR + i] = make_item(
            len_bytes[shift],
            len_bytes[shift + 1],
            len_bytes[shift + 2],
            len_bytes[shift + 3],
        );
    }
    msg_block
}

// Verify that the message length was correctly written by `attach_len_to_msg_block`,
// and that everything between the byte pointer and the size pointer was zeroed,
// and that everything before the byte pointer was untouched.
fn verify_msg_len(
    msg_block: MSG_BLOCK,
    last_block: MSG_BLOCK,
    msg_byte_ptr: BLOCK_BYTE_PTR,
    message_size: u32,
) {
    // Check zeros up to the size pointer.
    verify_msg_block_zeros(msg_block, msg_byte_ptr, INT_SIZE_PTR);

    // Check that up to the pointer we match the last block.
    verify_msg_block_equals_last(msg_block, last_block, msg_byte_ptr);

    // We verify the message length was inserted correctly by reversing the byte decomposition.
    let mut reconstructed_len: u64 = 0;
    for i in INT_SIZE_PTR..INT_BLOCK_SIZE {
        reconstructed_len = reconstructed_len * TWO_POW_32;
        reconstructed_len = reconstructed_len + msg_block[i] as u64;
    }
    let len = 8 * message_size as u64;
    assert_eq(reconstructed_len, len);
}

// Perform the final compression, then transform the `STATE` into `HASH`.
fn hash_final_block(msg_block: MSG_BLOCK, mut state: STATE) -> HASH {
    let mut out_h: HASH = [0; 32]; // Digest as sequence of bytes
    // Hash final padded block
    state = sha256_compression(msg_block, state);

    // Return final hash as byte array
    for j in 0..8 {
        let h_bytes: [u8; 4] = (state[j] as Field).to_be_bytes();
        for k in 0..4 {
            out_h[4 * j + k] = h_bytes[k];
        }
    }

    out_h
}

mod tests {
    use super::{
        attach_len_to_msg_block, build_msg_block, byte_into_item, get_item_byte, make_item,
        set_item_byte_then_zeros, set_item_zeros,
    };
    use super::INT_BLOCK;
    use super::sha256_var;

    #[test]
    fn smoke_test() {
        let input = [0xbd];
        let result = [
            0x68, 0x32, 0x57, 0x20, 0xaa, 0xbd, 0x7c, 0x82, 0xf3, 0x0f, 0x55, 0x4b, 0x31, 0x3d,
            0x05, 0x70, 0xc9, 0x5a, 0xcc, 0xbb, 0x7d, 0xc4, 0xb5, 0xaa, 0xe1, 0x12, 0x04, 0xc0,
            0x8f, 0xfe, 0x73, 0x2b,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_just_over_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99,
            111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116,
        ];
        let result = [
            91, 122, 146, 93, 52, 109, 133, 148, 171, 61, 156, 70, 189, 238, 153, 7, 222, 184, 94,
            24, 65, 114, 192, 244, 207, 199, 87, 232, 192, 224, 171, 207,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_multiple_over_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99,
            111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116, 61, 117, 115, 45, 97, 115,
            99, 105, 105, 13, 10, 109, 105, 109, 101, 45, 118, 101, 114, 115, 105, 111, 110, 58, 49,
            46, 48, 32, 40, 77, 97, 99, 32, 79, 83, 32, 88, 32, 77, 97, 105, 108, 32, 49, 54, 46,
            48, 32, 92, 40, 51, 55, 51, 49, 46, 53, 48, 48, 46, 50, 51, 49, 92, 41, 41, 13, 10, 115,
            117, 98, 106, 101, 99, 116, 58, 72, 101, 108, 108, 111, 13, 10, 109, 101, 115, 115, 97,
            103, 101, 45, 105, 100, 58, 60, 56, 70, 56, 49, 57, 68, 51, 50, 45, 66, 54, 65, 67, 45,
            52, 56, 57, 68, 45, 57, 55, 55, 70, 45, 52, 51, 56, 66, 66, 67, 52, 67, 65, 66, 50, 55,
            64, 109, 101, 46, 99, 111, 109, 62, 13, 10, 100, 97, 116, 101, 58, 83, 97, 116, 44, 32,
            50, 54, 32, 65, 117, 103, 32, 50, 48, 50, 51, 32, 49, 50, 58, 50, 53, 58, 50, 50, 32,
            43, 48, 52, 48, 48, 13, 10, 116, 111, 58, 122, 107, 101, 119, 116, 101, 115, 116, 64,
            103, 109, 97, 105, 108, 46, 99, 111, 109, 13, 10, 100, 107, 105, 109, 45, 115, 105, 103,
            110, 97, 116, 117, 114, 101, 58, 118, 61, 49, 59, 32, 97, 61, 114, 115, 97, 45, 115,
            104, 97, 50, 53, 54, 59, 32, 99, 61, 114, 101, 108, 97, 120, 101, 100, 47, 114, 101,
            108, 97, 120, 101, 100, 59, 32, 100, 61, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109,
            59, 32, 115, 61, 49, 97, 49, 104, 97, 105, 59, 32, 116, 61, 49, 54, 57, 51, 48, 51, 56,
            51, 51, 55, 59, 32, 98, 104, 61, 55, 120, 81, 77, 68, 117, 111, 86, 86, 85, 52, 109, 48,
            87, 48, 87, 82, 86, 83, 114, 86, 88, 77, 101, 71, 83, 73, 65, 83, 115, 110, 117, 99, 75,
            57, 100, 74, 115, 114, 99, 43, 118, 85, 61, 59, 32, 104, 61, 102, 114, 111, 109, 58, 67,
            111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 77, 105, 109, 101, 45, 86, 101,
            114, 115, 105, 111, 110, 58, 83, 117, 98, 106, 101, 99,
        ];
        let result = [
            116, 90, 151, 31, 78, 22, 138, 180, 211, 189, 69, 76, 227, 200, 155, 29, 59, 123, 154,
            60, 47, 153, 203, 129, 157, 251, 48, 2, 79, 11, 65, 47,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_just_under_block() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99,
            111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 59,
        ];
        let result = [
            143, 140, 76, 173, 222, 123, 102, 68, 70, 149, 207, 43, 39, 61, 34, 79, 216, 252, 213,
            165, 74, 16, 110, 74, 29, 64, 138, 167, 30, 1, 9, 119,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_big_not_block_multiple() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99,
            111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116, 61, 117, 115, 45, 97, 115,
            99, 105, 105, 13, 10, 109, 105, 109, 101, 45, 118, 101, 114, 115, 105, 111, 110, 58, 49,
            46, 48, 32, 40, 77, 97, 99, 32, 79, 83, 32, 88, 32, 77, 97, 105, 108, 32, 49, 54, 46,
            48, 32, 92, 40, 51, 55, 51, 49, 46, 53, 48, 48, 46, 50, 51, 49, 92, 41, 41, 13, 10, 115,
            117, 98, 106, 101, 99, 116, 58, 72, 101, 108, 108, 111, 13, 10, 109, 101, 115, 115, 97,
            103, 101, 45, 105, 100, 58, 60, 56, 70, 56, 49, 57, 68, 51, 50, 45, 66, 54, 65, 67, 45,
            52, 56, 57, 68, 45, 57, 55, 55, 70, 45, 52, 51, 56, 66, 66, 67, 52, 67, 65, 66, 50, 55,
            64, 109, 101, 46, 99, 111, 109, 62, 13, 10, 100, 97, 116, 101, 58, 83, 97, 116, 44, 32,
            50, 54, 32, 65, 117, 103, 32, 50, 48, 50, 51, 32, 49, 50, 58, 50, 53, 58, 50, 50, 32,
            43, 48, 52, 48, 48, 13, 10, 116, 111, 58, 122, 107, 101, 119, 116, 101, 115, 116, 64,
            103, 109, 97, 105, 108, 46, 99, 111, 109, 13, 10, 100, 107, 105, 109, 45, 115, 105, 103,
            110, 97, 116, 117, 114, 101, 58, 118, 61, 49, 59, 32, 97, 61, 114, 115, 97, 45, 115,
            104, 97, 50, 53, 54, 59, 32, 99, 61, 114, 101, 108, 97, 120, 101, 100, 47, 114, 101,
            108, 97, 120, 101, 100, 59, 32, 100, 61, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109,
            59, 32, 115, 61, 49, 97, 49, 104, 97, 105, 59, 32, 116, 61, 49, 54, 57, 51, 48, 51, 56,
            51, 51, 55, 59, 32, 98, 104, 61, 55, 120, 81, 77, 68, 117, 111, 86, 86, 85, 52, 109, 48,
            87, 48, 87, 82, 86, 83, 114, 86, 88, 77, 101, 71, 83, 73, 65, 83, 115, 110, 117, 99, 75,
            57, 100, 74, 115, 114, 99, 43, 118, 85, 61, 59, 32, 104, 61, 102, 114, 111, 109, 58, 67,
            111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 77, 105, 109, 101, 45, 86, 101,
            114, 115, 105, 111, 110, 58, 83, 117, 98, 106, 101, 99, 116, 58, 77, 101, 115, 115, 97,
            103, 101, 45, 73, 100, 58, 68, 97, 116, 101, 58, 116, 111, 59, 32, 98, 61,
        ];
        let result = [
            112, 144, 73, 182, 208, 98, 9, 238, 54, 229, 61, 145, 222, 17, 72, 62, 148, 222, 186,
            55, 192, 82, 220, 35, 66, 47, 193, 200, 22, 38, 26, 186,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn msg_big_with_padding() {
        let input = [
            48, 130, 1, 37, 2, 1, 0, 48, 11, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 48, 130, 1, 17,
            48, 37, 2, 1, 1, 4, 32, 176, 223, 31, 133, 108, 84, 158, 102, 70, 11, 165, 175, 196, 12,
            201, 130, 25, 131, 46, 125, 156, 194, 28, 23, 55, 133, 157, 164, 135, 136, 220, 78, 48,
            37, 2, 1, 2, 4, 32, 190, 82, 180, 235, 222, 33, 79, 50, 152, 136, 142, 35, 116, 224, 6,
            242, 156, 141, 128, 248, 10, 61, 98, 86, 248, 45, 207, 210, 90, 232, 175, 38, 48, 37, 2,
            1, 3, 4, 32, 0, 194, 104, 108, 237, 246, 97, 230, 116, 198, 69, 110, 26, 87, 17, 89,
            110, 199, 108, 250, 36, 21, 39, 87, 110, 102, 250, 213, 174, 131, 171, 174, 48, 37, 2,
            1, 11, 4, 32, 136, 155, 87, 144, 111, 15, 152, 127, 85, 25, 154, 81, 20, 58, 51, 75,
            193, 116, 234, 0, 60, 30, 29, 30, 183, 141, 72, 247, 255, 203, 100, 124, 48, 37, 2, 1,
            12, 4, 32, 41, 234, 106, 78, 31, 11, 114, 137, 237, 17, 92, 71, 134, 47, 62, 78, 189,
            233, 201, 214, 53, 4, 47, 189, 201, 133, 6, 121, 34, 131, 64, 142, 48, 37, 2, 1, 13, 4,
            32, 91, 222, 210, 193, 62, 222, 104, 82, 36, 41, 138, 253, 70, 15, 148, 208, 156, 45,
            105, 171, 241, 195, 185, 43, 217, 162, 146, 201, 222, 89, 238, 38, 48, 37, 2, 1, 14, 4,
            32, 76, 123, 216, 13, 51, 227, 72, 245, 59, 193, 238, 166, 103, 49, 23, 164, 171, 188,
            194, 197, 156, 187, 249, 28, 198, 95, 69, 15, 182, 56, 54, 38, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let result = [
            32, 85, 108, 174, 127, 112, 178, 182, 8, 43, 134, 123, 192, 211, 131, 66, 184, 240, 212,
            181, 240, 180, 106, 195, 24, 117, 54, 129, 19, 10, 250, 53,
        ];
        let message_size = 297;
        assert_eq(sha256_var(input, message_size), result);
    }

    #[test]
    fn msg_big_no_padding() {
        let input = [
            48, 130, 1, 37, 2, 1, 0, 48, 11, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 48, 130, 1, 17,
            48, 37, 2, 1, 1, 4, 32, 176, 223, 31, 133, 108, 84, 158, 102, 70, 11, 165, 175, 196, 12,
            201, 130, 25, 131, 46, 125, 156, 194, 28, 23, 55, 133, 157, 164, 135, 136, 220, 78, 48,
            37, 2, 1, 2, 4, 32, 190, 82, 180, 235, 222, 33, 79, 50, 152, 136, 142, 35, 116, 224, 6,
            242, 156, 141, 128, 248, 10, 61, 98, 86, 248, 45, 207, 210, 90, 232, 175, 38, 48, 37, 2,
            1, 3, 4, 32, 0, 194, 104, 108, 237, 246, 97, 230, 116, 198, 69, 110, 26, 87, 17, 89,
            110, 199, 108, 250, 36, 21, 39, 87, 110, 102, 250, 213, 174, 131, 171, 174, 48, 37, 2,
            1, 11, 4, 32, 136, 155, 87, 144, 111, 15, 152, 127, 85, 25, 154, 81, 20, 58, 51, 75,
            193, 116, 234, 0, 60, 30, 29, 30, 183, 141, 72, 247, 255, 203, 100, 124, 48, 37, 2, 1,
            12, 4, 32, 41, 234, 106, 78, 31, 11, 114, 137, 237, 17, 92, 71, 134, 47, 62, 78, 189,
            233, 201, 214, 53, 4, 47, 189, 201, 133, 6, 121, 34, 131, 64, 142, 48, 37, 2, 1, 13, 4,
            32, 91, 222, 210, 193, 62, 222, 104, 82, 36, 41, 138, 253, 70, 15, 148, 208, 156, 45,
            105, 171, 241, 195, 185, 43, 217, 162, 146, 201, 222, 89, 238, 38, 48, 37, 2, 1, 14, 4,
            32, 76, 123, 216, 13, 51, 227, 72, 245, 59, 193, 238, 166, 103, 49, 23, 164, 171, 188,
            194, 197, 156, 187, 249, 28, 198, 95, 69, 15, 182, 56, 54, 38,
        ];
        let result = [
            32, 85, 108, 174, 127, 112, 178, 182, 8, 43, 134, 123, 192, 211, 131, 66, 184, 240, 212,
            181, 240, 180, 106, 195, 24, 117, 54, 129, 19, 10, 250, 53,
        ];
        assert_eq(sha256_var(input, input.len() as u64), result);
    }

    #[test]
    fn same_msg_len_variable_padding() {
        let input = [
            29, 81, 165, 84, 243, 114, 101, 37, 242, 146, 127, 99, 69, 145, 39, 72, 213, 39, 253,
            179, 218, 37, 217, 201, 172, 93, 198, 50, 249, 70, 15, 30, 162, 112, 187, 40, 140, 9,
            236, 53, 32, 44, 38, 163, 113, 254, 192, 197, 44, 89, 71, 130, 169, 242, 17, 211, 214,
            72, 19, 178, 186, 168, 147, 127, 99, 101, 252, 227, 8, 147, 150, 85, 97, 158, 17, 107,
            218, 244, 82, 113, 247, 91, 208, 214, 60, 244, 87, 137, 173, 201, 130, 18, 66, 56, 198,
            149, 207, 189, 175, 120, 123, 224, 177, 167, 251, 159, 143, 110, 68, 183, 189, 70, 126,
            32, 35, 164, 44, 30, 44, 12, 65, 18, 62, 239, 242, 2, 248, 104, 2, 178, 64, 28, 126, 36,
            137, 24, 14, 116, 91, 98, 90, 159, 218, 102, 45, 11, 110, 223, 245, 184, 52, 99, 59,
            245, 136, 175, 3, 72, 164, 146, 145, 116, 22, 66, 24, 49, 193, 121, 3, 60, 37, 41, 97,
            3, 190, 66, 195, 225, 63, 46, 3, 118, 4, 208, 15, 1, 40, 254, 235, 151, 123, 70, 180,
            170, 44, 172, 90, 4, 254, 53, 239, 116, 246, 67, 56, 129, 61, 22, 169, 213, 65, 27, 216,
            116, 162, 239, 214, 207, 126, 177, 20, 100, 25, 48, 143, 84, 215, 70, 197, 53, 65, 70,
            86, 172, 61, 62, 9, 212, 167, 169, 133, 41, 126, 213, 196, 33, 192, 238, 0, 63, 246,
            215, 58, 128, 110, 101, 92, 3, 170, 214, 130, 149, 52, 81, 125, 118, 233, 3, 118, 193,
            104, 207, 120, 115, 77, 253, 191, 122, 0, 107, 164, 207, 113, 81, 169, 36, 201, 228, 74,
            134, 131, 218, 178, 35, 30, 216, 101, 2, 103, 174, 87, 95, 50, 50, 215, 157, 5, 210,
            188, 54, 211, 78, 45, 199, 96, 121, 241, 241, 176, 226, 194, 134, 130, 89, 217, 210,
            186, 32, 140, 39, 91, 103, 212, 26, 87, 32, 72, 144, 228, 230, 117, 99, 188, 50, 15, 69,
            79, 179, 50, 12, 106, 86, 218, 101, 73, 142, 243, 29, 250, 122, 228, 233, 29, 255, 22,
            121, 114, 125, 103, 41, 250, 241, 179, 126, 158, 198, 116, 209, 65, 94, 98, 228, 175,
            169, 96, 3, 9, 233, 133, 214, 55, 161, 164, 103, 80, 85, 24, 186, 64, 167, 92, 131, 53,
            101, 202, 47, 25, 104, 118, 155, 14, 12, 12, 25, 116, 45, 221, 249, 28, 246, 212, 200,
            157, 167, 169, 56, 197, 181, 4, 245, 146, 1, 140, 234, 191, 212, 228, 125, 87, 81, 86,
            119, 30, 63, 129, 143, 32, 96,
        ];

        // Prepare inputs of different lengths
        let mut input_511 = [0; 511];
        let mut input_512 = [0; 512]; // Next block
        let mut input_575 = [0; 575];
        let mut input_576 = [0; 576]; // Next block
        for i in 0..input.len() {
            input_511[i] = input[i];
            input_512[i] = input[i];
            input_575[i] = input[i];
            input_576[i] = input[i];
        }

        // Compute hashes of all inputs (with same message length)
        let fixed_length_hash = super::sha256(input);
        let var_full_length_hash = sha256_var(input, input.len() as u64);
        let var_length_hash_511 = sha256_var(input_511, input.len() as u64);
        let var_length_hash_512 = sha256_var(input_512, input.len() as u64);
        let var_length_hash_575 = sha256_var(input_575, input.len() as u64);
        let var_length_hash_576 = sha256_var(input_576, input.len() as u64);

        // All of the above should have produced the same hash
        assert_eq(var_full_length_hash, fixed_length_hash);
        assert_eq(var_length_hash_511, fixed_length_hash);
        assert_eq(var_length_hash_512, fixed_length_hash);
        assert_eq(var_length_hash_575, fixed_length_hash);
        assert_eq(var_length_hash_576, fixed_length_hash);
    }

    #[test]
    fn test_get_item_byte() {
        let fld = make_item(10, 20, 30, 40);
        assert_eq(fld, 0x0a141e28);
        assert_eq(get_item_byte(fld, 0), 10);
        assert_eq(get_item_byte(fld, 4), 10);
        assert_eq(get_item_byte(fld, 6), 30);
    }

    #[test]
    fn test_byte_into_item() {
        let fld = make_item(0, 20, 0, 0);
        assert_eq(byte_into_item(20, 1), fld);
        assert_eq(byte_into_item(20, 5), fld);
    }

    #[test]
    fn test_set_item_zeros() {
        let fld0 = make_item(10, 20, 30, 40);
        let fld1 = make_item(10, 0, 0, 0);
        assert_eq(set_item_zeros(fld0, 3), fld1);
        assert_eq(set_item_zeros(fld0, 4), 0);
        assert_eq(set_item_zeros(0, 4), 0);
    }

    #[test]
    fn test_set_item_byte_then_zeros() {
        let fld0 = make_item(10, 20, 30, 40);
        let fld1 = make_item(10, 50, 0, 0);
        assert_eq(set_item_byte_then_zeros(fld0, 1, 50), fld1);
    }

    #[test]
    fn test_build_msg_block_start_0() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48,
        ];
        assert_eq(input.len(), 22);
        let (msg_block, msg_byte_ptr) = unsafe { build_msg_block(input, input.len(), 0) };
        assert_eq(msg_byte_ptr, input.len());
        assert_eq(msg_block[0], make_item(input[0], input[1], input[2], input[3]));
        assert_eq(msg_block[1], make_item(input[4], input[5], input[6], input[7]));
        assert_eq(msg_block[5], make_item(input[20], input[21], 0, 0));
        assert_eq(msg_block[6], 0);
    }

    #[test]
    fn test_build_msg_block_start_1() {
        let input = [
            102, 114, 111, 109, 58, 114, 117, 110, 110, 105, 101, 114, 46, 108, 101, 97, 103, 117,
            101, 115, 46, 48, 106, 64, 105, 99, 108, 111, 117, 100, 46, 99, 111, 109, 13, 10, 99,
            111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 59, 32, 99, 104, 97, 114, 115, 101, 116,
        ];
        assert_eq(input.len(), 68);
        let (msg_block, msg_byte_ptr) = unsafe { build_msg_block(input, input.len(), 64) };
        assert_eq(msg_byte_ptr, 4);
        assert_eq(msg_block[0], make_item(input[64], input[65], input[66], input[67]));
        assert_eq(msg_block[1], 0);
    }

    #[test]
    fn test_attach_len_to_msg_block() {
        let input: INT_BLOCK = [
            2152555847, 1397309779, 1936618851, 1262052426, 1936876331, 1985297723, 543702374,
            1919905082, 1131376244, 1701737517, 1417244773, 978151789, 1697470053, 1920166255,
            1849316213, 1651139939,
        ];
        let msg_block = unsafe { attach_len_to_msg_block(input, 1, 448) };
        assert_eq(msg_block[0], ((1 << 7) as u32) * 256 * 256 * 256);
        assert_eq(msg_block[1], 0);
        assert_eq(msg_block[15], 3584);
    }
}
