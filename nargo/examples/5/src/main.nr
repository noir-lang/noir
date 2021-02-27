   
// Sha256 example
// 
// Calls Sha256 from the standard library.
//
// The Compiler sees this special function and creates an ACIR gate
//
// The ACIR SHA256 gate is passed to PLONK who should
// know how to create the necessary constraints.
//
// Not yet here: For R1CS, it is more about manipulating arithmetic gates to get performance
// This can be done in ACIR!
use dep::std;

fn main(x : Witness,low : Public, high: Public) {

    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Witness
    // The padding is taken care of by the program
    let digest = std::hash::sha256([x as u8]);
     
    constrain digest[0] == low;
    constrain digest[1] == high;
}