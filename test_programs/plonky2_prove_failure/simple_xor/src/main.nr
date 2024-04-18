// Tests that 28 `xor` 1002 == 900. Expected to fail.
fn main(x: u64, y: u64, z: u64) {
    assert(x ^ y == z);
}
