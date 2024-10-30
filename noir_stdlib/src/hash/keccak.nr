use crate::runtime::is_unconstrained;

global BLOCK_SIZE_IN_BYTES: u32 = 136; //(1600 - BITS * 2) / WORD_SIZE;
global WORD_SIZE: u32 = 8; // Limbs are made up of u64s so 8 bytes each.
global LIMBS_PER_BLOCK: u32 = BLOCK_SIZE_IN_BYTES / WORD_SIZE;
global NUM_KECCAK_LANES: u32 = 25;

#[foreign(keccakf1600)]
fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}

#[no_predicates]
pub(crate) fn keccak256<let N: u32>(input: [u8; N], message_size: u32) -> [u8; 32] {
    assert(N >= message_size);

    // Copy input to block bytes. For that we'll need at least input bytes (N)
    // but we want it to be padded to a multiple of BLOCK_SIZE_IN_BYTES.
    let mut block_bytes = [0; ((N / BLOCK_SIZE_IN_BYTES) + 1) * BLOCK_SIZE_IN_BYTES];
    if is_unconstrained() {
        for i in 0..message_size {
            block_bytes[i] = input[i];
        }
    } else {
        for i in 0..N {
            if i < message_size {
                block_bytes[i] = input[i];
            }
        }
    }

    //1. format_input_lanes
    let max_blocks = (N + BLOCK_SIZE_IN_BYTES) / BLOCK_SIZE_IN_BYTES;
    //maximum number of bytes to hash
    let real_max_blocks = (message_size + BLOCK_SIZE_IN_BYTES) / BLOCK_SIZE_IN_BYTES;
    let real_blocks_bytes = real_max_blocks * BLOCK_SIZE_IN_BYTES;

    block_bytes[message_size] = 1;
    block_bytes[real_blocks_bytes - 1] = 0x80;

    // populate a vector of 64-bit limbs from our byte array
    let mut sliced_buffer =
        [0; (((N / BLOCK_SIZE_IN_BYTES) + 1) * BLOCK_SIZE_IN_BYTES) / WORD_SIZE];
    for i in 0..sliced_buffer.len() {
        let limb_start = WORD_SIZE * i;

        let mut sliced = 0;
        let mut v = 1;
        for k in 0..WORD_SIZE {
            sliced += v * (block_bytes[limb_start + k] as Field);
            v *= 256;
        }

        sliced_buffer[i] = sliced as u64;
    }

    //2. sponge_absorb
    let mut state: [u64; NUM_KECCAK_LANES] = [0; NUM_KECCAK_LANES];
    // When in an unconstrained runtime we can take advantage of runtime loop bounds,
    // thus allowing us to simplify the loop body.
    if is_unconstrained() {
        for i in 0..real_max_blocks {
            if (i == 0) {
                for j in 0..LIMBS_PER_BLOCK {
                    state[j] = sliced_buffer[j];
                }
            } else {
                for j in 0..LIMBS_PER_BLOCK {
                    state[j] = state[j] ^ sliced_buffer[i * LIMBS_PER_BLOCK + j];
                }
            }
            state = keccakf1600(state);
        }
    } else {
        // `real_max_blocks` is guaranteed to at least be `1`
        // We peel out the first block as to avoid a conditional inside of the loop.
        // Otherwise, a dynamic predicate can cause a blowup in a constrained runtime.
        for j in 0..LIMBS_PER_BLOCK {
            state[j] = sliced_buffer[j];
        }
        state = keccakf1600(state);
        for i in 1..max_blocks {
            if i < real_max_blocks {
                for j in 0..LIMBS_PER_BLOCK {
                    state[j] = state[j] ^ sliced_buffer[i * LIMBS_PER_BLOCK + j];
                }
                state = keccakf1600(state);
            }
        }
    }

    //3. sponge_squeeze
    let mut result = [0; 32];
    for i in 0..4 {
        let lane = state[i] as Field;
        let lane_le: [u8; 8] = lane.to_le_bytes();
        for j in 0..8 {
            result[8 * i + j] = lane_le[j];
        }
    }
    result
}

mod tests {
    use super::keccak256;

    #[test]
    fn smoke_test() {
        let input = [0xbd];
        let result = [
            0x5a, 0x50, 0x2f, 0x9f, 0xca, 0x46, 0x7b, 0x26, 0x6d, 0x5b, 0x78, 0x33, 0x65, 0x19,
            0x37, 0xe8, 0x05, 0x27, 0x0c, 0xa3, 0xf3, 0xaf, 0x1c, 0x0d, 0xd2, 0x46, 0x2d, 0xca,
            0x4b, 0x3b, 0x1a, 0xbf,
        ];
        assert_eq(keccak256(input, input.len()), result);
    }

    #[test]
    fn hash_hello_world() {
        let input = "Hello world!".as_bytes();
        let result = [
            0xec, 0xd0, 0xe1, 0x8, 0xa9, 0x8e, 0x19, 0x2a, 0xf1, 0xd2, 0xc2, 0x50, 0x55, 0xf4, 0xe3,
            0xbe, 0xd7, 0x84, 0xb5, 0xc8, 0x77, 0x20, 0x4e, 0x73, 0x21, 0x9a, 0x52, 0x3, 0x25, 0x1f,
            0xea, 0xab,
        ];
        assert_eq(keccak256(input, input.len()), result);
    }

    #[test]
    fn var_size_hash() {
        let input = [
            189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205,
            206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222,
            223,
        ];
        let result = [
            226, 37, 115, 94, 94, 196, 72, 116, 194, 105, 79, 233, 65, 12, 30, 94, 181, 131, 170,
            219, 171, 166, 236, 88, 143, 67, 255, 160, 248, 214, 39, 129,
        ];
        assert_eq(keccak256(input, 13), result);
    }

    #[test]
    fn hash_longer_than_136_bytes() {
        let input = "123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789"
            .as_bytes();
        assert(input.len() > 136);

        let result = [
            0x1d, 0xca, 0xeb, 0xdf, 0xd9, 0xd6, 0x24, 0x67, 0x1c, 0x18, 0x16, 0xda, 0xd, 0x8a, 0xeb,
            0xa8, 0x75, 0x71, 0x2c, 0xc, 0x89, 0xe0, 0x25, 0x2, 0xe8, 0xb6, 0x5e, 0x16, 0x5, 0x55,
            0xe4, 0x40,
        ];
        assert_eq(keccak256(input, input.len()), result);
    }
}
