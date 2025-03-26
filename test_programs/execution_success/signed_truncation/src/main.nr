fn main(x: i64) {
    assert_eq(x as i8, -1);
    assert_eq(x as i16, -1);
    assert_eq(x as i32, -1);
    assert_eq(x as i64, 0x7fffffffffffffff);
}
