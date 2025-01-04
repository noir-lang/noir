// Tests a very simple program.
//
// The features being tested are array reads and writes
fn main(x: [Field; 3]) {
    unsafe {
        //@safety: testing context
        read_array(x);
        read_write_array(x);
    }
}

unconstrained fn read_array(x: [Field; 3]) {
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
