// docs:start:poseidon2
use std::hash::poseidon2;

fn main(inputs: [Field; 4], expected_hash: Field) {
    let hash = poseidon2::Poseidon2::hash(inputs, inputs.len());
    assert_eq(hash, expected_hash);
}
// docs:end:poseidon2
