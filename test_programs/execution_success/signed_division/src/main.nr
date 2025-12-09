use std::ops::WrappingSub;

// Testing signed integer division:
//  7/3  = 2
// -7/3  = -2
// -7/-3 = 2
//  7/-3 = -2
fn main(mut x: i32, mut y: i32, mut z: i32) {
    // 7/3 = 2
    assert(x / y == z);
    // -7/3 = -2
    let minus_x = 0i32.wrapping_sub(x);
    let minus_z = 0i32.wrapping_sub(z);
    let minus_y = 0i32.wrapping_sub(y);
    assert(x + minus_x == 0);
    assert(z + minus_z == 0);
    assert(minus_x / y == minus_z);
    // -7/-3 = 2
    assert(minus_x / minus_y == z);
    // 7/-3 = -2
    assert(x / minus_y == minus_z);
}
