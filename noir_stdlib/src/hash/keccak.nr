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

    let mut block_bytes = Vec::from_slice(input.as_slice());
    for _i in N..N + BLOCK_SIZE {
        block_bytes.push(0);
    }
    block_bytes.set(message_size, 1);
    block_bytes.set(real_blocks_bytes - 1, 0x80);

    // keccak lanes interpret memory as little-endian integers,
    // means we need to swap our byte ordering
    let num_limbs = max_blocks * LIMBS_PER_BLOCK; //max_blocks_length / WORD_SIZE;
    for i in 0..num_limbs {
        let mut temp = [0; 8];
        for j in 0..8 {
            temp[j] = block_bytes.get(8*i+j);
        }
        for j in 0..8 {
            block_bytes.set(8 * i + j, temp[7 - j]);
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
                sliced += v * (block_bytes.get(i * WORD_SIZE+7-k) as Field);
                v *= 256;
            }
            let w = 1 << (byte_shift as u8);
            sliced *= w as Field;
        } else {
            let mut v = 1;
            for k in 0..WORD_SIZE {
                sliced += v * (block_bytes.get(i * WORD_SIZE+7-k) as Field);
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
