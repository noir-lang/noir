// Test should fail, because addition only works on matching bit sizes.
fn main(x: Field, y: u64) {
    x + y;
}
