use std::ops::WrappingSub;

// Test unsafe integer subtraction with underflow: 12 - 2418266113 = 1876701195 modulo 2^32
fn main(mut x: u32, y: u32, z: u32) {
    x = x.wrapping_sub(y);
    assert(x == z);
}
