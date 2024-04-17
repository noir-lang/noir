// Test should fail, because multiplication only works on matching bit sizes.
fn main(x: u32, y: u8) {
    assert(x * y == x);
}
