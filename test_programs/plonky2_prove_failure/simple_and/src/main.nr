// Tests that 999 & 101 == 11. Expected to fail.
fn main(x: u64, y: u64, z: u64) {
    assert(x & y == z);
}
