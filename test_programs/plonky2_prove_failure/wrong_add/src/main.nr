// Test that should fail, because 2 + 2 != 5.
fn main(x: u64, y: u64, z: u64) {
    assert(x + y == z);
}
