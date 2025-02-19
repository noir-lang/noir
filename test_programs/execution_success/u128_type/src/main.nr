fn main(x: u128, y: u128) {
    let const_x = 12345;
    let const_y = 2345678;

    assert_eq(x + y, const_x + const_y);
    assert_eq(y - x, const_y - const_x);
    assert_eq(x * y, const_x * const_y);
    assert_eq(y / x, const_y / const_x);
    assert_eq(y % x, const_y % const_x);
    assert_eq(!x, !const_x);
    assert_eq(x ^ y, const_x ^ const_y);
    assert_eq(x & y, const_x & const_y);
    assert_eq(x | y, const_x | const_y);
    assert_eq(x >> 2, const_x >> 2);
    assert_eq(x << 2, const_x << 2);
    assert_eq(x < y, const_x < const_y);
    assert_eq(x <= y, const_x <= const_y);
    assert_eq(x != y, const_x != const_y);
    assert_eq(y > x, const_y > const_x);
    assert_eq(y >= x, const_y >= const_x);
}
