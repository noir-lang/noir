fn main(input: [Field; 2]) -> pub Field {
    poseidon::poseidon::bn254::hash_2(input)
}
