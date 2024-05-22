global LIMBS_PER_BLOCK = 17;    //BLOCK_SIZE / 8;
global NUM_KECCAK_LANES = 25;

fn hash<N>(input: [u8; N], input_length: u64) -> [u8; 32]
{ 
    // no var keccak for now
    assert(N == input_length);

    //1. format_input_lanes
    let max_blocks = (N as u64 + BLOCK_SIZE) / BLOCK_SIZE;
    let max_blocks_length = (BLOCK_SIZE * (max_blocks));        //maximum number of bytes to hash
    assert(max_blocks_length < 1000);
    let mut block_bytes = [0; 1000];   //should be max_blocks_length
    for i in 0..N {
        block_bytes[i] = input[i];
    }
    let byte_difference = max_blocks_length - N;
    block_bytes[N] = 1;
    block_bytes[max_blocks_length - 1] = 0x80;


    // keccak lanes interpret memory as little-endian integers,
    // means we need to swap our byte ordering...
    let num_limbs = max_blocks * LIMBS_PER_BLOCK;   //max_blocks_length / WORD_SIZE;
    for i in 0..num_limbs{
        let mut temp = [0;8];

        for j in 0..8 {
            temp[j] = block_bytes[8*i+j];
        }
        for j in 0..8 {
            block_bytes[8*i+j] = temp[7-j];
        }
    }
    let  byte_size = max_blocks_length;
    assert(num_limbs < 1000);
    let mut sliced_buffer = [0;1000];
    // populate a vector of 64-bit limbs from our byte array
    for i in 0..num_limbs {
        let mut sliced = 0;
        if ((i as u64) * WORD_SIZE + WORD_SIZE > byte_size) {
            let slice_size = byte_size - (i * WORD_SIZE);
            let byte_shift = (WORD_SIZE - slice_size) * 8;
            let mut v = 1;
            for k in 0..slice_size {
                sliced += v*(block_bytes[i * WORD_SIZE+7-k] as Field);
                v = v*256;
            }
            let w = 1 << byte_shift;
            sliced *= w as Field;
        }else {
            let mut v = 1;
            for k in 0..WORD_SIZE {
                sliced += v*(block_bytes[i * WORD_SIZE+7-k] as Field);
                v = v*256;
            }
        }
        sliced_buffer[i] =sliced as u64;
    }

    //2. sponge_absorb
    let num_blocks = max_blocks;
    let mut state : [u64;NUM_KECCAK_LANES]= [0; NUM_KECCAK_LANES];
    for i in 0..num_blocks {
          if (i == 0) {
            for j in 0..LIMBS_PER_BLOCK {
                state[j] = sliced_buffer[j];
            }
        } else {
            for j in 0..LIMBS_PER_BLOCK {
                state[j] = state[j] ^ sliced_buffer[i * LIMBS_PER_BLOCK + j];
            }   
        }
        state = std::hash::keccakf1600(state);
    }

    //3. sponge_squeeze
    let mut result = [0; 32];

    // Each hash limb represents a little-endian integer. Need to reverse bytes before we write into the output array
    for i in 0..4 {
        let lane = state[i] as Field;
        let lane_le = lane.to_le_bytes(8);
        for j in 0..8 {
            result[8*i+j] = lane_le[7-j];
        }
    }
    result
}

fn main(input: [u8; 1], input_length: u64, result: [u8; 32])  {
    assert_eq(hash(input, input_length), result);
}