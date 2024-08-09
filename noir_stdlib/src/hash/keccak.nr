global LIMBS_PER_BLOCK = 17; //BLOCK_SIZE / 8;
global NUM_KECCAK_LANES = 25;
global BLOCK_SIZE = 136; //(1600 - BITS * 2) / WORD_SIZE;
global WORD_SIZE = 8;

use crate::collections::vec::Vec;

#[foreign(keccakf1600)]
fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}

#[no_predicates]
pub(crate) fn keccak256<let N: u32>(mut input: [u8; N], message_size: u32) -> [u8; 32] {
    assert(N >= message_size);
    for i in 0..N {
        if i >= message_size {
            input[i] = 0;
        }
    }

    //1. format_input_lanes
    let max_blocks = (N + BLOCK_SIZE) / BLOCK_SIZE;
    //maximum number of bytes to hash
    let max_blocks_length = (BLOCK_SIZE * (max_blocks));
    let real_max_blocks = (message_size + BLOCK_SIZE) / BLOCK_SIZE;
    let real_blocks_bytes = real_max_blocks * BLOCK_SIZE;

    let mut block_bytes = [0; BLOCK_SIZE];
    for i in 0..N {
        block_bytes[i] = input[i];
    }

    block_bytes[message_size] = 1;
    block_bytes[real_blocks_bytes - 1] = 0x80;

    // keccak lanes interpret memory as little-endian integers,
    // means we need to swap our byte ordering
    let num_limbs = max_blocks * LIMBS_PER_BLOCK; //max_blocks_length / WORD_SIZE;
    for i in 0..num_limbs {
        let mut temp = [0; 8];
        for j in 0..8 {
            temp[j] = block_bytes[8*i+j];
        }
        for j in 0..8 {
            block_bytes[8 * i + j] = temp[7 - j];
        }
    }
    let  byte_size = max_blocks_length;
    let mut sliced_buffer = Vec::new();
    for _i in 0..num_limbs {
        sliced_buffer.push(0);
    }
    // populate a vector of 64-bit limbs from our byte array
    for i in 0..num_limbs {
        let mut sliced = 0;
        if (i * WORD_SIZE + WORD_SIZE > byte_size) {
            let slice_size = byte_size - (i * WORD_SIZE);
            let byte_shift = (WORD_SIZE - slice_size) * 8;
            let mut v = 1;
            for k in 0..slice_size {
                sliced += v * (block_bytes[i * WORD_SIZE+7-k] as Field);
                v *= 256;
            }
            let w = 1 << (byte_shift as u8);
            sliced *= w as Field;
        } else {
            let mut v = 1;
            for k in 0..WORD_SIZE {
                sliced += v * (block_bytes[i * WORD_SIZE+7-k] as Field);
                v *= 256;
            }
        }
        sliced_buffer.set(i, sliced as u64);
    }

    //2. sponge_absorb
    let num_blocks = max_blocks;
    let mut state : [u64;NUM_KECCAK_LANES]= [0; NUM_KECCAK_LANES];
    let mut under_block = true;
    for i in 0..num_blocks {
        if i == real_max_blocks {
            under_block = false;
        }
        if under_block {
            if (i == 0) {
                for j in 0..LIMBS_PER_BLOCK {
                    state[j] = sliced_buffer.get(j);
                }
            } else {
                for j in 0..LIMBS_PER_BLOCK {
                    state[j] = state[j] ^ sliced_buffer.get(i * LIMBS_PER_BLOCK + j);
                }
            }
            state = keccakf1600(state);
        }
    }

    //3. sponge_squeeze
    let mut result = [0; 32];
    for i in 0..4 {
        let lane = state[i] as Field;
        let lane_le = lane.to_le_bytes(8);
        for j in 0..8 {
            result[8*i+j] = lane_le[j];
        }
    }
    result
}

mod tests {
    use crate::hash::keccak::keccak256;

    #[test]
    fn smoke_test() {
        let input = [0xbd];
        let result = [
            0x5a, 0x50, 0x2f, 0x9f, 0xca, 0x46, 0x7b, 0x26, 0x6d, 0x5b, 0x78, 0x33, 0x65, 0x19, 0x37, 0xe8, 0x05, 0x27, 0x0c, 0xa3, 0xf3, 0xaf, 0x1c, 0x0d, 0xd2, 0x46, 0x2d, 0xca, 0x4b, 0x3b, 0x1a, 0xbf
        ];
        assert_eq(keccak256(input, input.len()), result);
    }

    #[test]
    fn hash_hello_world() {
        // "hello world"
        let input = [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33];
        let result = [
            0xec, 0xd0, 0xe1, 0x8, 0xa9, 0x8e, 0x19, 0x2a, 0xf1, 0xd2, 0xc2, 0x50, 0x55, 0xf4, 0xe3, 0xbe, 0xd7, 0x84, 0xb5, 0xc8, 0x77, 0x20, 0x4e, 0x73, 0x21, 0x9a, 0x52, 0x3, 0x25, 0x1f, 0xea, 0xab
        ];
        assert_eq(keccak256(input, input.len()), result);
    }

    #[test]
    fn var_size_hash() {
        let input = [
            189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223
        ];
        let result = [
            226, 37, 115, 94, 94, 196, 72, 116, 194, 105, 79, 233, 65, 12, 30, 94, 181, 131, 170, 219, 171, 166, 236, 88, 143, 67, 255, 160, 248, 214, 39, 129
        ];
        assert_eq(keccak256(input, 13), result);
    }
}

