fn main(x: u8) {
    // This would previously overflow in ACIR. Now it returns zero.
    let value = 1 >> x;
    assert_eq(value, 0);
}
