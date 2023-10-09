use dep::std;
// Testing signed integer division: 
//  7/3  = 2
// -7/3  = -2
// -7/-3 = 2
//  7/-3 = -2
fn main(mut x: i32, mut y: i32, mut z: i32) {
    // 7/3 = 2
    assert(x / y == z);

    // -7/3 = -2
    let zero: i32 = 0;
    let minus_x: i32 = std::wrapping_sub(zero,x);
    let minus_z = std::wrapping_sub(0,z);
    let minus_y = std::wrapping_sub(0,y);
    // TODO: Fix addition and subtraction for signed integers
    // We should not have to wrap to do 7 + (-7)
    // let x_plus_minus_x = std::wrapping_add(x, minus_x);
    // This will print `0` as we expect for normal signed addition
    // std::println(x_plus_minus_x);
    assert(x+minus_x == 0);
    assert(z+minus_z == 0);
    assert(minus_x / y == minus_z);

    // -7/-3 = 2
    assert(minus_x / minus_y == z);
   
    // 7/-3 = -2
    assert(x / minus_y == minus_z);
}
