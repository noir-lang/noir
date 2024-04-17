// Test should fail, because subtraction only works on matching bit sizes.
fn main(x: u1, y: u64) {
    x - y;
}
