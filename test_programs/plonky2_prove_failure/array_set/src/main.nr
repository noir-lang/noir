// Tests that set does not have effect on the array. Should fail.
fn main(mut x: [u8; 4], y: [u8; 4], w: u8) {
    x[2] = w;
    assert(x == y);
}
