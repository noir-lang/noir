// Test that should fail, because 7 - 3 != 3.
fn main(x: u64, y: u64, z: u64) {
    assert(x - y == z);
}
