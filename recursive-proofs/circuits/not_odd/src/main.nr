#[recursive]
fn main(n: Field) -> pub u8 {
    // not_odd(n);
    n as u8
}

// fn main(x: Field, y: Field) -> pub bool {
//     not_odd(x) & not_odd(y) & not_equal(x, y);
// }

fn not_equal(x: Field, y: Field) -> bool {
    x != y
}

fn not_odd(n: Field) -> bool {
    (n as u8) & 1 == 0
}

#[test]
fn test_not_equal() {
    assert(not_equal(1, 2));
    // Uncomment to make test fail
    // assert(not_equal(1, 1));
}

fn test_not_odd() {
    assert(not_odd(2));
    // Uncomment to make test fail
    // assert(not_odd(1));
}
