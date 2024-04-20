use dep::std;
// Tests a very simple program.
// 
// The features being tested is sha256 in brillig
fn main(x: field, result: [u8; 32]) {
    assert(result == sha256(x));
}

unconstrained fn sha256(x: field) -> [u8; 32] {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x field
    // The padding is taken care of by the program
    std::hash::sha256([x as u8])
}

