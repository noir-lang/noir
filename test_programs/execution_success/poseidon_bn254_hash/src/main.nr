// docs:start:poseidon
use std::hash::poseidon;

fn main(x1: [Field; 2], y1: pub Field, x2: [Field; 4], y2: pub Field) {
    let hash1 = poseidon::bn254::hash_2(x1);
    assert(hash1 == y1);

    let hash2 = poseidon::bn254::hash_4(x2);
    assert(hash2 == y2);
}
// docs:end:poseidon
