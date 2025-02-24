use std::hash::poseidon2::Poseidon2;

global NUM_HASHES: u32 = 2;
global HASH_LENGTH: u32 = 10;

#[fold]
pub fn poseidon_hash<let N: u32>(inputs: [Field; N]) -> Field {
    Poseidon2::hash(inputs, inputs.len())
}

fn main(
    to_hash: [[Field; HASH_LENGTH]; NUM_HASHES],
    enable: [bool; NUM_HASHES],
) -> pub [Field; NUM_HASHES + 1] {
    let mut result = [0; NUM_HASHES + 1];
    for i in 0..NUM_HASHES {
        let enable = enable[i];
        let to_hash = to_hash[i];
        if enable {
            result[i] = poseidon_hash(to_hash);
        }
    }

    // We want to make sure that the foldable function with a numeric generic
    // is monomorphized correctly.
    let mut double_preimage = [0; 20];
    for i in 0..HASH_LENGTH * 2 {
        double_preimage[i] = to_hash[0][i % HASH_LENGTH];
    }
    result[NUM_HASHES] = poseidon_hash(double_preimage);

    result
}
