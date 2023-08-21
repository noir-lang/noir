// Tests a very simple program.
// 
// The features being tested is hash_to_field in brillig

use dep::std;

fn main(input: Field) -> pub Field {
    hash_to_field(input)
}

unconstrained fn hash_to_field(input : Field) -> Field {
    std::hash::hash_to_field([input])
}
