// Test extracting the least significant bit from signed integers
// (previously tested via `as u1`, now uses `as u8 % 2`)
fn main(
    zero: i32,
    one: i32,
    two: i32,
    three: i32,
    minus_one: i32,
    minus_two: i32,
    minus_three: i32,
) {
    assert_eq(zero as u8 % 2, 0);
    assert_eq(one as u8 % 2, 1);
    assert_eq(two as u8 % 2, 0);
    assert_eq(three as u8 % 2, 1);
    assert_eq(minus_one as u8 % 2, 1);
    assert_eq(minus_two as u8 % 2, 0);
    assert_eq(minus_three as u8 % 2, 1);

    let zero: i32 = 0;
    let one: i32 = 1;
    let two: i32 = 2;
    let three: i32 = 3;
    let minus_one: i32 = -1;
    let minus_two: i32 = -2;
    let minus_three: i32 = -3;
    assert_eq(zero as u8 % 2, 0);
    assert_eq(one as u8 % 2, 1);
    assert_eq(two as u8 % 2, 0);
    assert_eq(three as u8 % 2, 1);
    assert_eq(minus_one as u8 % 2, 1);
    assert_eq(minus_two as u8 % 2, 0);
    assert_eq(minus_three as u8 % 2, 1);
}
