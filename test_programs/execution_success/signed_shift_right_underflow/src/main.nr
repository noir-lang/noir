fn main(a: i64, b: u8) {
    // This would previously underflow in ACIR and return -1. Now it returns zero.
    let res = a >> b;
    assert_eq(res, 0);
}
