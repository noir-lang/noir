use std::hash::poseidon;

fn main(input: [Field; 2]) -> pub Field {
    poseidon2::Poseidon2::hash(input, input.len())
}
