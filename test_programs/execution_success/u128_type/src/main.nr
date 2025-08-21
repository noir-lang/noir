fn main(x: u128, y: u128, z: u128) {
    let const_x = 12345;
    let const_y = 2345678;
    let const_z = 2;

    assert_eq(x + y, const_x + const_y);
    assert_eq(y - x, const_y - const_x);
    assert_eq(x * y, const_x * const_y);
    assert_eq(y / x, const_y / const_x);
    assert_eq(y % x, const_y % const_x);
    assert_eq(!x, !const_x);
    assert_eq(x ^ y, const_x ^ const_y);
    assert_eq(x & y, const_x & const_y);
    assert_eq(x | y, const_x | const_y);
    assert_eq(x >> z, const_x >> const_z);
    assert_eq(x << z, const_x << const_z);
    assert_eq(x < y, const_x < const_y);
    assert_eq(x <= y, const_x <= const_y);
    assert_eq(x != y, const_x != const_y);
    assert_eq(y > x, const_y > const_x);
    assert_eq(y >= x, const_y >= const_x);
}
