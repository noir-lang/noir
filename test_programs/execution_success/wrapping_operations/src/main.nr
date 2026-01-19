use std::ops::{WrappingAdd, WrappingMul, WrappingSub};

fn main(x: u8, y: u8) {
    assert(x.wrapping_sub(1) == y);
    assert(y.wrapping_add(1) == x);
    assert(y.wrapping_mul(y) == 1);
}
