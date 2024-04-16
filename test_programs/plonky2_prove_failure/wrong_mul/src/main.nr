// Test that fails because 3 * 3 is not 10.
fn main(x: u64, y: u64, z: u64) {
    assert(x * y == z);
}
