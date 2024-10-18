// Test unsafe integer arithmetic
// Test odd bits integer
fn main(mut x: u32, y: u32) {
    x = std::wrapping_mul(x, x);
    assert(y == x);

    let c: u1 = 0;
    assert(x as u1 > c);
}
