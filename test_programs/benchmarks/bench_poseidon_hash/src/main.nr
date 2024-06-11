use dep::std::hash::poseidon;

fn main(input: [Field; 2]) -> pub Field {
    poseidon::bn254::hash_2(input)
}
