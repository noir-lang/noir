// Tests a very simple program.
//
// The features being tested are array reads and writes
fn main(x: [Field; 3]) {
    // Safety: testing context
    unsafe {
        read_array(x);
        read_write_array(x);
    }
}

unconstrained fn read_array(
    /// The signature internally will be represented as `(r, s)`
    /// where `r` and `s` are fixed-sized big endian scalar values.
    /// As the `secp256k1` has a 256-bit modulus, we have a 64 byte signature
    /// while `r` and `s` will both be 32 bytes.
    /// We expect `s` to be normalized. This means given an the curve's order,
    /// `s` should be less than or equal to `order / 2`. 
    /// This is done to prevent malleability. 
    /// For more context regarding malleability you can reference BIP 0062.
    x: [Field; 3]
) {
    assert(x[0] == 1);
    let y = [1, 5, 27];

    assert(y[x[0]] == 5);
}

unconstrained fn read_write_array(x: [Field; 3]) {
    let mut y = x;

    y[0] = 5;

    assert(y[0] == 5);
    assert(y[1] == 2);
    assert(y[2] == 3);

    assert(x[0] == 1);
    assert(x[1] == 2);
    assert(x[2] == 3);
}
