use std::hash::{poseidon, poseidon2::Poseidon2};

global NUM_HASHES: u32 = 3;
global HASH_LENGTH: u32 = 20;

pub fn poseidon_hash<let N: u32>(inputs: [Field; N]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

fn main(
    to_hash: [[Field; HASH_LENGTH]; NUM_HASHES],
    enable: [bool; NUM_HASHES],
) -> pub [Field; NUM_HASHES] {
    let mut result = [0; NUM_HASHES];
    for i in 0..NUM_HASHES {
        let enable = enable[i];
        let to_hash = to_hash[i];
        if enable {
            result[i] = poseidon_hash(to_hash) + poseidon::bn254::sponge(to_hash);
        }
    }
    result
}
