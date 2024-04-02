use dep::std::hash::{pedersen_hash_with_separator, poseidon2::Poseidon2};

global NUM_HASHES = 4;
global HASH_LENGTH = 20;

pub fn pedersen_hash<N>(inputs: [Field; N], hash_index: u32) -> Field {
    dep::std::hash::pedersen_hash_with_separator(inputs, hash_index)
}

pub fn poseidon_hash<N>(inputs: [Field; N]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

pub fn poseideon_hash_internal<N>(inputs: [Field; N]) -> Field {
    Poseidon2::hash_internal(inputs, inputs.len(), false)
}

fn main(
    to_hash: [Field; HASH_LENGTH],
    enable: bool
) -> pub Field {
    let mut result = 0;
    if enable { 
        // result = poseidon_hash(to_hash);
        // result = pedersen_hash(to_hash, 0);
        result = poseidon_in_fold(to_hash);
    }
    result
}

// fn main(
//     to_hash: [[Field; HASH_LENGTH]; NUM_HASHES],
//     enable: [bool; NUM_HASHES],
//     // to_hash: [Field; HASH_LENGTH],
//     // enable: bool
// ) -> pub [Field; NUM_HASHES] {
//     let mut result = [0; NUM_HASHES];
//     for i in 0..NUM_HASHES {
//         let enable = enable[i];
//         let to_hash = to_hash[i];
//         if enable {
//             // result[i] = pedersen_hash(to_hash, 0);
//             // result[i] = poseidon_hash(to_hash);
//             // result[i] = poseideon_hash_internal(to_hash);
//             // Failing with Assertion `(arg.mul_terms.size() <= 1)
//             // result[i] = poseidon_in_fold(to_hash);
//         }
//     }
//     result
// }

#[fold]
fn poseidon_in_fold(inputs: [Field; HASH_LENGTH]) -> Field {
    poseidon_hash(inputs)
}