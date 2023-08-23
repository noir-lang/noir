// Tests a very simple program.
// 
// The features being tested are array reads and writes

fn main(x: [Field; 3])  {
    read_array(x);
    read_write_array(x);
}

unconstrained fn read_array(x: [Field; 3]) {
    assert_eq(x[0], 1);
    let y = [1, 5, 27];

    assert_eq(y[x[0]], 5);
}

unconstrained fn read_write_array(x: [Field; 3]) {
    let mut y = x;

    y[0] = 5;

    assert_eq(y[0], 5);
    assert_eq(y[1], 2);
    assert_eq(y[2], 3);

    assert_eq(x[0], 1);
    assert_eq(x[1], 2);
    assert_eq(x[2], 3);
}
