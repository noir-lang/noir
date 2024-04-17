use dep::std::hash::{pedersen_hash_with_separator, poseidon2::Poseidon2};

global NUM_HASHES = 2;
global HASH_LENGTH = 10;

pub fn pedersen_hash<N>(inputs: [Field; N], hash_index: u32) -> Field {
    dep::std::hash::pedersen_hash_with_separator(inputs, hash_index)
}

#[fold]
pub fn poseidon_hash<N>(inputs: [Field; N]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

fn main(
    to_hash: [[Field; HASH_LENGTH]; NUM_HASHES],
    enable: [bool; NUM_HASHES]
) -> pub [Field; NUM_HASHES] {
    let mut result = [0; NUM_HASHES];
    for i in 0..NUM_HASHES {
        let enable = enable[i];
        let to_hash = to_hash[i];
        if enable {
            // result[i] = pedersen_hash(to_hash, 0);
            result[i] = poseidon_hash(to_hash);
        }
    }
    result
}