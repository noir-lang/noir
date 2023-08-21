fn main(x: Field, y: Field) {
    let mut z = x + y;
    assert_eq(z, 3);
    z = x * y;
    assert_eq(z, 2);
}
