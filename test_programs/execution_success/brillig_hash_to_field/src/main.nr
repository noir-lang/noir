use dep::std;
// Tests a very simple program.
// 
// The features being tested is hash_to_field in brillig
fn main(input: field) -> pub field {
    hash_to_field(input)
}

unconstrained fn hash_to_field(input: field) -> field {
    std::hash::hash_to_field([input])
}
