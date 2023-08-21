// Tests a very simple program.
// 
// The features being tested is sha256 in brillig

use dep::std;

fn main(x: Field, result: [u8; 32]) {
    assert(result == sha256(x));
}

unconstrained fn sha256(x: Field) -> [u8; 32] {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
    // The padding is taken care of by the program
    std::hash::sha256([x as u8])
}

