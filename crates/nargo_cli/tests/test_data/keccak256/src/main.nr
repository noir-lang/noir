// Keccak256 example
//
use dep::std;

fn main(x: Field, result: [u8; 32]) {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
    // The padding is taken care of by the program
    let digest = std::hash::keccak256([x as u8]);
    assert(digest == result);
}
