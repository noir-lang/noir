// Test should fail, because allowed bit sizes are 1, 8, 32, and 64.
fn main(x: u60) {
    assert(x == x);
}
