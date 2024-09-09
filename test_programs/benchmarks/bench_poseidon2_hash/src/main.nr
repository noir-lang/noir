use std::hash::poseidon2;

fn main(input: [Field; 2]) -> pub Field {
    poseidon2::Poseidon2::hash(input, input.len())
}
