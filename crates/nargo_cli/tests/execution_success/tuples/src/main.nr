fn main(x: Field, y: Field) {
    let pair = (x, y);
    assert_eq(pair.0, 1);
    assert_eq(pair.1, 0);

    let (a, b) = if true { (0, 1) } else { (2, 3) };
    assert_eq(a, 0);
    assert_eq(b, 1);

    let (u,v) = if x as u32 < 1 {
        (x, x + 1)
    } else {
        (x + 1, x)
    };
    assert_eq(u, x+1);
    assert_eq(v, x);

    // Test mutating tuples
    let mut mutable = ((0, 0), 1, 2, 3);
    mutable.0 = (x, y);
    mutable.2 = 7;
    assert_eq(mutable.0.0, 1);
    assert_eq(mutable.0.1, 0);
    assert_eq(mutable.1, 1);
    assert_eq(mutable.2, 7);
    assert_eq(mutable.3, 3);
}
