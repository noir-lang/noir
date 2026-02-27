use std::hash::poseidon2_permutation;

pub fn main(input1: [Field; 16], input2: [Field; 4], flag: bool) -> pub Field {
    if flag {
        poseidon2_permutation(input1, 16)[0]
    } else {
        poseidon2_permutation(input2, 4)[0]
    }
}
