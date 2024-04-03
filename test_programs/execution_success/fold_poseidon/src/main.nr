use dep::std::hash::{pedersen_hash_with_separator, poseidon2::Poseidon2};

global NUM_HASHES = 4;
global HASH_LENGTH = 20;

pub fn pedersen_hash<N>(inputs: [Field; N], hash_index: u32) -> Field {
    dep::std::hash::pedersen_hash_with_separator(inputs, hash_index)
}

pub fn poseidon_hash<N>(inputs: [Field; N]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

fn main(
    to_hash: [Field; HASH_LENGTH],
    enable: bool
) -> pub [Field; NUM_HASHES] {
// ) -> pub Field {
    // let mut result = 0;
    // if enable {
    //     // result = pedersen_hash(to_hash, 0);
    //     // result = poseidon_hash(to_hash);
    //     result = poseidon_hash_fold(to_hash);
    // }
    // result

    let mut result = [0; NUM_HASHES];
    for i in 0..NUM_HASHES {
        if enable {
            result[i] = poseidon_hash_fold(to_hash);
        }
    }
    result
}

#[fold]
pub fn poseidon_hash_fold(inputs: [Field; HASH_LENGTH]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

// TODO: for now this is type checked but we should allow more types here as 
// we can figure out the size during compilation
// #[fold]
// pub fn poseidon_hash_fold<N>(inputs: [Field; N]) -> Field {
//     Poseidon2::hash(inputs, inputs.len())
// }