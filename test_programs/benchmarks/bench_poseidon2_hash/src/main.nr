fn main(input: [Field; 2]) -> pub Field {
    poseidon::poseidon2::Poseidon2::hash(input, input.len())
}
