// This test was relevant due to a regression in circuit size that only
// really came out in aztec-packages. Moving forward we can use it to
// have a reduced test case to monitor the opcode count.
pub struct Poseidon2Sponge {
    pub cache: [Field; 3],
    pub state: [Field; 4],
}

fn absorb_full_permutations<let NUM_ITEMS: u32, let MAX_PERMUTATIONS: u32>(
    mut sponge: Poseidon2Sponge,
    input: [Field; NUM_ITEMS],
    num_permutations: u32,
) -> Poseidon2Sponge {
    let mut should_add = true;

    for i in 0..MAX_PERMUTATIONS {
        should_add &= i != num_permutations;
        let chunk_base_index = i * 3;
        if should_add {
            sponge.cache[1] = input[chunk_base_index];
            sponge.cache[2] = input[chunk_base_index + 1];
            for j in 0..3 {
                // This conditional array_set was not optimized well.
                sponge.state[j] += sponge.cache[j];
            }
            sponge.state = std::hash::poseidon2_permutation(sponge.state, 4);
            sponge.cache[0] = input[chunk_base_index + 2];
        }
    }

    sponge
}

fn absorb<let NUM_ITEMS: u32>(
    mut sponge: Poseidon2Sponge,
    input: [Field; NUM_ITEMS],
    input_len: u32,
) -> Poseidon2Sponge {
    absorb_full_permutations::<NUM_ITEMS, (NUM_ITEMS - (NUM_ITEMS % 3)) / 3>(
        sponge,
        input,
        input_len / 3,
    )
}

fn main(input: [Field; 12], input_len: u32) -> pub [Field; 4] {
    let sponge = Poseidon2Sponge { cache: [0, 0, 0], state: [0, 0, 0, 0] };
    let sponge = absorb(sponge, input, input_len);
    sponge.state
}
