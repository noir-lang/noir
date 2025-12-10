fn main(x: u8) {
    // 1>>8 and 1>>9 would previously return zero, now they overflow.
    let value = 1 >> 7;
    assert_eq(value, 0);
}
