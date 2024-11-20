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
fn main(x: Field, result: [u8; 32], input: [u8; 2], toggle: bool) {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
    // The padding is taken care of by the program
    // docs:start:sha256_var
    let digest = std::hash::sha256_var([x as u8], 1);
    // docs:end:sha256_var
    assert(digest == result);

    let digest = std::hash::sha256([x as u8]);
    assert(digest == result);

    // variable size
    let size: Field = 1 + toggle as Field;
    let var_sha = std::hash::sha256_var(input, size as u64);
    assert(var_sha == std::hash::sha256_var(input, 1));
}
