use std::hash::poseidon2_permutation;

fn main(input1: [Field; 16], input2: [Field; 4], flag: bool) -> pub Field {
    if flag {
        // Even if on a disabled branch, a call with unexpected input length should cause compile failure.
        poseidon2_permutation(input1, 16)[0]
    } else {
        poseidon2_permutation(input2, 4)[0]
    }
}
