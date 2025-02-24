use std::hash;

global SIZE: u32 = 30;

fn main(input: [[Field; 2]; SIZE]) -> pub [Field; SIZE] {
    let mut results: [Field; SIZE] = [0; SIZE];
    for i in 0..SIZE {
        results[i] = hash::poseidon::bn254::hash_2(input[i]);
    }

    results
}
