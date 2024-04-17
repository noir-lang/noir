// Test should fail, because division only works on matching bit sizes.
fn main(x: u64, y: u8) {
    x / y;
}
