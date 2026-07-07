// Test unsafe integer arithmetic
// Test odd bits integer
use std::ops::WrappingMul;
fn main(mut x: u32, y: u32) {
    x = x.wrapping_mul(x);
    assert(y == x);

    let c: u8 = 0;
    assert(x as u8 % 2 > c);
}
